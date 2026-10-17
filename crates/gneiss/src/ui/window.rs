//! Windows, the [`Layout`]s they load, and handles to them.
extern crate alloc;
use alloc::boxed::Box;
use core::{any::TypeId, cell::Cell, ffi::c_void, marker::PhantomData, mem, ptr::NonNull};

use crate::{
	SyncCell,
	alloc::PebbleAlloc,
	fail::PblUnwrap,
	sys,
	ui::{ClickHandlers, UiGuard, layer::RootLayer},
};

/// A window. One is displayed at a time: whichever is on top of the window stack.
///
/// `C` is its [`Layout`], which owns the layers and any other state that lives from load to
/// unload. The layout's [`Data`](Layout::Data) is optional persistent data that lives as long as
/// the window.
///
/// Dropping a `Window` takes it off the window stack and destroys it, with its layout and data.
/// While a [`UiGuard`] is held, such as inside a window callback, that waits until the guard is
/// released, so the window stays on screen until the callback returns, and its data drops then.
pub struct Window<C: Layout + 'static> {
	pub(super) ptr: *mut sys::Window,
	data:           PhantomData<Box<WindowData<C>>>,
}

/// A window's context inside its own callbacks: everything but the layout, which is lent alongside
/// it.
pub struct WindowCtx<'w, D = ()> {
	ptr:   *mut sys::Window,
	// Raw, not &mut: firmware calls made through the ctx can fire callbacks that read the data
	data:  *mut Option<D>,
	_data: PhantomData<&'w mut Option<D>>,
}

/// Shared access to a loaded window's layout and data, from [`AnyWindow::downcast`] or
/// [`StaticWindow::borrow`].
pub struct WindowRef<'r, C: Layout> {
	ptr: *mut sys::Window,
	_p:  PhantomData<(&'r C, &'r C::Data)>,
}

/// A type-erased handle to a window, for navigation and [`AnyWindow::downcast`].
#[derive(Clone, Copy)]
pub struct AnyWindow<'w> {
	pub(super) ptr: *mut sys::Window,
	_p:             PhantomData<&'w sys::Window>,
}

#[repr(C)]
pub(super) struct WindowData<C: Layout + 'static> {
	// read at offset 0 through type-erased pointers: must stay the first field
	pub(super) header:  Header,
	// Some while loaded
	pub(super) layout:  Option<C>,
	pub(super) data:    Option<C::Data>,
	// from UiQueue::push_with, taken by the next appear
	pub(super) pending: Option<C::Arg>,
}

/// The part of a window's user data that doesn't depend on its types. Gneiss does bookkeeping for
/// handlers and dropping here - few extra words at runtime save error prone and heavy codepaths
#[repr(C)]
pub(super) struct Header {
	// read at offset 0 through type-erased pointers: must stay the first field
	pub(super) tag:          TypeId,
	pub(super) clicks:       ClickHandlers,
	// A MenuWindow's menu, destroyed with the window
	pub(super) menu:         Option<NonNull<sys::MenuLayer>>,
	// Frees the WindowData: set when the handle is dropped. Windows that don't drop don't set
	// this field, and LTO sees that their `free` is dead code and drops it from the binary
	pub(super) free:         Option<unsafe fn(*mut c_void)>,
	// The next window in the `DROPPED` list
	pub(super) next_dropped: Option<NonNull<sys::Window>>,
}

/// A firmware window with `handlers` installed. `None` if the firmware is out of memory.
fn create(handlers: sys::WindowHandlers) -> Option<*mut sys::Window> {
	// SAFETY: plain firmware calls on the window just created
	unsafe {
		let raw = sys::window_create();
		if raw.is_null() {
			return None;
		}
		sys::window_set_window_handlers(raw, handlers);
		Some(raw)
	}
}

// Windows whose handles were dropped, waiting to be destroyed: linked through Header::next_dropped
static DROPPED: SyncCell<Option<NonNull<sys::Window>>> = SyncCell::new(None);
// Set while destroy_dropped runs, so callbacks it fires don't run it again
static DESTROYING: SyncCell<bool> = SyncCell::new(false);

/// Put the raw window into the `DROPPED` list after its handle is gone, and call
/// [`destroy_dropped`] if the ui is idle
///
/// # Safety
/// `window` must be a live window created by [`Window::new`], not used again, and `free` must free
/// its user data.
unsafe fn drop_window(window: *mut sys::Window, free: unsafe fn(*mut c_void)) {
	// SAFETY: the window is live, and its handle, the only other way to it, is gone
	unsafe {
		let header = header(window);
		(*header).free = Some(free);

		// Insert this window as the Head of DROPPED, shifting the existing list:
		// DROPPED(c) -> b -> a
		// d -> DROPPED(c) -> b -> a
		// DROPPED(d) -> c -> b -> a
		(*header).next_dropped = DROPPED.get();
		DROPPED.set(Some(NonNull::new_unchecked(window)));
	}

	if super::ui_idle() {
		destroy_dropped();
	}
}

/// Take every dropped window off the stack and destroy it.
///
/// Called only with no guard held or lent, so no window callback, queue drain or firmware frame
/// still refers to a dropped window.
pub(super) fn destroy_dropped() {
	if DESTROYING.replace(true) {
		return;
	}
	// Freeing a window's data can drop more windows, onto the list this is still taking from
	while let Some(window) = DROPPED.take() {
		let window = window.as_ptr();
		// SAFETY: a dropped window is live until here, and only reachable through the list. Taking
		// it off the stack fires its disappear and unload, which can take the guard
		unsafe {
			let header = header(window);
			DROPPED.set((*header).next_dropped);
			sys::window_stack_remove(window, false);
			if let Some(menu) = (*header).menu {
				sys::menu_layer_destroy(menu.as_ptr());
			}
			let free = (*header).free.unwrap_unchecked();
			let data = sys::window_get_user_data(window);
			#[cfg(feature = "trace-ui")]
			trace("destroy", window, "");
			sys::window_destroy(window);
			free(data);
		}
	}
	DESTROYING.set(false);
}

/// # Safety
/// `window` must be a live window created by [`Window::new`].
pub(super) unsafe fn header(window: *mut sys::Window) -> *mut Header {
	// SAFETY: user_data is the repr(C) WindowData installed in new(), which starts with its Header
	unsafe { sys::window_get_user_data(window).cast() }
}

/// Implemented for every type that does not contain a [`Window`], directly or behind any pointer.
///
/// A [`Layout`] is dropped on unload, which fires synchronously when its window is popped, so a
/// `Window` owned by a layout would be destroyed mid-navigation.
///
/// # Safety
/// A manual impl asserts the type never owns or holds a [`Window`].
pub unsafe auto trait NoWindow {}
impl<C: Layout> !NoWindow for Window<C> {}

/// A window's contents for one load to unload: its layers and any state with that lifetime.
///
/// # Reusing a layout with different data
///
/// A layout shared by windows with different data is generic over it, and names it as its
/// [`Data`](Self::Data). Rust requires a struct to use each of its type parameters in a field, so
/// a layout that doesn't store a `D` itself holds a zero-sized `PhantomData` marker instead. Write
/// it as `PhantomData<fn() -> D>`, not `PhantomData<D>`: the plain form counts as owning a `D`, so
/// data holding a [`Window`] would make the layout hold one too, and it would no longer be a
/// [`NoWindow`].
///
/// ```no_run
/// use core::marker::PhantomData;
/// use gneiss::ui::*;
///
/// trait Titled {
///     fn title(&self) -> &str;
/// }
///
/// struct TitleLayout<D> {
///     text:  TextLayer<HeapText>,
///     _data: PhantomData<fn() -> D>,
/// }
///
/// impl<D: Titled + 'static> Layout for TitleLayout<D> {
///     type Data = D;
///
///     fn load(ctx: &mut WindowCtx<'_, D>) -> Self {
///         let mut root = ctx.root();
///         let mut text = TextLayer::new(root.bounds()).expect("text layer");
///         if let Some(data) = ctx.data() {
///             text.set_text(data.title());
///         }
///         root.add_child(&mut text);
///         Self { text, _data: PhantomData }
///     }
/// }
///
/// struct Song { name: &'static str }
/// impl Titled for Song {
///     fn title(&self) -> &str { self.name }
/// }
///
/// static NOW_PLAYING: StaticWindow<TitleLayout<Song>> = StaticWindow::new();
/// ```
#[allow(unused, reason = "no-op defaults")]
pub trait Layout: NoWindow + Sized {
	/// The window's persistent data, which lives as long as the window and every callback can
	/// reach through its [`WindowCtx`].
	type Data: 'static = ();

	/// What [`UiQueue::push_with`](super::UiQueue::push_with) hands this window, through
	/// [`appear`](Self::appear).
	type Arg: 'static = ();

	/// Build the layout when the window loads.
	fn load(parent: &mut WindowCtx<'_, Self::Data>) -> Self;

	/// Called when the window comes on screen, after `load` if it just loaded. `arg` is what the
	/// latest [`UiQueue::push_with`](super::UiQueue::push_with) to this window passed.
	fn appear(&mut self, parent: &mut WindowCtx<'_, Self::Data>, arg: Option<Self::Arg>) {}
	/// Called when the window leaves the screen, but is still loaded. (stacked over)
	fn disappear(&mut self, parent: &mut WindowCtx<'_, Self::Data>) {}

	/// Called with the layout when the window unloads. It drops once this returns.
	///
	/// You aren't responsible for `Layer` specific cleanup in here (`Layer`'s `Drop` is), but it's
	/// a good time to save things to persistent storage or other clean-up behaviour
	fn unload(self, parent: &mut WindowCtx<'_, Self::Data>) {}
}

/// No layout: a window with persistent data `D` whose contents need nothing from load to unload,
/// such as a [`MenuWindow`](super::MenuWindow) showing only its menu.
pub struct NoLayout<D>(PhantomData<fn() -> D>);

impl<D: 'static> Layout for NoLayout<D> {
	type Data = D;

	fn load(_parent: &mut WindowCtx<'_, D>) -> Self {
		Self(PhantomData)
	}
}

impl<C: Layout> Window<C> {
	/// Create a window with persistent data `data`. `None` if the firmware is out of memory.
	pub fn new(data: Option<C::Data>) -> Option<Self> {
		let ptr = create(sys::WindowHandlers {
			load:      Some(Self::__load),
			appear:    Some(Self::__appear),
			disappear: Some(Self::__disappear),
			unload:    Some(Self::__unload),
		})?;
		let data = Box::new_in(
			WindowData::<C> {
				header: Header {
					tag:          TypeId::of::<WindowData<C>>(),
					clicks:       ClickHandlers::new(),
					menu:         None,
					free:         None,
					next_dropped: None,
				},
				layout: None,
				data,
				pending: None,
			},
			PebbleAlloc,
		);
		// SAFETY: ptr was just created
		unsafe { sys::window_set_user_data(ptr, Box::into_raw_with_allocator(data).0.cast()) };
		Some(Self {
			ptr,
			data: PhantomData,
		})
	}

	/// The window's [`WindowCtx`], from outside its callbacks.
	pub fn as_ctx<'g>(&'g mut self, _ui: &'g mut UiGuard) -> WindowCtx<'g, C::Data> {
		// SAFETY: &mut self excludes other borrows of this window; the UiGuard borrow excludes
		// any window callback or UiQueue drain for as long as the result lives
		unsafe { WindowCtx::new(self.ptr, &raw mut (*Self::window_data(self.ptr)).data) }
	}

	/// The window's layout. `None` while it is unloaded.
	pub fn layout<'g>(&'g mut self, _ui: &'g mut UiGuard) -> Option<&'g mut C> {
		// SAFETY: &mut self excludes other borrows of this window; the UiGuard borrow excludes
		// any window callback or UiQueue drain for as long as the result lives
		unsafe { &mut (*Self::window_data(self.ptr)).layout }.as_mut()
	}

	/// The whole `WindowData`, shared, for read-only access such as [`WindowRef`]'s. Anything that
	/// writes to the window goes through [`window_data`](Self::window_data) instead.
	///
	/// # Safety
	/// `window` must be a live window created by [`Window::new`] with this `C`, and nothing may
	/// write to its `WindowData` while the returned reference lives.
	pub(super) unsafe fn window_data_ref<'a>(window: *mut sys::Window) -> &'a WindowData<C> {
		// SAFETY: user_data is the WindowData<C> installed in new(), freed only in Drop
		unsafe { Self::window_data(window).as_ref().unwrap_unchecked() }
	}

	/// The window's `WindowData` as a raw pointer, for mutable access. Borrow only the fields
	/// needed through it (`&mut (*w).layout`, `&raw mut (*w).data`), never the whole struct: while
	/// a field is borrowed, a firmware call can run a callback, such as a menu's, that reaches the
	/// other fields through this same pointer.
	///
	/// # Safety
	/// `window` must be a live window created by [`Window::new`] with this `C`, and a borrow
	/// made through the pointer must not overlap another live borrow of the same field.
	pub(super) unsafe fn window_data(window: *mut sys::Window) -> *mut WindowData<C> {
		// SAFETY: user_data is the WindowData<C> installed in new(), freed only in Drop
		unsafe { sys::window_get_user_data(window).cast() }
	}

	/// Leak this window and return a handle to it. Afterwards only the handle and the window's own
	/// callbacks can reach it, so keep the handle, e.g. in a [`StaticWindow`].
	#[must_use]
	pub fn leak(self) -> AnyWindow<'static> {
		let ptr = self.ptr;
		// suppress drop, causing the allocation to become 'static
		mem::forget(self);
		AnyWindow {
			ptr,
			_p: PhantomData,
		}
	}
}

/// Log a window event, with the `trace-ui` feature: `ui <event> <window> <layout type>`.
macro_rules! trace_ui {
	(@ $event:literal, $window:expr, $layout:expr) => {
		#[cfg(feature = "trace-ui")]
		trace($event, $window, $layout);
	};
	($event:literal, $window:expr) => {
		trace_ui!(@ $event, $window, "")
	};
	($event:literal, $window:expr, $layout:ty) => {
		trace_ui!(@ $event, $window, core::any::type_name::<$layout>())
	};
}

#[cfg(feature = "trace-ui")]
fn trace(event: &str, window: *mut sys::Window, layout: &str) {
	// SAFETY: %.*s reads exactly each str's bytes, and %p prints the pointer without reading it
	unsafe {
		sys::app_log(
			crate::log::LogLevel::Info as u8,
			c"gneiss".as_ptr().cast(),
			0,
			c"ui %.*s %p %.*s".as_ptr(),
			event.len() as i32,
			event.as_ptr(),
			window,
			layout.len() as i32,
			layout.as_ptr(),
		);
	}
}

// Trampolines.
// SAFETY: the OS only fires handlers on a live window, and each borrows its fields
// once per call; UiGuard rules out reentrancy
impl<C: Layout> Window<C> {
	unsafe extern "C" fn __load(window: *mut sys::Window) {
		let _ui = UiGuard::enter_internal();
		trace_ui!("load", window, C);
		let (layout, data) = unsafe {
			let w = Self::window_data(window);
			(&mut (*w).layout, &raw mut (*w).data)
		};
		debug_assert!(layout.is_none());
		layout.get_or_insert_with(|| C::load(&mut unsafe { WindowCtx::new(window, data) }));
	}

	unsafe extern "C" fn __appear(window: *mut sys::Window) {
		let _ui = UiGuard::enter_internal();
		trace_ui!("appear", window, C);
		let (layout, data, pending) = unsafe {
			let w = Self::window_data(window);
			(&mut (*w).layout, &raw mut (*w).data, &mut (*w).pending)
		};
		if let Some(layout) = layout {
			layout.appear(&mut unsafe { WindowCtx::new(window, data) }, pending.take());
		}
	}

	unsafe extern "C" fn __disappear(window: *mut sys::Window) {
		let _ui = UiGuard::enter_internal();
		trace_ui!("disappear", window, C);
		let (layout, data) = unsafe {
			let w = Self::window_data(window);
			(&mut (*w).layout, &raw mut (*w).data)
		};
		if let Some(layout) = layout {
			layout.disappear(&mut unsafe { WindowCtx::new(window, data) });
		}
	}

	unsafe extern "C" fn __unload(window: *mut sys::Window) {
		let _ui = UiGuard::enter_internal();
		trace_ui!("unload", window, C);
		let (layout, data) = unsafe {
			let w = Self::window_data(window);
			(&mut (*w).layout, &raw mut (*w).data)
		};
		if let Some(layout) = layout.take() {
			layout.unload(&mut unsafe { WindowCtx::new(window, data) });
		}
	}
}

impl<C: Layout> Window<C> {
	/// # Safety
	/// `data` must be the user data a `Window<C>` was created with, not used again.
	unsafe fn free_data(data: *mut c_void) {
		drop(unsafe { Box::from_raw_in(data.cast::<WindowData<C>>(), PebbleAlloc) });
	}
}

impl<C: Layout> Drop for Window<C> {
	fn drop(&mut self) {
		trace_ui!("drop", self.ptr, C);
		// SAFETY: self.ptr is live, its handle is going, and free_data frees the WindowData<C>
		// new() installed
		unsafe { drop_window(self.ptr, Self::free_data) };
	}
}

impl<D> WindowCtx<'_, D> {
	/// # Safety
	/// `data` must be the data field of `ptr`'s `WindowData`, valid for the ctx's lifetime, with no
	/// reference to it live meanwhile but those this ctx hands out.
	pub(super) unsafe fn new(ptr: *mut sys::Window, data: *mut Option<D>) -> Self {
		Self {
			ptr,
			data,
			_data: PhantomData,
		}
	}

	/// The window's persistent data, if it has any.
	pub fn data(&mut self) -> Option<&mut D> {
		// SAFETY: valid for 'w, per new(); &mut self makes this the only reference handed out
		unsafe { (*self.data).as_mut() }
	}

	/// The raw window.
	pub fn ptr(&self) -> *mut sys::Window {
		self.ptr
	}

	/// The window's root layer, to add layers to.
	pub fn root(&mut self) -> RootLayer {
		RootLayer::from_ctx(self)
	}
}

impl<'r, C: Layout + 'static> WindowRef<'r, C> {
	/// The window's layout.
	pub fn layout(&self) -> &'r C {
		// SAFETY: only built for a loaded window of these types, under a shared UiGuard borrow that
		// excludes unloading and any &mut into its WindowData for 'r
		unsafe {
			Window::<C>::window_data_ref(self.ptr)
				.layout
				.as_ref()
				.unwrap_unchecked()
		}
	}

	/// The window's persistent data, if it has any.
	pub fn data(&self) -> Option<&'r C::Data> {
		// SAFETY: as for layout
		unsafe { Window::<C>::window_data_ref(self.ptr) }
			.data
			.as_ref()
	}

	/// The raw window.
	pub fn ptr(&self) -> *mut sys::Window {
		self.ptr
	}
}

impl<'w> AnyWindow<'w> {
	/// The raw window.
	pub fn as_ptr(self) -> *mut sys::Window {
		self.ptr
	}

	/// Shared access to the window's layout and data. `None` unless it is a loaded `Window<C>`.
	pub fn downcast<'g, C: Layout + 'static>(self, _ui: &'g UiGuard) -> Option<WindowRef<'g, C>>
	where
		'w: 'g,
	{
		// SAFETY: every AnyWindow comes from a gneiss Window, whose user data is a repr(C)
		// WindowData with its TypeId at offset 0, alive for 'w. The shared UiGuard borrow excludes
		// any &mut into it
		let tag = unsafe {
			sys::window_get_user_data(self.ptr)
				.cast::<TypeId>()
				.as_ref()
		}?;
		if *tag != TypeId::of::<WindowData<C>>() {
			return None;
		}
		// SAFETY: the tag matched, so this is a WindowData<C>
		unsafe { Window::<C>::window_data_ref(self.ptr) }
			.layout
			.as_ref()?;
		Some(WindowRef {
			ptr: self.ptr,
			_p:  PhantomData,
		})
	}
}

impl<'w, C: Layout> From<&'w Window<C>> for AnyWindow<'w> {
	fn from(window: &'w Window<C>) -> Self {
		Self {
			ptr: window.ptr,
			_p:  PhantomData,
		}
	}
}

/// A set-once global slot for a leaked [`Window`], pushable from anywhere as `&SLOT`.
pub struct StaticWindow<C: Layout + 'static> {
	inner: SyncCell<Option<AnyWindow<'static>>>,
	// This can be used to obtain references to C and its data
	_win:  PhantomData<fn() -> C>,
}

impl<C: Layout> StaticWindow<C> {
	/// An empty slot.
	pub const fn new() -> Self {
		Self {
			inner: SyncCell::new(None),
			_win:  PhantomData,
		}
	}

	/// Leak `window` into this slot. Configure it first: it can't be reached mutably afterwards.
	///
	/// # Panics
	/// If the slot is already set.
	#[track_caller]
	pub fn set(&self, window: impl LeakWindow<C>) {
		slot_set(&self.inner, window.leak());
	}

	/// The slot's window, to navigate to.
	///
	/// # Panics
	/// If the slot has not been set.
	#[track_caller]
	pub fn get(&self) -> AnyWindow<'static> {
		slot_get(&self.inner)
	}

	/// Shared access to the window's layout and data. `None` while it is unloaded.
	///
	/// # Panics
	/// If the slot has not been set.
	#[track_caller]
	pub fn borrow<'g>(&self, ui: &'g UiGuard) -> Option<WindowRef<'g, C>> {
		self.get().downcast(ui)
	}

	/// Mutable access to the window from outside its callbacks, as its callbacks get it: the layout
	/// (`None` while the window is unloaded) and its [`WindowCtx`].
	///
	/// # Panics
	/// If the slot has not been set.
	#[track_caller]
	pub fn with<R>(
		&self,
		_ui: &mut UiGuard,
		f: impl FnOnce(Option<&mut C>, &mut WindowCtx<'_, C::Data>) -> R,
	) -> R {
		let ptr = self.get().ptr;
		// SAFETY: the slot holds a leaked Window<C>, never freed. Holding &mut UiGuard means no
		// window callback is running and no other borrow into any window is live
		let (layout, mut ctx) = unsafe {
			let w = Window::<C>::window_data(ptr);
			(&mut (*w).layout, WindowCtx::new(ptr, &raw mut (*w).data))
		};
		f(layout.as_mut(), &mut ctx)
	}
}

/// A window that [`StaticWindow::set`] can leak: a [`Window`], a
/// [`MenuWindow`](super::MenuWindow), or a wrapper around either.
///
/// # Safety
/// `leak` must return a window created by [`Window::new`] with this `C`, which is never freed.
/// [`StaticWindow`] reaches its layout and data as those types without checking.
pub unsafe trait LeakWindow<C: Layout> {
	/// Leak the window and return a handle to it.
	#[must_use]
	fn leak(self) -> AnyWindow<'static>;
}

// SAFETY: Window::leak forgets the window, so it's never freed
unsafe impl<C: Layout> LeakWindow<C> for Window<C> {
	fn leak(self) -> AnyWindow<'static> {
		Window::leak(self)
	}
}

#[track_caller]
fn slot_set(slot: &Cell<Option<AnyWindow<'static>>>, window: AnyWindow<'static>) {
	crate::pbl_assert!(slot.get().is_none(), "StaticWindow set twice");
	slot.set(Some(window));
}

#[track_caller]
fn slot_get(slot: &Cell<Option<AnyWindow<'static>>>) -> AnyWindow<'static> {
	slot.get().pbl_expect(c"StaticWindow used before set")
}

impl<C: Layout> Default for StaticWindow<C> {
	fn default() -> Self {
		Self::new()
	}
}

/// A window whose types are known, so [`UiQueue::push_with`](super::UiQueue::push_with) can hand
/// it an argument.
pub trait TypedWindow<'w>: Into<AnyWindow<'w>> + Copy {
	/// The window's [`Layout`].
	type Layout: Layout + 'static;
}

impl<'w, C: Layout> TypedWindow<'w> for &'w Window<C> {
	type Layout = C;
}

impl<'w, C: Layout> TypedWindow<'w> for &'w StaticWindow<C> {
	type Layout = C;
}

impl<C: Layout> From<&StaticWindow<C>> for AnyWindow<'_> {
	fn from(slot: &StaticWindow<C>) -> Self {
		// Not get(): navigation calls this from inside gneiss, so the location would be gneiss's
		crate::fail::gneiss_expect!(slot.inner.get(), "StaticWindow used before set")
	}
}
