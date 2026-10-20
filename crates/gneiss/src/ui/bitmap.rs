//! Bitmap layers.
use core::borrow::Borrow;

use crate::{
	resource::image::GBitMap,
	sys,
	ui::{
		AsLayer,
		graphics::{Colour, Rectangle, grect},
	},
};

/// Where a bitmap sits within its layer.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Align {
	Center      = sys::GAlign_GAlignCenter,
	TopLeft     = sys::GAlign_GAlignTopLeft,
	TopRight    = sys::GAlign_GAlignTopRight,
	Top         = sys::GAlign_GAlignTop,
	Left        = sys::GAlign_GAlignLeft,
	Bottom      = sys::GAlign_GAlignBottom,
	Right       = sys::GAlign_GAlignRight,
	BottomRight = sys::GAlign_GAlignBottomRight,
	BottomLeft  = sys::GAlign_GAlignBottomLeft,
}

/// A layer that draws a bitmap it owns. `B` is a [`GBitMap`], or `&'static GBitMap` for one that
/// lives as long as the app, or an `Rc<GBitMap>` to share one between windows that come and go.
/// [`replace_bitmap`](Self::replace_bitmap) hands the old bitmap back, to swap between bitmaps
/// without loading them again.
pub struct BitmapLayer<B: Borrow<GBitMap> = GBitMap> {
	ptr:    *mut sys::BitmapLayer,
	bitmap: Option<B>,
}

impl<B: Borrow<GBitMap>> BitmapLayer<B> {
	/// Create a bitmap layer with no bitmap. `None` if the firmware is out of memory.
	pub fn new(frame: Rectangle) -> Option<Self> {
		// SAFETY: no preconditions; NULL on OOM
		let ptr = unsafe { sys::bitmap_layer_create(grect(frame)).as_mut()? };
		Some(Self { ptr, bitmap: None })
	}

	/// Draw `bitmap`, dropping the old one.
	pub fn set_bitmap(&mut self, bitmap: B) -> &mut Self {
		self.replace_bitmap(bitmap);
		self
	}

	/// Draw `bitmap`. Returns the old bitmap, if it had one, which the layer no longer points at.
	pub fn replace_bitmap(&mut self, bitmap: B) -> Option<B> {
		// SAFETY: the firmware bitmap lives as long as the GBitMap, which self.bitmap keeps alive;
		// moving the handle doesn't move it
		unsafe { sys::bitmap_layer_set_bitmap(self.ptr, bitmap.borrow().as_ptr()) };
		self.bitmap.replace(bitmap)
	}

	/// The current bitmap, if any.
	pub fn bitmap(&self) -> Option<&GBitMap> {
		self.bitmap.as_ref().map(Borrow::borrow)
	}

	/// Set where the bitmap sits within the layer.
	pub fn set_alignment(&mut self, alignment: Align) -> &mut Self {
		unsafe { sys::bitmap_layer_set_alignment(self.ptr, alignment as u8) };
		self
	}

	/// Set the colour behind the bitmap.
	pub fn set_background_colour(&mut self, colour: Colour) -> &mut Self {
		unsafe { sys::bitmap_layer_set_background_color(self.ptr, colour.into()) };
		self
	}
}

unsafe impl<B: Borrow<GBitMap>> AsLayer for BitmapLayer<B> {
	fn as_ptr(&self) -> *mut sys::Layer {
		unsafe { sys::bitmap_layer_get_layer(self.ptr) }
	}
}

impl<B: Borrow<GBitMap>> Drop for BitmapLayer<B> {
	fn drop(&mut self) {
		// SAFETY: destroyed before the bitmap field drops, so the firmware never reads a freed
		// bitmap
		unsafe {
			sys::layer_remove_child_layers(self.as_ptr());
			sys::bitmap_layer_destroy(self.ptr);
		}
	}
}
