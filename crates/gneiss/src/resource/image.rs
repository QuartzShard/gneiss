use core::{cell::Cell, ptr, slice};

extern crate alloc;

use crate::{
	resource::{ImageKey, PdcKey, PdcSeqKey},
	sys,
};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct GColour {
	pub argb: u8,
}

impl GColour {
	/// Each channel is 2-bit, 0-3. Higher values are truncated!
	pub const fn argb(a: u8, r: u8, g: u8, b: u8) -> Self {
		let a = (a & 0b11) << 6;
		let r = (r & 0b11) << 4;
		let g = (g & 0b11) << 2;
		let b = b & 0b11;
		Self {
			argb: a | r | g | b,
		}
	}

	/// Each channel is 2-bit, 0-3. Higher values are truncated!
	pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
		Self::argb(0b11, r, g, b)
	}

	/// Rounds each channel into the pebble's colorspace.
	pub const fn from_argb8(argb: u32) -> Self {
		// Round-to-nearest in the 8-bit space: 0..=255 -> 0..=3
		const fn round2(c: u32) -> u8 {
			((c * 3 + 127) / 255) as u8
		}
		Self::argb(
			round2((argb >> 24) & 0xFF),
			round2((argb >> 16) & 0xFF),
			round2((argb >> 8) & 0xFF),
			round2(argb & 0xFF),
		)
	}

	pub const fn from_rgb8(rgb: u32) -> Self {
		Self::from_argb8(0xFF00_0000 | rgb)
	}
}

/// Rust representation of a valid, shared, mutible palette. Long enough to account for any use,
/// handing to a 2-bit image will just ignore the extra 12 colours.
#[repr(transparent)]
pub struct SharedPalette {
	inner: [Cell<GColour>; 16],
}

// SAFETY: Pebble apps are single-threaded - every callback runs on the app task,
// so a shared reference never actually crosses a thread boundary.
unsafe impl Sync for SharedPalette {}

impl SharedPalette {
	/// Construct Self from an array. Usable in const context
	/// ```
	/// static PALETTE: SharedPalette = SharedPalette::new([GColour { argb: 0xa5 }; 16]);
	/// ```
	pub const fn new(val: [GColour; 16]) -> Self {
		Self {
			inner: val.map(Cell::new),
		}
	}

	/// Construct self from an array and leak it onto the heap ([`PebbleAlloc`]),
	/// yielding the `&'static` that [`GBitMap::set_shared_palette`] requires.
	/// Convenience for runtime-created shared palettes
	pub fn new_leak(val: [GColour; 16]) -> &'static Self {
		use alloc::boxed::Box;
		Box::leak(Box::new_in(Self::new(val), crate::alloc::PebbleAlloc))
	}

	/// For static allocations, get a pointer to Self viewed as `sys::GColor`
	/// used at FFI boundary
	pub fn as_sys_ptr(&'static self) -> *mut sys::GColor {
		core::ptr::from_ref(self).cast_mut().cast()
	}

	/// Set a palette entry. NOP on OOB.
	pub fn set(&self, i: usize, val: GColour) {
		if let Some(v) = self.inner.get(i) {
			v.set(val);
		}
	}

	/// Get a palette entry. NOP on OOB
	pub fn get(&self, i: usize) -> Option<GColour> {
		self.inner.get(i).map(Cell::get)
	}
}

/// An image loaded at runtime
pub struct GBitMap {
	inner:          ptr::NonNull<sys::GBitmap>,
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
		ptr::NonNull::new(inner).map(|inner| GBitMap {
			inner,
			shared_palette: false,
		})
	}

	pub fn palette(&mut self) -> Option<&mut [GColour]> {
		let format = unsafe { sys::gbitmap_get_format(self.inner.as_ptr()) };
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
				slice::from_raw_parts_mut(sys::gbitmap_get_palette(self.inner.as_ptr()).cast(), len)
			})
	}

	pub fn origin(&self) -> sys::GPoint {
		self.bounds().origin
	}

	pub fn size(&self) -> sys::GSize {
		self.bounds().size
	}

	pub fn bounds(&self) -> sys::GRect {
		unsafe { sys::gbitmap_get_bounds(self.inner.as_ptr()) }
	}

	pub fn as_ptr(&self) -> *mut sys::GBitmap {
		self.inner.as_ptr()
	}

	pub fn set_shared_palette(&mut self, shared: &'static SharedPalette) {
		// SAFETY: Takes &mut self, so no ref to the current palette exists - safe to destroy it.
		self.shared_palette = true;
		unsafe {
			sys::gbitmap_set_palette(self.inner.as_ptr(), shared.as_sys_ptr(), false);
		}
	}
}

impl Drop for GBitMap {
	fn drop(&mut self) {
		unsafe { sys::gbitmap_destroy(self.inner.as_ptr()) }
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
