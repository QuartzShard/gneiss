//! Image resources: bitmaps and their palettes, and draw command images and sequences.
use core::{cell::Cell, slice};

extern crate alloc;

use crate::{
	resource::{ImageKey, PdcKey, PdcSeqKey},
	sys,
	ui::graphics::Colour,
};

/// Rust representation of a valid, shared, mutible palette. Long enough to account for any use,
/// handing to a 2-bit image will just ignore the extra 12 colours.
#[repr(transparent)]
pub struct SharedPalette {
	inner: [Cell<Colour>; 16],
}

// SAFETY: Pebble apps are single-threaded - every callback runs on the app task,
// so a shared reference never actually crosses a thread boundary.
unsafe impl Sync for SharedPalette {}

impl SharedPalette {
	/// Construct Self from an array. Usable in const context
	/// ```
	/// static PALETTE: SharedPalette = SharedPalette::new([Colour { argb: 0xa5 }; 16]);
	/// ```
	pub const fn new(val: [Colour; 16]) -> Self {
		Self {
			inner: val.map(Cell::new),
		}
	}

	/// Construct self from an array and leak it onto the heap ([`PebbleAlloc`]),
	/// yielding the `&'static` that [`GBitMap::set_shared_palette`] requires.
	/// Convenience for runtime-created shared palettes
	pub fn new_leak(val: [Colour; 16]) -> &'static Self {
		use alloc::boxed::Box;
		Box::leak(Box::new_in(Self::new(val), crate::alloc::PebbleAlloc))
	}

	/// For static allocations, get a pointer to Self viewed as `sys::GColor`
	/// used at FFI boundary
	pub fn as_sys_ptr(&'static self) -> *mut sys::GColor {
		core::ptr::from_ref(self).cast_mut().cast()
	}

	/// Set a palette entry. NOP on OOB.
	pub fn set(&self, i: usize, val: Colour) {
		if let Some(v) = self.inner.get(i) {
			v.set(val);
		}
	}

	/// Get a palette entry. NOP on OOB
	pub fn get(&self, i: usize) -> Option<Colour> {
		self.inner.get(i).map(Cell::get)
	}
}

/// An image loaded at runtime
pub struct GBitMap {
	inner:          *mut sys::GBitmap,
	shared_palette: bool,
}

//	The following are currently absent from this type, and are either unimplemented!() or for a
//	future `GBitMapBuilder`-type API.
//	sys::gbitmap_get_data(bitmap);
//	sys::gbitmap_get_bytes_per_row(bitmap);
//	sys::gbitmap_get_data_row_info(bitmap, y);
//	sys::gbitmap_set_data(bitmap, data, format, row_size_bytes, free_on_destroy);
//	sys::gbitmap_set_bounds(bitmap, bounds);
impl GBitMap {
	pub fn load(key: ImageKey) -> Option<Self> {
		let inner = unsafe { sys::gbitmap_create_with_resource(key.0.get()) };
		(!inner.is_null()).then_some(GBitMap {
			inner,
			shared_palette: false,
		})
	}

	pub fn palette(&mut self) -> Option<&mut [Colour]> {
		let format = unsafe { sys::gbitmap_get_format(self.inner) };
		let mut palette_len = match format {
			sys::GBitmapFormat_GBitmapFormat1BitPalette => Some(2),
			sys::GBitmapFormat_GBitmapFormat2BitPalette => Some(4),
			sys::GBitmapFormat_GBitmapFormat4BitPalette => Some(16),
			_ => None,
		};
		palette_len
			// SAFETY: Constructing the slice is not valid if it's pointing to shared memory
			.take_if(|_| !self.shared_palette)
			.map(|len| unsafe {
				slice::from_raw_parts_mut(sys::gbitmap_get_palette(self.inner).cast(), len)
			})
	}

	pub fn origin(&self) -> sys::GPoint {
		self.bounds().origin
	}

	pub fn size(&self) -> sys::GSize {
		self.bounds().size
	}

	pub fn bounds(&self) -> sys::GRect {
		unsafe { sys::gbitmap_get_bounds(self.inner) }
	}

	pub fn as_ptr(&self) -> *mut sys::GBitmap {
		self.inner
	}

	pub fn set_shared_palette(&mut self, shared: &'static SharedPalette) {
		// SAFETY: Takes &mut self, so no ref to the current palette exists - safe to destroy it.
		self.shared_palette = true;
		unsafe {
			sys::gbitmap_set_palette(self.inner, shared.as_sys_ptr(), false);
		}
	}
}

impl Drop for GBitMap {
	fn drop(&mut self) {
		unsafe { sys::gbitmap_destroy(self.inner) }
	}
}

pub struct DrawCommand {
	inner: *mut sys::GDrawCommandImage,
}
impl DrawCommand {
	pub fn load(key: PdcKey) -> Option<Self> {
		let inner = unsafe { sys::gdraw_command_image_create_with_resource(key.0.get()) };
		if inner.is_null() {
			None
		} else {
			Some(DrawCommand { inner })
		}
	}

	pub fn as_ptr(&self) -> *mut sys::GDrawCommandImage {
		self.inner
	}
}
impl Drop for DrawCommand {
	fn drop(&mut self) {
		unsafe { sys::gdraw_command_image_destroy(self.inner) }
	}
}

pub struct DrawCommandSequence {
	inner: *mut sys::GDrawCommandSequence,
}
impl DrawCommandSequence {
	pub fn load(key: PdcSeqKey) -> Option<Self> {
		let inner = unsafe { sys::gdraw_command_sequence_create_with_resource(key.0.get()) };
		if inner.is_null() {
			None
		} else {
			Some(DrawCommandSequence { inner })
		}
	}

	pub fn as_ptr(&self) -> *mut sys::GDrawCommandSequence {
		self.inner
	}
}
impl Drop for DrawCommandSequence {
	fn drop(&mut self) {
		unsafe { sys::gdraw_command_sequence_destroy(self.inner) }
	}
}
