use core::slice;

use crate::{
	resource::{ImageKey, PdcKey},
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

/// An image loaded at runtime
pub struct GBitMap {
	inner:   *mut sys::GBitmap,
	bounds:  sys::GRect,
	// Points into `inner`'s allocation, valid until gbitmap_destroy;
	palette: Option<(*mut GColour, usize)>,
}
//	sys::gbitmap_get_data(bitmap);
//	sys::gbitmap_get_bounds(bitmap);
//	sys::gbitmap_get_format(z);
//	sys::gbitmap_get_palette(a);
//	sys::gbitmap_get_bytes_per_row(bitmap);
//	sys::gbitmap_get_data_row_info(bitmap, y);

//	sys::gbitmap_set_data(bitmap, data, format, row_size_bytes, free_on_destroy);
//	sys::gbitmap_set_bounds(bitmap, bounds);
//	sys::gbitmap_set_palette(bitmap, palette, free_on_destroy);
impl GBitMap {
	pub fn load(key: ImageKey) -> Option<Self> {
		let inner = unsafe { sys::gbitmap_create_with_resource(key.0.get()) };
		if inner.is_null() {
			None
		} else {
			unsafe {
				let format = sys::gbitmap_get_format(inner);
				let palette_len = match format {
					sys::GBitmapFormat_GBitmapFormat1BitPalette => Some(2),
					sys::GBitmapFormat_GBitmapFormat2BitPalette => Some(4),
					sys::GBitmapFormat_GBitmapFormat4BitPalette => Some(16),
					_ => None,
				};
				let palette = palette_len.map(|len| (sys::gbitmap_get_palette(inner).cast(), len));
				let bounds = sys::gbitmap_get_bounds(inner);

				Some(GBitMap {
					inner,
					bounds,
					palette,
				})
			}
		}
	}

	pub fn palette(&mut self) -> Option<&mut [GColour]> {
		self.palette
			.map(|(ptr, len)| unsafe { slice::from_raw_parts_mut(ptr, len) })
	}

	pub fn origin(&self) -> sys::GPoint {
		self.bounds.origin
	}

	pub fn size(&self) -> sys::GSize {
		self.bounds.size
	}

	pub fn bounds(&self) -> sys::GRect {
		self.bounds
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
	pub fn load(key: PdcKey) -> Option<Self> {
		let inner = unsafe { sys::gdraw_command_sequence_create_with_resource(key.0.get()) };
		if inner.is_null() {
			None
		} else {
			Some(DrawCommandSequence { inner })
		}
	}
}
impl Drop for DrawCommandSequence {
	fn drop(&mut self) {
		unsafe { sys::gdraw_command_sequence_destroy(self.inner) }
	}
}
