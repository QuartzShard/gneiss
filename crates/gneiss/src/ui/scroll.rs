//! [`ScrollLayer`], a layer whose content scrolls, and [`ScrollWindow`], a window built around
//! one.
extern crate alloc;
use alloc::boxed::Box;
use core::{ffi::c_void, marker::PhantomData, mem::ManuallyDrop, ptr::NonNull};

use crate::{
	alloc::PebbleAlloc,
	fail::PblUnwrap,
	sys,
	ui::{
		AnyWindow, AsLayer, Button, Click, ClickKinds, Layout, NoWindow, RootLayer, TypedWindow,
		UiGuard, UiQueue, Window, WindowCtx,
		click::{self, ClickHandler},
		graphics::{Colour, Point, Rectangle, Size, gpoint, grect, gsize, point, rect, size},
		window::{Header, LeakWindow, WindowData, drop_window},
	},
};

/// The height of the strips a [`ScrollLayer`]'s arrows draw in. The firmware's own dialogs use the
/// same.
const ARROW_HEIGHT: u32 = if cfg!(pbl_round) { 10 } else { 16 };

/// A layer whose content scrolls.
///
/// Layers added to its [`content`](Self::content) scroll. Scrolling stops at the edges of the
/// [content size](Self::set_content_size). Layers added to the scroll layer itself don't scroll,
/// and draw over the content.
///
/// It doesn't handle any buttons. Call [`scroll_up`](Self::scroll_up) and
/// [`scroll_down`](Self::scroll_down) from your own click handlers, and subscribe them with
/// [`ClickKinds::REPEATING`] to keep scrolling while the button is held. A [`ScrollWindow`] sets
/// this up for you.
pub struct ScrollLayer {
	ptr:     NonNull<sys::ScrollLayer>,
	// A plain layer filling the scroll layer's content, to hand out as its root
	content: NonNull<sys::Layer>,
	// The strips the up and down arrows draw in, over the content
	arrows:  Option<[NonNull<sys::Layer>; 2]>,
}

impl ScrollLayer {
	/// Create an empty scroll layer. Its content size starts as `frame`'s size. `None` if the
	/// firmware is out of memory.
	pub fn new(frame: Rectangle) -> Option<Self> {
		// SAFETY: no preconditions; NULL on OOM
		let ptr = NonNull::new(unsafe { sys::scroll_layer_create(grect(frame)) })?;
		let bounds = Rectangle::new(Point::zero(), frame.size);
		// SAFETY: NULL on OOM, and the scroll layer is live
		unsafe {
			let Some(content) = NonNull::new(sys::layer_create(grect(bounds))) else {
				sys::scroll_layer_destroy(ptr.as_ptr());
				return None;
			};
			sys::scroll_layer_add_child(ptr.as_ptr(), content.as_ptr());
			Some(Self {
				ptr,
				content,
				arrows: None,
			})
		}
	}

	/// The scrolling content, to add layers to.
	pub fn content(&mut self) -> RootLayer {
		RootLayer::from_ptr(self.content.as_ptr())
	}

	/// Set the content's size. Scrolling stops at its edges.
	pub fn set_content_size(&mut self, size: Size) -> &mut Self {
		// SAFETY: both layers are live
		unsafe {
			sys::layer_set_frame(
				self.content.as_ptr(),
				grect(Rectangle::new(Point::zero(), size)),
			);
			sys::scroll_layer_set_content_size(self.ptr.as_ptr(), gsize(size));
		}
		self
	}

	/// The content's size.
	pub fn content_size(&self) -> Size {
		size(unsafe { sys::scroll_layer_get_content_size(self.ptr.as_ptr()) })
	}

	/// Set the content size to fit `frames`, the frames of the content's layers. Passing only the
	/// ones that reach furthest is enough. The width never shrinks below the scroll layer's.
	pub fn fit_content(&mut self, frames: impl IntoIterator<Item = Rectangle>) -> &mut Self {
		let (mut right, mut bottom) = (self.frame().size.width as i32, 0);
		for frame in frames {
			right = right.max(frame.top_left.x + frame.size.width as i32);
			bottom = bottom.max(frame.top_left.y + frame.size.height as i32);
		}
		self.set_content_size(Size::new(right as u32, bottom.max(0) as u32))
	}

	/// The position of the content's top left corner. Scrolling down by 40 makes it `(0, -40)`.
	pub fn content_offset(&self) -> Point {
		point(unsafe { sys::scroll_layer_get_content_offset(self.ptr.as_ptr()) })
	}

	/// Scroll to `offset`, in the same form as [`content_offset`](Self::content_offset). It's
	/// clamped, so it never scrolls past the content's edges.
	pub fn set_content_offset(&mut self, offset: Point, animated: bool) -> &mut Self {
		unsafe {
			sys::scroll_layer_set_content_offset(self.ptr.as_ptr(), gpoint(offset), animated);
		}
		self
	}

	/// Scroll up one step, or one page when paging. Up does this in a [`ScrollWindow`].
	pub fn scroll_up(&mut self) {
		// SAFETY: the firmware's handler ignores the recognizer, and takes the scroll layer as its
		// context
		unsafe {
			sys::scroll_layer_scroll_up_click_handler(
				core::ptr::null_mut(),
				self.ptr.as_ptr().cast(),
			);
		}
	}

	/// Scroll down one step, or one page when paging. Down does this in a [`ScrollWindow`].
	pub fn scroll_down(&mut self) {
		// SAFETY: as for scroll_up
		unsafe {
			sys::scroll_layer_scroll_down_click_handler(
				core::ptr::null_mut(),
				self.ptr.as_ptr().cast(),
			);
		}
	}

	/// Scroll by whole pages instead of steps. A page is the scroll layer's height.
	pub fn set_paging(&mut self, paging: bool) -> &mut Self {
		unsafe { sys::scroll_layer_set_paging(self.ptr.as_ptr(), paging) };
		self
	}

	/// Hide or show the shadow at the top and bottom edges, where content scrolls out of view.
	/// Paging hides it.
	pub fn set_shadow_hidden(&mut self, hidden: bool) -> &mut Self {
		unsafe { sys::scroll_layer_set_shadow_hidden(self.ptr.as_ptr(), hidden) };
		self
	}

	/// Move or resize the scroll layer within its parent. The arrows move with it.
	pub fn set_frame(&mut self, frame: Rectangle) -> &mut Self {
		unsafe { sys::layer_set_frame(self.as_ptr(), grect(frame)) };
		if let Some(arrows) = self.arrows {
			for (arrow, frame) in arrows.into_iter().zip(Self::arrow_frames(frame.size)) {
				unsafe { sys::layer_set_frame(arrow.as_ptr(), grect(frame)) };
			}
		}
		self
	}

	/// Show an arrow at the top or bottom edge while there's more content that way. Each arrow is
	/// `foreground` on a strip of `background`. Call it again to change the colours. `None` if the
	/// firmware is out of memory.
	pub fn show_arrows(&mut self, foreground: Colour, background: Colour) -> Option<&mut Self> {
		let scroll = self.ptr.as_ptr();
		// SAFETY: the scroll layer is live; NULL if the firmware's indicators are used up
		let indicator = unsafe { sys::scroll_layer_get_content_indicator(scroll) };
		if indicator.is_null() {
			return None;
		}
		let arrows = if let Some(arrows) = self.arrows {
			arrows
		} else {
			let arrows = self.create_arrows()?;
			self.arrows = Some(arrows);
			arrows
		};
		let alignments = if cfg!(pbl_round) {
			[sys::GAlign_GAlignBottom, sys::GAlign_GAlignTop]
		} else {
			[sys::GAlign_GAlignCenter; 2]
		};
		let directions = [
			sys::ContentIndicatorDirection_ContentIndicatorDirectionUp,
			sys::ContentIndicatorDirection_ContentIndicatorDirectionDown,
		];
		for ((arrow, alignment), direction) in arrows.into_iter().zip(alignments).zip(directions) {
			let config = sys::ContentIndicatorConfig {
				layer:     arrow.as_ptr(),
				times_out: false,
				alignment: alignment as _,
				colors:    sys::ContentIndicatorConfig__bindgen_ty_1 {
					foreground: foreground.into(),
					background: background.into(),
				},
			};
			// SAFETY: the indicator and strip are live, and the strip outlives the indicator,
			// which drops with the scroll layer
			unsafe {
				sys::content_indicator_configure_direction(indicator, direction, &raw const config)
			};
		}
		// The indicator only checks for content when the offset is set
		let offset = self.content_offset();
		Some(self.set_content_offset(offset, false))
	}

	/// Stop drawing arrows.
	pub fn hide_arrows(&mut self) -> &mut Self {
		let Some(arrows) = self.arrows.take() else {
			return self;
		};
		// SAFETY: the scroll layer and strips are live. A NULL config resets a direction, and the
		// indicator stops drawing in that strip
		unsafe {
			let indicator = sys::scroll_layer_get_content_indicator(self.ptr.as_ptr());
			if !indicator.is_null() {
				for direction in [
					sys::ContentIndicatorDirection_ContentIndicatorDirectionUp,
					sys::ContentIndicatorDirection_ContentIndicatorDirectionDown,
				] {
					sys::content_indicator_configure_direction(
						indicator,
						direction,
						core::ptr::null(),
					);
				}
			}
			for arrow in arrows {
				sys::layer_destroy(arrow.as_ptr());
			}
		}
		self
	}

	fn create_arrows(&mut self) -> Option<[NonNull<sys::Layer>; 2]> {
		let [up, down] = Self::arrow_frames(self.frame().size);
		// SAFETY: NULL on OOM, and the scroll layer is live
		unsafe {
			let up = NonNull::new(sys::layer_create(grect(up)))?;
			let Some(down) = NonNull::new(sys::layer_create(grect(down))) else {
				sys::layer_destroy(up.as_ptr());
				return None;
			};
			sys::layer_add_child(self.as_ptr(), up.as_ptr());
			sys::layer_add_child(self.as_ptr(), down.as_ptr());
			Some([up, down])
		}
	}

	fn arrow_frames(size: Size) -> [Rectangle; 2] {
		let strip = Size::new(size.width, ARROW_HEIGHT);
		let bottom = size.height as i32 - ARROW_HEIGHT as i32;
		[
			Rectangle::new(Point::zero(), strip),
			Rectangle::new(Point::new(0, bottom), strip),
		]
	}

	fn frame(&self) -> Rectangle {
		rect(unsafe { sys::layer_get_frame(self.as_ptr()) })
	}
}

unsafe impl AsLayer for ScrollLayer {
	fn as_ptr(&self) -> *mut sys::Layer {
		unsafe { sys::scroll_layer_get_layer(self.ptr.as_ptr()) }
	}
}

impl Drop for ScrollLayer {
	fn drop(&mut self) {
		// SAFETY: each layer's children are removed before it's destroyed, so no child is left
		// pointing at a freed parent. The indicator is destroyed with the scroll layer, before the
		// strips it draws in
		unsafe {
			sys::layer_remove_child_layers(self.content.as_ptr());
			sys::layer_destroy(self.content.as_ptr());
			sys::layer_remove_child_layers(self.as_ptr());
			sys::scroll_layer_destroy(self.ptr.as_ptr());
			for arrow in self.arrows.into_iter().flatten() {
				sys::layer_destroy(arrow.as_ptr());
			}
		}
	}
}

/// A window whose layout scrolls, with arrows showing where there's more.
///
/// In the layout, [`ctx.root()`](WindowCtx::root) is the scrolling content, and
/// [`ctx.scroll()`](WindowCtx::scroll) is the [`ScrollLayer`]. Set a
/// [content size](ScrollLayer::set_content_size), or nothing scrolls. Put layers that shouldn't
/// scroll, such as a status bar, on [`ctx.window_root()`](WindowCtx::window_root), and shrink the
/// scroll layer's frame to make room for them.
///
/// Up and Down scroll, a page at a time on round screens. Only Select takes a handler. Back pops
/// the window.
pub struct ScrollWindow<L: Layout + 'static> {
	// Its user data is a ScrollData<L>, which this frees instead
	window: ManuallyDrop<Window<L>>,
	_data:  PhantomData<Box<ScrollData<L>>>,
}

/// A [`ScrollWindow`]'s user data: a plain window's, then the scroll layer while it's loaded.
#[repr(C)]
struct ScrollData<L: Layout + 'static> {
	// read as a WindowData<L> through the window's user data: must stay the first field
	window: WindowData<L>,
	scroll: Option<ScrollLayer>,
}

impl<L: Layout> ScrollWindow<L> {
	/// Create a window with persistent data `data`. `None` if the firmware is out of memory.
	pub fn new(data: Option<L::Data>) -> Option<Self> {
		let handlers = sys::WindowHandlers {
			load:      Some(Self::__load),
			appear:    Some(Self::__appear),
			disappear: Some(Self::__disappear),
			unload:    Some(Self::__unload),
		};
		let data = ScrollData {
			window: WindowData::<L>::new(data),
			scroll: None,
		};
		// SAFETY: ScrollData starts with a WindowData<L>, and the handlers call Window<L>'s
		let ptr = unsafe { Window::<L>::create_with(handlers, data)? };
		Some(Self {
			// SAFETY: just created, with a WindowData<L> at the start of its user data
			window: ManuallyDrop::new(unsafe { Window::from_raw(ptr) }),
			_data:  PhantomData,
		})
	}

	/// The window's [`WindowCtx`], from outside its callbacks.
	pub fn as_ctx<'g>(&'g mut self, ui: &'g mut UiGuard) -> WindowCtx<'g, L::Data> {
		let scroll = unsafe { Self::scroll(self.window.ptr) };
		// SAFETY: as for the rest of the ctx: &mut self and the UiGuard borrow exclude every other
		// borrow of the scroll layer
		unsafe { self.window.as_ctx(ui).with_scroll(scroll) }
	}

	/// The window's layout. `None` while it is unloaded.
	pub fn layout<'g>(&'g mut self, ui: &'g mut UiGuard) -> Option<&'g mut L> {
		self.window.layout(ui)
	}

	/// Replace Select's click handler, subscribed to `kinds`. Takes effect immediately if the
	/// window is on screen.
	pub fn set_click_handler(
		&mut self,
		ui: &mut UiGuard,
		kinds: ClickKinds,
		mut handler: impl for<'n> FnMut(
			&'n mut L,
			Option<&'n mut L::Data>,
			&'n mut ScrollLayer,
			Click,
			&mut UiQueue<'n>,
		) + NoWindow
		+ 'static,
	) {
		let erased = ClickHandler::new(
			kinds,
			move |header: *mut Header, click, queue: &mut UiQueue<'_>| {
				let window = header.cast::<ScrollData<L>>();
				// SAFETY: the header is the start of this window's ScrollData<L>. The trampoline
				// borrows only the header's handlers, so these field borrows are the only others,
				// and its UiGuard excludes every other borrow into the window
				let (layout, data, scroll) = unsafe {
					(
						&mut (*window).window.layout,
						&mut (*window).window.data,
						&mut (*window).scroll,
					)
				};
				if let (Some(layout), Some(scroll)) = (layout, scroll) {
					handler(layout, data.as_mut(), scroll, click, queue);
				}
			},
		);
		// SAFETY: &mut self keeps the window alive
		unsafe { self.replace(ui, Some(erased)) };
	}

	/// Remove Select's click handler.
	pub fn clear_click_handler(&mut self, ui: &mut UiGuard) {
		// SAFETY: &mut self keeps the window alive
		unsafe { self.replace(ui, None) };
	}

	/// # Safety
	/// No borrow into the window may be live.
	unsafe fn replace(&mut self, ui: &mut UiGuard, handler: Option<ClickHandler>) {
		let window = self.window.ptr;
		// SAFETY: the window is live. The scroll layer's provider calls click_config. While the
		// window is unloaded there's no scroll layer, and the next load installs the provider
		unsafe {
			let scroll = (*Self::scroll_data(window)).scroll.as_ref().map(|s| s.ptr);
			click::replace_with(window, ui, Button::Select, handler, || {
				if let Some(scroll) = scroll {
					sys::scroll_layer_set_click_config_onto_window(scroll.as_ptr(), window);
				}
			});
		}
	}

	/// Leak this window and return a handle to it. Afterwards only the handle and the window's own
	/// callbacks can reach it, so keep the handle, e.g. in a [`StaticWindow`](super::StaticWindow).
	#[must_use]
	pub fn leak(self) -> AnyWindow<'static> {
		let mut this = ManuallyDrop::new(self);
		// SAFETY: this is never used again, and the Window it holds is leaked, so never freed
		unsafe { ManuallyDrop::take(&mut this.window) }.leak()
	}

	/// # Safety
	/// `window` must be a live `ScrollWindow<L>`'s. Borrows made through the pointer must not
	/// overlap another live borrow of the same field.
	unsafe fn scroll_data(window: *mut sys::Window) -> *mut ScrollData<L> {
		// SAFETY: user_data is the ScrollData<L> installed in new(), freed only in Drop
		unsafe { sys::window_get_user_data(window).cast() }
	}

	/// The scroll layer, null while unloaded.
	///
	/// # Safety
	/// As [`scroll_data`](Self::scroll_data).
	unsafe fn scroll(window: *mut sys::Window) -> *mut ScrollLayer {
		// SAFETY: forwarded
		unsafe { (*Self::scroll_data(window)).scroll.as_mut() }
			.map_or(core::ptr::null_mut(), |scroll| scroll)
	}

	/// Builds the scroll layer before the layout loads into it.
	unsafe extern "C" fn __load(window: *mut sys::Window) {
		// SAFETY: the firmware fires this on a live ScrollWindow<L>. Installing the click config
		// provider can run it, and no guard is held here, so the provider can take its own
		unsafe {
			let root = sys::window_get_root_layer(window);
			let mut scroll =
				ScrollLayer::new(rect(sys::layer_get_bounds(root))).pbl_expect(c"scroll layer");
			sys::layer_add_child(root, scroll.as_ptr());
			let _ = scroll.show_arrows(Colour::BLACK, Colour::WHITE);
			if cfg!(pbl_round) {
				scroll.set_paging(true);
			}
			let ptr = scroll.ptr.as_ptr();
			// The click trampolines read their context as the window
			sys::scroll_layer_set_context(ptr, window.cast());
			sys::scroll_layer_set_callbacks(ptr, sys::ScrollLayerCallbacks {
				click_config_provider:          Some(click::click_config),
				content_offset_changed_handler: None,
			});
			let scroll = (*Self::scroll_data(window)).scroll.insert(scroll);
			Window::<L>::load_with(window, &raw mut *scroll);
			sys::scroll_layer_set_click_config_onto_window(ptr, window);
		}
	}

	unsafe extern "C" fn __appear(window: *mut sys::Window) {
		// SAFETY: the firmware fires this on a live ScrollWindow<L>
		unsafe { Window::<L>::appear_with(window, Self::scroll(window)) }
	}

	unsafe extern "C" fn __disappear(window: *mut sys::Window) {
		// SAFETY: the firmware fires this on a live ScrollWindow<L>
		unsafe { Window::<L>::disappear_with(window, Self::scroll(window)) }
	}

	unsafe extern "C" fn __unload(window: *mut sys::Window) {
		// SAFETY: the firmware fires this on a live ScrollWindow<L>. The layout's layers go before
		// the scroll layer. The window's click config provider still points at the freed scroll
		// layer, but the window is off screen, so it can't run before the next load replaces it
		unsafe {
			Window::<L>::unload_with(window, Self::scroll(window));
			(*Self::scroll_data(window)).scroll = None;
		}
	}

	/// # Safety
	/// `data` must be the user data a `ScrollWindow<L>` was created with, not used again.
	unsafe fn free_data(data: *mut c_void) {
		drop(unsafe { Box::from_raw_in(data.cast::<ScrollData<L>>(), PebbleAlloc) });
	}
}

impl<L: Layout> Drop for ScrollWindow<L> {
	fn drop(&mut self) {
		// SAFETY: the window is live, its handle is going, and free_data frees the ScrollData new()
		// installed
		unsafe { drop_window(self.window.ptr, Self::free_data) };
	}
}

// SAFETY: leaks the window
unsafe impl<L: Layout> LeakWindow<L> for ScrollWindow<L> {
	fn leak(self) -> AnyWindow<'static> {
		Self::leak(self)
	}
}

impl<'w, L: Layout> From<&'w ScrollWindow<L>> for AnyWindow<'w> {
	fn from(window: &'w ScrollWindow<L>) -> Self {
		(&*window.window).into()
	}
}

impl<'w, L: Layout> TypedWindow<'w> for &'w ScrollWindow<L> {
	type Layout = L;
}
