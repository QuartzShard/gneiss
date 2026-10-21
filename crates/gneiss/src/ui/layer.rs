//! The layer tree, and [`DataLayer`] for generic layers that carry and draw their own state.
extern crate alloc;
use alloc::boxed::Box;
use core::marker::PhantomData;

use crate::{
	alloc::PebbleAlloc,
	sys,
	ui::{
		UiGuard, WindowCtx,
		graphics::{GContext, Rectangle, grect, rect},
	},
};

/// Anything backed by a firmware layer.
///
/// # Safety
/// `as_ptr` must return a valid layer for as long as `self` lives.
pub unsafe trait AsLayer {
	/// The raw layer.
	fn as_ptr(&self) -> *mut sys::Layer;
}

/// Operations on any layer.
pub trait Layer: AsLayer {
	/// Add `child` above this layer's other children.
	fn add_child(&mut self, child: &mut impl AsLayer);
	/// Schedule a redraw.
	fn mark_dirty(&mut self);
	/// The layer's own coordinate space, which its drawing uses.
	fn bounds(&self) -> Rectangle;
	/// The part of [`bounds`](Self::bounds) not covered by system overlays, such as Timeline Quick
	/// View. [`UnobstructedService`](crate::service::UnobstructedService) reports changes.
	#[cfg(not(pbl_platform_aplite))]
	fn unobstructed_bounds(&self) -> Rectangle;
	/// The layer's position and size in its parent.
	fn frame(&self) -> Rectangle;
	/// Move or resize the layer within its parent.
	fn set_frame(&mut self, frame: Rectangle) -> &mut Self;
	/// Hide or show the layer and its children.
	fn set_hidden(&mut self, hidden: bool) -> &mut Self;
}

impl<T: AsLayer> Layer for T {
	fn add_child(&mut self, child: &mut impl AsLayer) {
		unsafe { sys::layer_add_child(self.as_ptr(), child.as_ptr()) };
	}

	fn mark_dirty(&mut self) {
		unsafe { sys::layer_mark_dirty(self.as_ptr()) };
	}

	fn bounds(&self) -> Rectangle {
		rect(unsafe { sys::layer_get_bounds(self.as_ptr()) })
	}

	#[cfg(not(pbl_platform_aplite))]
	fn unobstructed_bounds(&self) -> Rectangle {
		rect(unsafe { sys::layer_get_unobstructed_bounds(self.as_ptr()) })
	}

	fn frame(&self) -> Rectangle {
		rect(unsafe { sys::layer_get_frame(self.as_ptr()) })
	}

	fn set_frame(&mut self, frame: Rectangle) -> &mut Self {
		unsafe { sys::layer_set_frame(self.as_ptr(), grect(frame)) };
		self
	}

	fn set_hidden(&mut self, hidden: bool) -> &mut Self {
		unsafe { sys::layer_set_hidden(self.as_ptr(), hidden) };
		self
	}
}

/// Draws a [`DataLayer`]'s contents, given its data and bounds.
pub type DrawFn<T> = fn(&T, Rectangle, &mut GContext<'_>);

// A DrawFn<T> with its data pointer erased
type ErasedDrawFn = fn(*const (), Rectangle, &mut GContext<'_>);

/// A layer to add children to, owned by something else: a window's root layer, or a
/// [`ScrollLayer`](super::ScrollLayer)'s content.
pub struct RootLayer {
	ptr: *mut sys::Layer,
}

/// A layer that owns a `T` in its own allocation and can draw it with a [`DrawFn`].
pub struct DataLayer<T> {
	ptr:   *mut sys::Layer,
	_data: PhantomData<Box<LayerData<T>, PebbleAlloc>>,
}

#[repr(C)]
struct LayerData<T> {
	draw: Option<ErasedDrawFn>,
	val:  T,
}

/// Where `val` sits in every `LayerData<T>`, so one draw trampoline serves them all.
const VAL_OFFSET: usize = size_of::<Option<ErasedDrawFn>>();

impl RootLayer {
	/// The root layer of `win`.
	pub fn from_ctx<D>(win: &mut WindowCtx<'_, D>) -> Self {
		let ptr = unsafe { sys::window_get_root_layer(win.ptr()) };
		RootLayer { ptr }
	}

	pub(super) fn from_ptr(ptr: *mut sys::Layer) -> Self {
		RootLayer { ptr }
	}
}

unsafe impl AsLayer for RootLayer {
	fn as_ptr(&self) -> *mut gneiss_sys::Layer {
		self.ptr
	}
}
impl<T> DataLayer<T> {
	/// Take ownership of a raw layer.
	///
	/// # Safety
	/// `ptr` must be a layer created by [`DataLayer::new_with_data`] with this `T`, and not owned by
	/// any other `DataLayer`.
	pub unsafe fn from_raw(ptr: *mut sys::Layer) -> Self {
		Self {
			ptr,
			_data: PhantomData,
		}
	}

	/// Create a layer holding `data` in the same app heap allocation. `None` if the firmware is out
	/// of memory.
	pub fn new_with_data(frame: Rectangle, data: T) -> Option<Self> {
		// SAFETY: LayerData<T> must be align <= 4 or we can't be sure the layer alloc is correctly
		// aligned
		let _: () = const { assert!(align_of::<LayerData<T>>() <= 4) };
		let _: () = const { assert!(core::mem::offset_of!(LayerData<T>, val) == VAL_OFFSET) };
		let ptr = unsafe {
			sys::layer_create_with_data(grect(frame), size_of::<LayerData<T>>()).as_mut()?
		};
		unsafe {
			core::ptr::write(sys::layer_get_data(ptr).cast(), LayerData {
				draw: None,
				val:  data,
			});
		};
		Some(Self {
			ptr,
			_data: PhantomData,
		})
	}

	fn layer_data(&mut self) -> &mut LayerData<T> {
		// SAFETY: written in new_with_data, lives until Drop; &mut self excludes other borrows, and
		// the draw trampoline only runs during render, never inside a borrow of self
		unsafe {
			sys::layer_get_data(self.ptr)
				.cast::<LayerData<T>>()
				.as_mut()
				.unwrap_unchecked()
		}
	}

	/// The layer's data.
	pub fn data(&mut self) -> &mut T {
		&mut self.layer_data().val
	}

	/// Draw this layer's contents with `draw` from now on.
	pub fn set_draw(&mut self, draw: DrawFn<T>) -> &mut Self {
		// SAFETY: &T and *const () are ABI-compatible for a sized T, and the trampoline only calls
		// this with a pointer to this layer's T
		self.layer_data().draw =
			Some(unsafe { core::mem::transmute::<DrawFn<T>, ErasedDrawFn>(draw) });
		unsafe { sys::layer_set_update_proc(self.ptr, Some(draw_trampoline)) };
		self.mark_dirty();
		self
	}

	/// Stop drawing this layer's contents.
	pub fn clear_draw(&mut self) -> &mut Self {
		self.layer_data().draw = None;
		unsafe { sys::layer_set_update_proc(self.ptr, None) };
		self.mark_dirty();
		self
	}
}

unsafe extern "C" fn draw_trampoline(layer: *mut sys::Layer, ctx: *mut sys::GContext) {
	let _ui = UiGuard::enter_internal();
	// SAFETY: only registered on DataLayer layers, whose data is a repr(C) LayerData<T> with draw at
	// offset 0 and val at VAL_OFFSET; render never overlaps a &mut from layer_data
	let (draw, val) = unsafe {
		let data = sys::layer_get_data(layer);
		(
			*data.cast::<Option<ErasedDrawFn>>(),
			data.byte_add(VAL_OFFSET).cast::<()>(),
		)
	};
	let Some(draw) = draw else {
		return;
	};
	// SAFETY: these are this update proc call's arguments, and ctx doesn't escape it
	let mut ctx = unsafe { GContext::new(ctx, layer) };
	// SAFETY: the layer is live while it renders
	let bounds = rect(unsafe { sys::layer_get_bounds(layer) });
	draw(val, bounds, &mut ctx);
}

impl DataLayer<()> {
	/// Create a layer with no data. `None` if the firmware is out of memory.
	pub fn new(frame: Rectangle) -> Option<Self> {
		Self::new_with_data(frame, ())
	}
}

unsafe impl<T> AsLayer for DataLayer<T> {
	fn as_ptr(&self) -> *mut sys::Layer {
		self.ptr
	}
}
impl<T> Drop for DataLayer<T> {
	fn drop(&mut self) {
		unsafe {
			// SAFETY: We don't own this allocation, but we do own the T and must drop() it
			core::ptr::drop_in_place(self.data());
			sys::layer_remove_child_layers(self.ptr);
			sys::layer_destroy(self.ptr);
		}
	}
}
