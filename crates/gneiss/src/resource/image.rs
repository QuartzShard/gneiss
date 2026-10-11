//! Image resources: bitmaps and their palettes, and draw command images and sequences.
//!
//! A [`GBitMap`] is shown by a [`BitmapLayer`](crate::ui::BitmapLayer) or drawn with
//! [`GContext::draw_bitmap`](crate::ui::graphics::GContext::draw_bitmap). Draw command images and
//! sequences are vector graphics, drawn from a [`DataLayer`](crate::ui::DataLayer) with
//! [`draw_pdc`](crate::ui::graphics::GContext::draw_pdc) and
//! [`draw_pdc_sequence`](crate::ui::graphics::GContext::draw_pdc_sequence). Each type owns its
//! firmware object and frees it on drop.
use core::slice;

extern crate alloc;

use crate::{
	cell::SyncCell,
	resource::{ImageKey, PdcKey, PdcSeqKey},
	sys,
	ui::graphics::Colour,
};

/// A palette several bitmaps can share, set with [`GBitMap::set_shared_palette`]. It holds 16
/// colours, enough for any palettised format; smaller formats use the first 2 or 4.
#[repr(transparent)]
pub struct SharedPalette {
	inner: [SyncCell<Colour>; 16],
}

impl SharedPalette {
	/// Construct Self from an array. Usable in const context
	/// ```no_run
	/// # use gneiss::{resource::image::SharedPalette, ui::graphics::Colour};
	/// static PALETTE: SharedPalette = SharedPalette::new([Colour { argb: 0xa5 }; 16]);
	/// ```
	pub const fn new(val: [Colour; 16]) -> Self {
		Self {
			inner: val.map(SyncCell::new),
		}
	}

	/// Construct self from an array and leak it onto the heap ([`PebbleAlloc`](crate::alloc::PebbleAlloc)),
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
		self.inner.get(i).map(|c| c.get())
	}
}

/// A bitmap, loaded from an image resource.
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
	/// Load the image resource `key`. `None` if the firmware is out of memory.
	pub fn load(key: ImageKey) -> Option<Self> {
		let inner = unsafe { sys::gbitmap_create_with_resource(key.0.get()) };
		(!inner.is_null()).then_some(GBitMap {
			inner,
			shared_palette: false,
		})
	}

	/// The palette of a 1, 2 or 4-bit palettised bitmap, to recolour it in place. `None` for other
	/// formats, and once [`set_shared_palette`](Self::set_shared_palette) has replaced it.
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

	/// The top-left of [`bounds`](Self::bounds).
	pub fn origin(&self) -> sys::GPoint {
		self.bounds().origin
	}

	/// The size of [`bounds`](Self::bounds).
	pub fn size(&self) -> sys::GSize {
		self.bounds().size
	}

	/// The part of the bitmap that gets drawn, usually all of it.
	pub fn bounds(&self) -> sys::GRect {
		unsafe { sys::gbitmap_get_bounds(self.inner) }
	}

	/// The raw bitmap, valid while this `GBitMap` lives.
	pub fn as_ptr(&self) -> *mut sys::GBitmap {
		self.inner
	}

	/// Draw with `shared` instead of the bitmap's own palette, so several bitmaps can be recoloured
	/// together.
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

/// A draw command image: vector graphics, from an `svg` or `pdc` resource.
pub struct DrawCommand {
	inner: *mut sys::GDrawCommandImage,
}
impl DrawCommand {
	/// Load the resource `key`. `None` if the firmware is out of memory.
	pub fn load(key: PdcKey) -> Option<Self> {
		let inner = unsafe { sys::gdraw_command_image_create_with_resource(key.0.get()) };
		if inner.is_null() {
			None
		} else {
			Some(DrawCommand { inner })
		}
	}

	/// The raw image, valid while this `DrawCommand` lives.
	pub fn as_ptr(&self) -> *mut sys::GDrawCommandImage {
		self.inner
	}
}
impl Drop for DrawCommand {
	fn drop(&mut self) {
		unsafe { sys::gdraw_command_image_destroy(self.inner) }
	}
}

/// A draw command sequence: frames of vector graphics, from a directory of `svg` files or an
/// animated `pdc`.
pub struct DrawCommandSequence {
	inner: *mut sys::GDrawCommandSequence,
}
impl DrawCommandSequence {
	/// Load the resource `key`. `None` if the firmware is out of memory.
	pub fn load(key: PdcSeqKey) -> Option<Self> {
		let inner = unsafe { sys::gdraw_command_sequence_create_with_resource(key.0.get()) };
		if inner.is_null() {
			None
		} else {
			Some(DrawCommandSequence { inner })
		}
	}

	/// The raw sequence, valid while this `DrawCommandSequence` lives.
	pub fn as_ptr(&self) -> *mut sys::GDrawCommandSequence {
		self.inner
	}
}
impl Drop for DrawCommandSequence {
	fn drop(&mut self) {
		unsafe { sys::gdraw_command_sequence_destroy(self.inner) }
	}
}
