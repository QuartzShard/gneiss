//! The layer tree, and [`DataLayer`] for layers that carry and draw their own state.
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
	/// The layer's position and size in its parent.
	fn frame(&self) -> Rectangle;
	/// Move or resize the layer within its parent.
	fn set_frame(&mut self, frame: Rectangle);
	/// Hide or show the layer and its children.
	fn set_hidden(&mut self, hidden: bool);
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

	fn frame(&self) -> Rectangle {
		rect(unsafe { sys::layer_get_frame(self.as_ptr()) })
	}

	fn set_frame(&mut self, frame: Rectangle) {
		unsafe { sys::layer_set_frame(self.as_ptr(), grect(frame)) };
	}

	fn set_hidden(&mut self, hidden: bool) {
		unsafe { sys::layer_set_hidden(self.as_ptr(), hidden) };
	}
}

/// Draws a [`DataLayer`]'s contents, given its data and bounds.
pub type DrawFn<T> = fn(&T, Rectangle, &mut GContext<'_>);

/// A window's root layer.
pub struct RootLayer {
	ptr: *mut sys::Layer,
}

/// A layer that owns a `T` in its own allocation and can draw it with a [`DrawFn`].
pub struct DataLayer<T> {
	ptr:   *mut sys::Layer,
	_data: PhantomData<Box<LayerData<T>, PebbleAlloc>>,
}

struct LayerData<T> {
	draw: Option<DrawFn<T>>,
	val:  T,
}

impl RootLayer {
	/// The root layer of `win`.
	pub fn from_ctx<C, D>(win: &mut WindowCtx<'_, C, D>) -> Self {
		let ptr = unsafe { sys::window_get_root_layer(win.ptr()) };
		RootLayer { ptr }
	}
}

unsafe impl AsLayer for RootLayer {
	fn as_ptr(&self) -> *mut pebble_sys::Layer {
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
	pub fn set_draw(&mut self, draw: DrawFn<T>) {
		self.layer_data().draw = Some(draw);
		unsafe { sys::layer_set_update_proc(self.ptr, Some(Self::__draw)) };
		self.mark_dirty();
	}

	/// Stop drawing this layer's contents.
	pub fn clear_draw(&mut self) {
		self.layer_data().draw = None;
		unsafe { sys::layer_set_update_proc(self.ptr, None) };
		self.mark_dirty();
	}

	unsafe extern "C" fn __draw(layer: *mut sys::Layer, ctx: *mut sys::GContext) {
		let _ui = UiGuard::enter();
		// SAFETY: only registered on DataLayer<T> layers, whose data is a LayerData<T>; render
		// never overlaps a &mut from layer_data
		let data = unsafe { sys::layer_get_data(layer).cast::<LayerData<T>>().as_ref() };
		let Some(LayerData {
			draw: Some(draw),
			val,
		}) = data
		else {
			return;
		};
		// SAFETY: these are this update proc call's arguments, and ctx doesn't escape it
		let mut ctx = unsafe { GContext::new(ctx, layer) };
		// SAFETY: the layer is live while it renders
		let bounds = rect(unsafe { sys::layer_get_bounds(layer) });
		draw(val, bounds, &mut ctx);
	}
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
