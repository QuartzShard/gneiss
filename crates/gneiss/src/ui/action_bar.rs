//! [`ActionWindow`], a window with an action bar: icons beside the Up, Select and Down buttons.
extern crate alloc;
use alloc::boxed::Box;
use core::{borrow::Borrow, ffi::c_void, marker::PhantomData, mem::ManuallyDrop, ptr::NonNull};

use crate::{
	alloc::PebbleAlloc,
	resource::image::GBitMap,
	sys,
	ui::{
		AnyWindow, Button, Click, ClickKinds, Layout, NoWindow, TypedWindow, UiGuard, UiQueue,
		Window, WindowCtx,
		click::{self, ClickHandler},
		graphics::Colour,
		window::{Header, LeakWindow, WindowData, drop_window},
	},
};

/// The action bar's width on this platform.
pub const ACTION_BAR_WIDTH: u32 = sys::ACTION_BAR_WIDTH;

/// A button with a place on the action bar.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum ActionButton {
	Up     = sys::ButtonId_BUTTON_ID_UP,
	Select = sys::ButtonId_BUTTON_ID_SELECT,
	Down   = sys::ButtonId_BUTTON_ID_DOWN,
}

impl ActionButton {
	const fn index(self) -> usize {
		self as usize - 1
	}
}

impl From<ActionButton> for Button {
	fn from(button: ActionButton) -> Self {
		match button {
			ActionButton::Up => Self::Up,
			ActionButton::Select => Self::Select,
			ActionButton::Down => Self::Down,
		}
	}
}

/// How an icon moves while its button is held.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum PressAnimation {
	None      = sys::ActionBarLayerIconPressAnimation_ActionBarLayerIconPressAnimationNone,
	MoveLeft  = sys::ActionBarLayerIconPressAnimation_ActionBarLayerIconPressAnimationMoveLeft,
	MoveUp    = sys::ActionBarLayerIconPressAnimation_ActionBarLayerIconPressAnimationMoveUp,
	MoveRight = sys::ActionBarLayerIconPressAnimation_ActionBarLayerIconPressAnimationMoveRight,
	MoveDown  = sys::ActionBarLayerIconPressAnimation_ActionBarLayerIconPressAnimationMoveDown,
}

bitflags::bitflags! {
	/// The [`ClickKinds`] an [`ActionWindow`] handler can subscribe to. Raw presses are left out:
	/// the action bar uses them to highlight a pressed button's icon.
	pub struct ActionClickKinds: u8 {
		const SINGLE       = ClickKinds::SINGLE.bits();
		const MULTI        = ClickKinds::MULTI.bits();
		const LONG         = ClickKinds::LONG.bits();
		const LONG_RELEASE = ClickKinds::LONG_RELEASE.bits();
		/// As [`ClickKinds::REPEATING`].
		const REPEATING    = ClickKinds::REPEATING.bits();
	}
}

impl From<ActionClickKinds> for ClickKinds {
	fn from(kinds: ActionClickKinds) -> Self {
		Self::from_bits_retain(kinds.bits())
	}
}

/// An [`ActionWindow`]'s bar, which owns the icons it shows. `B` is a [`GBitMap`], or
/// `&'static GBitMap` for one that lives as long as the app, or an `Rc<GBitMap>` to share one
/// between windows that come and go. [`replace_icon`](Self::replace_icon) and
/// [`clear_icon`](Self::clear_icon) hand the old icon back, to swap between icons without loading
/// them again.
pub struct ActionBar<B: Borrow<GBitMap> = GBitMap> {
	ptr:   NonNull<sys::ActionBarLayer>,
	icons: [Option<B>; 3],
}

impl<B: Borrow<GBitMap>> ActionBar<B> {
	/// Show `icon` beside `button`, dropping the old one.
	pub fn set_icon(&mut self, button: ActionButton, icon: B) -> &mut Self {
		self.replace(button, icon, false);
		self
	}

	/// [`set_icon`](Self::set_icon), sliding the new icon in.
	pub fn set_icon_animated(&mut self, button: ActionButton, icon: B) -> &mut Self {
		self.replace(button, icon, true);
		self
	}

	/// Show `icon` beside `button`. Returns the old icon, if it had one, which the bar no longer
	/// points at.
	pub fn replace_icon(&mut self, button: ActionButton, icon: B) -> Option<B> {
		self.replace(button, icon, false)
	}

	/// [`replace_icon`](Self::replace_icon), sliding the new icon in.
	pub fn replace_icon_animated(&mut self, button: ActionButton, icon: B) -> Option<B> {
		self.replace(button, icon, true)
	}

	fn replace(&mut self, button: ActionButton, icon: B, animated: bool) -> Option<B> {
		let slot = self.icons.get_mut(button.index())?;
		// SAFETY: the firmware bitmap lives as long as the GBitMap, which self.icons keeps alive;
		// moving the handle doesn't move it
		unsafe {
			sys::action_bar_layer_set_icon_animated(
				self.ptr.as_ptr(),
				button as u8,
				icon.borrow().as_ptr(),
				animated,
			);
		}
		slot.replace(icon)
	}

	/// Remove `button`'s icon. Returns it, if it had one.
	pub fn clear_icon(&mut self, button: ActionButton) -> Option<B> {
		unsafe { sys::action_bar_layer_clear_icon(self.ptr.as_ptr(), button as u8) };
		self.icons.get_mut(button.index())?.take()
	}

	/// `button`'s icon, if it has one.
	pub fn icon(&self, button: ActionButton) -> Option<&GBitMap> {
		self.icons.get(button.index())?.as_ref().map(Borrow::borrow)
	}

	/// Set how `button`'s icon moves while it's held. [`PressAnimation::MoveLeft`] until set.
	pub fn set_press_animation(
		&mut self,
		button: ActionButton,
		animation: PressAnimation,
	) -> &mut Self {
		unsafe {
			sys::action_bar_layer_set_icon_press_animation(
				self.ptr.as_ptr(),
				button as u8,
				animation as u8,
			);
		}
		self
	}

	/// Set the colour behind the icons. Black until set.
	pub fn set_background_colour(&mut self, colour: Colour) -> &mut Self {
		unsafe { sys::action_bar_layer_set_background_color(self.ptr.as_ptr(), colour.into()) };
		self
	}
}

impl<B: Borrow<GBitMap>> Drop for ActionBar<B> {
	fn drop(&mut self) {
		// SAFETY: destroyed before the icons drop, so the firmware never reads a freed one. The
		// window has unloaded by now, which took the bar off it
		unsafe { sys::action_bar_layer_destroy(self.ptr.as_ptr()) };
	}
}

/// A window with an [`ActionBar`] down its right edge, beside the buttons. The bar lives as long as
/// the window, and draws over the layout: lay the layout out in the [`ACTION_BAR_WIDTH`] it leaves.
///
/// The bar owns the buttons' clicks, so handlers are set here rather than on a [`Window`], and get
/// the bar to change its icons. Back pops the window until it's given a handler.
pub struct ActionWindow<L: Layout + 'static, B: Borrow<GBitMap> + 'static = GBitMap> {
	// Its user data is an ActionData<L, B>, which this frees instead
	window: ManuallyDrop<Window<L>>,
	_data:  PhantomData<Box<ActionData<L, B>>>,
}

/// An [`ActionWindow`]'s user data: a plain window's, then the bar.
#[repr(C)]
struct ActionData<L: Layout + 'static, B: Borrow<GBitMap>> {
	// read as a WindowData<L> through the window's user data: must stay the first field
	window: WindowData<L>,
	bar:    ActionBar<B>,
}

impl<L: Layout, B: Borrow<GBitMap> + 'static> ActionWindow<L, B> {
	/// Create a window with an empty action bar, and persistent data `data`. `None` if the
	/// firmware is out of memory.
	pub fn new(data: Option<L::Data>) -> Option<Self> {
		// SAFETY: no preconditions; NULL on OOM
		let bar = ActionBar::<B> {
			ptr:   NonNull::new(unsafe { sys::action_bar_layer_create() })?,
			icons: [None, None, None],
		};
		let layer = bar.ptr.as_ptr();
		let handlers = sys::WindowHandlers {
			load: Some(Self::__load),
			unload: Some(Self::__unload),
			..Window::<L>::HANDLERS
		};
		let data = ActionData {
			window: WindowData::<L>::new(data),
			bar,
		};
		// SAFETY: ActionData starts with a WindowData<L>, and the handlers call Window<L>'s
		let ptr = unsafe { Window::<L>::create_with(handlers, data)? };
		// SAFETY: plain firmware calls on the bar just created
		unsafe {
			// The click trampolines read their context as the window
			sys::action_bar_layer_set_context(layer, ptr.cast());
			// Stored until the bar is on the window, which installs it
			sys::action_bar_layer_set_click_config_provider(layer, Some(click::click_config));
		}
		Some(Self {
			// SAFETY: just created, with a WindowData<L> at the start of its user data
			window: ManuallyDrop::new(unsafe { Window::from_raw(ptr) }),
			_data:  PhantomData,
		})
	}

	/// The window's [`WindowCtx`], from outside its callbacks.
	pub fn as_ctx<'g>(&'g mut self, ui: &'g mut UiGuard) -> WindowCtx<'g, L::Data> {
		self.window.as_ctx(ui)
	}

	/// The window's layout. `None` while it is unloaded.
	pub fn layout<'g>(&'g mut self, ui: &'g mut UiGuard) -> Option<&'g mut L> {
		self.window.layout(ui)
	}

	/// The window's action bar, from outside its callbacks.
	pub fn action_bar<'g>(&'g mut self, _ui: &'g mut UiGuard) -> &'g mut ActionBar<B> {
		// SAFETY: &mut self excludes other borrows of this window; the UiGuard borrow excludes any
		// click handler, the only other place that reaches the bar
		unsafe { &mut (*Self::action_data(self.window.ptr)).bar }
	}

	/// Replace `button`'s click handler, subscribed to `kinds`. Takes effect immediately if the
	/// window is on screen.
	pub fn set_click_handler(
		&mut self,
		ui: &mut UiGuard,
		button: Button,
		kinds: ActionClickKinds,
		mut handler: impl for<'n> FnMut(
			&'n mut L,
			Option<&'n mut L::Data>,
			&'n mut ActionBar<B>,
			Click,
			&mut UiQueue<'n>,
		) + NoWindow
		+ 'static,
	) {
		let erased = ClickHandler::new(
			kinds.into(),
			move |header: *mut Header, click, queue: &mut UiQueue<'_>| {
				let action = header.cast::<ActionData<L, B>>();
				// SAFETY: the header is the start of this window's ActionData<L, B>. The trampoline
				// borrows only the header's handlers, so these field borrows are the only others,
				// and its UiGuard excludes every other borrow into the window
				let (layout, data, bar) = unsafe {
					(
						&mut (*action).window.layout,
						&mut (*action).window.data,
						&mut (*action).bar,
					)
				};
				if let Some(layout) = layout {
					handler(layout, data.as_mut(), bar, click, queue);
				}
			},
		);
		// SAFETY: &mut self keeps the window alive
		unsafe { self.replace(ui, button, Some(erased)) };
	}

	/// Remove `button`'s click handler.
	pub fn clear_click_handler(&mut self, ui: &mut UiGuard, button: Button) {
		// SAFETY: &mut self keeps the window alive
		unsafe { self.replace(ui, button, None) };
	}

	/// # Safety
	/// No borrow into the window may be live.
	unsafe fn replace(&mut self, ui: &mut UiGuard, button: Button, handler: Option<ClickHandler>) {
		let window = self.window.ptr;
		// SAFETY: the window is live. The bar's own provider calls click_config, so it's installed
		// through the bar, which holds it until it's on the window
		unsafe {
			let layer = (*Self::action_data(window)).bar.ptr.as_ptr();
			click::replace_with(window, ui, button, handler, || {
				sys::action_bar_layer_set_click_config_provider(layer, Some(click::click_config));
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
	/// `window` must be a live `ActionWindow<L, B>`'s. Borrows made through the pointer must not
	/// overlap another live borrow of the same field.
	unsafe fn action_data(window: *mut sys::Window) -> *mut ActionData<L, B> {
		// SAFETY: user_data is the ActionData<L, B> installed in new(), freed only in Drop
		unsafe { sys::window_get_user_data(window).cast() }
	}

	/// Puts the bar on the window after its layout loads, so the bar draws on top.
	unsafe extern "C" fn __load(window: *mut sys::Window) {
		// SAFETY: the firmware fires this on a live ActionWindow<L, B>. Attaching the bar can run
		// the click config provider, and no guard is held here, so the provider can take its own
		unsafe {
			Window::<L>::__load(window);
			let layer = (*Self::action_data(window)).bar.ptr.as_ptr();
			sys::action_bar_layer_add_to_window(layer, window);
		}
	}

	unsafe extern "C" fn __unload(window: *mut sys::Window) {
		// SAFETY: the firmware fires this on a live ActionWindow<L, B>
		unsafe {
			Window::<L>::__unload(window);
			let layer = (*Self::action_data(window)).bar.ptr.as_ptr();
			sys::action_bar_layer_remove_from_window(layer);
		}
	}

	/// # Safety
	/// `data` must be the user data an `ActionWindow<L, B>` was created with, not used again.
	unsafe fn free_data(data: *mut c_void) {
		drop(unsafe { Box::from_raw_in(data.cast::<ActionData<L, B>>(), PebbleAlloc) });
	}
}

impl<L: Layout, B: Borrow<GBitMap> + 'static> Drop for ActionWindow<L, B> {
	fn drop(&mut self) {
		// SAFETY: the window is live, its handle is going, and free_data frees the ActionData new()
		// installed
		unsafe { drop_window(self.window.ptr, Self::free_data) };
	}
}

// SAFETY: leaks the window
unsafe impl<L: Layout, B: Borrow<GBitMap> + 'static> LeakWindow<L> for ActionWindow<L, B> {
	fn leak(self) -> AnyWindow<'static> {
		Self::leak(self)
	}
}

impl<'w, L: Layout, B: Borrow<GBitMap> + 'static> From<&'w ActionWindow<L, B>> for AnyWindow<'w> {
	fn from(window: &'w ActionWindow<L, B>) -> Self {
		(&*window.window).into()
	}
}

impl<'w, L: Layout, B: Borrow<GBitMap> + 'static> TypedWindow<'w> for &'w ActionWindow<L, B> {
	type Layout = L;
}
