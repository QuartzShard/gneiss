//! Windows, the [`Layout`]s they load, and handles to them.
extern crate alloc;
use alloc::boxed::Box;
use core::{any::TypeId, cell::Cell, marker::PhantomData, mem};

use crate::{
	alloc::PebbleAlloc,
	cell::SyncCell,
	sys,
	ui::{ClickHandlers, UiGuard, layer::RootLayer},
};

/// A window. One is displayed at a time: whichever is on top of the window stack.
///
/// `C` is its [`Layout`], which owns the layers and any other state that lives from load to
/// unload. `D` is optional persistent data that lives as long as the window.
pub struct Window<C: Layout<D> + 'static, D: 'static = ()> {
	pub(super) ptr: *mut sys::Window,
	data:           PhantomData<Box<WindowData<C, D>>>,
}

/// A window's context inside its own callbacks: everything but the layout, which is lent alongside
/// it.
pub struct WindowCtx<'w, C, D = ()> {
	ptr:     *mut sys::Window,
	data:    &'w mut Option<D>,
	_layout: PhantomData<fn() -> C>,
}

/// Shared access to a loaded window's layout and data, from [`AnyWindow::downcast`] or
/// [`StaticWindow::borrow`].
pub struct WindowRef<'r, C, D = ()> {
	ptr: *mut sys::Window,
	_p:  PhantomData<(&'r C, &'r D)>,
}

/// A type-erased handle to a window, for navigation and [`AnyWindow::downcast`].
#[derive(Clone, Copy)]
pub struct AnyWindow<'w> {
	pub(super) ptr: *mut sys::Window,
	_p:             PhantomData<&'w sys::Window>,
}

#[repr(C)]
pub(super) struct WindowData<C: Layout<D> + 'static, D: 'static = ()> {
	// read at offset 0 through type-erased pointers: must stay the first field
	pub(super) header:  Header,
	// Some while loaded
	pub(super) layout:  Option<C>,
	pub(super) data:    Option<D>,
	// from Nav::push_with, taken by the next appear
	pub(super) pending: Option<C::Arg>,
}

/// The part of a window's user data that doesn't depend on its types.
#[repr(C)]
pub(super) struct Header {
	// read at offset 0 through type-erased pointers: must stay the first field
	pub(super) tag:    TypeId,
	pub(super) clicks: ClickHandlers,
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

/// Take `window` off the stack and destroy it, returning its user data.
///
/// # Safety
/// `window` must be a live window, not used again.
///
/// # Panics
/// If the window is loaded.
unsafe fn destroy(window: *mut sys::Window) -> *mut core::ffi::c_void {
	// SAFETY: window is live until window_destroy, as the caller guarantees
	unsafe {
		let user_data = sys::window_get_user_data(window);
		assert!(
			!sys::window_is_loaded(window),
			"Currently invalid to drop a loaded window"
		);
		sys::window_stack_remove(window, false);
		sys::window_destroy(window);
		user_data
	}
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

impl<C: Layout<D>, D> !NoWindow for Window<C, D> {}

/// A window's contents for one load to unload: its layers and any state with that lifetime.
#[allow(unused, reason = "warning that no-op default don't use args is noise")]
pub trait Layout<D = ()>: NoWindow + Sized {
	/// What [`Nav::push_with`](super::Nav::push_with) hands this window, through
	/// [`appear`](Self::appear).
	type Arg: 'static = ();

	/// Build the layout when the window loads.
	fn load(parent: &mut WindowCtx<'_, Self, D>) -> Self;

	/// Called when the window comes on screen, after `load` if it just loaded. `arg` is what the
	/// latest [`Nav::push_with`](super::Nav::push_with) to this window passed, if it hasn't been
	/// handed over yet.
	fn appear(&mut self, parent: &mut WindowCtx<'_, Self, D>, arg: Option<Self::Arg>) {}
	/// Called when the window leaves the screen.
	fn disappear(&mut self, parent: &mut WindowCtx<'_, Self, D>) {}

	/// Called with the layout when the window unloads. It drops once this returns.
	fn unload(self, parent: &mut WindowCtx<'_, Self, D>) {}
}

impl<C: Layout<D>, D> Window<C, D> {
	/// Create a window with persistent data `data`. `None` if the firmware is out of memory.
	pub fn new(data: Option<D>) -> Option<Self> {
		let ptr = create(sys::WindowHandlers {
			load:      Some(Self::__load),
			appear:    Some(Self::__appear),
			disappear: Some(Self::__disappear),
			unload:    Some(Self::__unload),
		})?;
		let data = Box::new_in(
			WindowData::<C, D> {
				header: Header {
					tag:    TypeId::of::<WindowData<C, D>>(),
					clicks: ClickHandlers::new(),
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
	pub fn as_ctx<'g>(&'g mut self, _ui: &'g mut UiGuard) -> WindowCtx<'g, C, D> {
		// SAFETY: &mut self excludes other borrows of this window; the UiGuard borrow excludes
		// any window callback or Nav drain for as long as the result lives
		WindowCtx::new(self.ptr, unsafe { &mut Self::window_data(self.ptr).data })
	}

	/// The window's layout. `None` while it is unloaded.
	pub fn layout<'g>(&'g mut self, _ui: &'g mut UiGuard) -> Option<&'g mut C> {
		// SAFETY: &mut self excludes other borrows of this window; the UiGuard borrow excludes
		// any window callback or Nav drain for as long as the result lives
		unsafe { Self::window_data(self.ptr) }.layout.as_mut()
	}

	/// # Safety
	/// `window` must be a live window created by [`Window::new`] with these `C`/`D`, and no mutable
	/// reference into its `WindowData` may be live alongside the returned shared one.
	pub(super) unsafe fn window_data_ref<'a>(window: *mut sys::Window) -> &'a WindowData<C, D> {
		// SAFETY: user_data is the WindowData<C, D> installed in new(), freed only in Drop
		unsafe {
			sys::window_get_user_data(window)
				.cast::<WindowData<C, D>>()
				.as_ref()
				.unwrap_unchecked()
		}
	}

	/// # Safety
	/// `window` must be a live window created by [`Window::new`] with these `C`/`D`, and the
	/// returned borrow must be the only live reference into its `WindowData`.
	pub(super) unsafe fn window_data<'a>(window: *mut sys::Window) -> &'a mut WindowData<C, D> {
		// SAFETY: user_data is the WindowData<C, D> installed in new(), freed only in Drop
		unsafe {
			sys::window_get_user_data(window)
				.cast::<WindowData<C, D>>()
				.as_mut()
				.unwrap_unchecked()
		}
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

// Trampolines. SAFETY: the OS only fires handlers on a live window, and each derives its &mut
// once per call; UiGuard rules out reentrancy
impl<C: Layout<D>, D> Window<C, D> {
	unsafe extern "C" fn __load(window: *mut sys::Window) {
		let _ui = UiGuard::enter();
		let WindowData { layout, data, .. } = unsafe { Self::window_data(window) };
		debug_assert!(layout.is_none());
		layout.get_or_insert_with(|| C::load(&mut WindowCtx::new(window, data)));
	}

	unsafe extern "C" fn __appear(window: *mut sys::Window) {
		let _ui = UiGuard::enter();
		let WindowData {
			layout,
			data,
			pending,
			..
		} = unsafe { Self::window_data(window) };
		if let Some(layout) = layout {
			layout.appear(&mut WindowCtx::new(window, data), pending.take());
		}
	}

	unsafe extern "C" fn __disappear(window: *mut sys::Window) {
		let _ui = UiGuard::enter();
		let WindowData { layout, data, .. } = unsafe { Self::window_data(window) };
		if let Some(layout) = layout {
			layout.disappear(&mut WindowCtx::new(window, data));
		}
	}

	unsafe extern "C" fn __unload(window: *mut sys::Window) {
		let _ui = UiGuard::enter();
		let WindowData { layout, data, .. } = unsafe { Self::window_data(window) };
		if let Some(layout) = layout.take() {
			layout.unload(&mut WindowCtx::new(window, data));
		}
	}
}

impl<C: Layout<D>, D> Drop for Window<C, D> {
	fn drop(&mut self) {
		// SAFETY: self.ptr is live until here and dropped with it; destroy hands back the user data
		// new() installed, re-assembled so it drops too
		unsafe {
			let data_ptr = destroy(self.ptr).cast::<WindowData<C, D>>();
			let _ = Box::from_raw_in(data_ptr, PebbleAlloc);
		}
	}
}

impl<'w, C, D> WindowCtx<'w, C, D> {
	pub(super) fn new(ptr: *mut sys::Window, data: &'w mut Option<D>) -> Self {
		Self {
			ptr,
			data,
			_layout: PhantomData,
		}
	}

	/// The window's persistent data, if it has any.
	pub fn data(&mut self) -> Option<&mut D> {
		self.data.as_mut()
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

impl<'r, C: Layout<D> + 'static, D: 'static> WindowRef<'r, C, D> {
	/// The window's layout.
	pub fn layout(&self) -> &'r C {
		// SAFETY: only built for a loaded window of these types, under a shared UiGuard borrow that
		// excludes unloading and any &mut into its WindowData for 'r
		unsafe {
			Window::<C, D>::window_data_ref(self.ptr)
				.layout
				.as_ref()
				.unwrap_unchecked()
		}
	}

	/// The window's persistent data, if it has any.
	pub fn data(&self) -> Option<&'r D> {
		// SAFETY: as for layout
		unsafe { Window::<C, D>::window_data_ref(self.ptr) }
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

	/// Shared access to the window's layout and data. `None` unless it is a loaded
	/// `Window<C, D>`.
	pub fn downcast<'g, C: Layout<D> + 'static, D: 'static>(
		self,
		_ui: &'g UiGuard,
	) -> Option<WindowRef<'g, C, D>>
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
		if *tag != TypeId::of::<WindowData<C, D>>() {
			return None;
		}
		// SAFETY: the tag matched, so this is a WindowData<C, D>
		unsafe { Window::<C, D>::window_data_ref(self.ptr) }
			.layout
			.as_ref()?;
		Some(WindowRef {
			ptr: self.ptr,
			_p:  PhantomData,
		})
	}
}

impl<'w, C: Layout<D>, D> From<&'w Window<C, D>> for AnyWindow<'w> {
	fn from(window: &'w Window<C, D>) -> Self {
		Self {
			ptr: window.ptr,
			_p:  PhantomData,
		}
	}
}

/// A set-once global slot for a leaked [`Window`], pushable from anywhere as `&SLOT`.
pub struct StaticWindow<C: Layout<D> + 'static, D: 'static = ()> {
	inner: SyncCell<Option<AnyWindow<'static>>>,
	// This can be used to obtain references to C and D
	_win:  PhantomData<fn() -> (C, D)>,
}

impl<C: Layout<D>, D> StaticWindow<C, D> {
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
	pub fn set(&self, window: Window<C, D>) {
		slot_set(&self.inner, window.leak());
	}

	/// The slot's window, to navigate to.
	///
	/// # Panics
	/// If the slot has not been set.
	pub fn get(&self) -> AnyWindow<'static> {
		slot_get(&self.inner)
	}

	/// Shared access to the window's layout and data. `None` while it is unloaded.
	///
	/// # Panics
	/// If the slot has not been set.
	pub fn borrow<'g>(&self, ui: &'g UiGuard) -> Option<WindowRef<'g, C, D>> {
		self.get().downcast(ui)
	}

	/// Mutable access to the window from outside its callbacks, as its callbacks get it: the layout
	/// (`None` while the window is unloaded) and its [`WindowCtx`].
	///
	/// # Panics
	/// If the slot has not been set.
	pub fn with<R>(
		&self,
		_ui: &mut UiGuard,
		f: impl FnOnce(Option<&mut C>, &mut WindowCtx<'_, C, D>) -> R,
	) -> R {
		let ptr = self.get().ptr;
		// SAFETY: the slot holds a leaked Window<C, D>, never freed. Holding &mut UiGuard means no
		// window callback is running and no other borrow into any window is live
		let WindowData { layout, data, .. } = unsafe { Window::<C, D>::window_data(ptr) };
		f(layout.as_mut(), &mut WindowCtx::new(ptr, data))
	}
}

fn slot_set(slot: &Cell<Option<AnyWindow<'static>>>, window: AnyWindow<'static>) {
	assert!(slot.get().is_none(), "StaticWindow set twice");
	slot.set(Some(window));
}

fn slot_get(slot: &Cell<Option<AnyWindow<'static>>>) -> AnyWindow<'static> {
	slot.get().expect("StaticWindow used before set")
}

impl<C: Layout<D>, D> Default for StaticWindow<C, D> {
	fn default() -> Self {
		Self::new()
	}
}

/// A window whose types are known, so [`Nav::push_with`](super::Nav::push_with) can hand it an
/// argument.
pub trait TypedWindow<'w>: Into<AnyWindow<'w>> + Copy {
	/// The window's [`Layout`].
	type Layout: Layout<Self::Data> + 'static;
	/// The window's persistent data.
	type Data: 'static;
}

impl<'w, C: Layout<D>, D> TypedWindow<'w> for &'w Window<C, D> {
	type Data = D;
	type Layout = C;
}

impl<'w, C: Layout<D>, D> TypedWindow<'w> for &'w StaticWindow<C, D> {
	type Data = D;
	type Layout = C;
}

impl<C: Layout<D>, D> From<&StaticWindow<C, D>> for AnyWindow<'_> {
	fn from(slot: &StaticWindow<C, D>) -> Self {
		slot.get()
	}
}
