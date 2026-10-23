//! Bitmap layers, plain and rotating.
use core::borrow::Borrow;

use crate::{
	angle::{Angle, DeltaAngle},
	resource::image::GBitMap,
	sys,
	ui::{
		AsLayer,
		graphics::{Colour, CompOp, Point, Rectangle, gpoint, grect},
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

	/// How the bitmap combines with what's behind it. [`CompOp::Set`] draws a PNG's transparency;
	/// the default, [`CompOp::Assign`], ignores it.
	pub fn set_compositing(&mut self, mode: CompOp) -> &mut Self {
		unsafe { sys::bitmap_layer_set_compositing_mode(self.ptr, mode as u8) };
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

/// A layer that draws a bitmap it owns, rotated. It starts at the top left of its parent, sized to
/// fit the bitmap at any angle: move it with [`Layer::set_frame`](super::Layer::set_frame), keeping
/// that size. Rotating is slow to draw, so keep it small or still.
///
/// `B` is as for [`BitmapLayer`].
pub struct RotBitmapLayer<B: Borrow<GBitMap> = GBitMap> {
	ptr:     *mut sys::RotBitmapLayer,
	_bitmap: B,
}

impl<B: Borrow<GBitMap>> RotBitmapLayer<B> {
	/// Create a layer drawing `bitmap`, unrotated. `None` if the firmware is out of memory.
	pub fn new(bitmap: B) -> Option<Self> {
		// SAFETY: the firmware keeps the bitmap pointer, and self owns the bitmap until the layer
		// is destroyed; NULL on OOM
		let ptr = unsafe { sys::rot_bitmap_layer_create(bitmap.borrow().as_ptr()).as_mut()? };
		Some(Self {
			ptr,
			_bitmap: bitmap,
		})
	}

	/// Rotate to `angle`, clockwise. A compass [`Heading`](crate::compass::Heading) points the
	/// bitmap's top at north.
	pub fn set_angle(&mut self, angle: Angle) -> &mut Self {
		unsafe { sys::rot_bitmap_layer_set_angle(self.ptr, angle.inner()) };
		self
	}

	/// Rotate by `change` more, clockwise.
	pub fn rotate_by(&mut self, change: DeltaAngle) -> &mut Self {
		unsafe { sys::rot_bitmap_layer_increment_angle(self.ptr, change.inner()) };
		self
	}

	/// Rotate around `point`, in the bitmap's coordinates, instead of its centre. The point stays
	/// at the layer's centre, and the layer resizes so the bitmap still fits.
	pub fn set_centre(&mut self, point: Point) -> &mut Self {
		unsafe { sys::rot_bitmap_set_src_ic(self.ptr, gpoint(point)) };
		self
	}

	/// The colour of the layer's corners, which the bitmap doesn't cover. Clear by default.
	pub fn set_corner_colour(&mut self, colour: Colour) -> &mut Self {
		unsafe { sys::rot_bitmap_layer_set_corner_clip_color(self.ptr, colour.into()) };
		self
	}

	/// As for [`BitmapLayer::set_compositing`].
	pub fn set_compositing(&mut self, mode: CompOp) -> &mut Self {
		unsafe { sys::rot_bitmap_set_compositing_mode(self.ptr, mode as u8) };
		self
	}
}

// SAFETY: the firmware's RotBitmapLayer starts with its Layer, which is all
// rot_bitmap_layer_get_layer returns. aplite's SDK has no such call
unsafe impl<B: Borrow<GBitMap>> AsLayer for RotBitmapLayer<B> {
	fn as_ptr(&self) -> *mut sys::Layer {
		self.ptr.cast()
	}
}

impl<B: Borrow<GBitMap>> Drop for RotBitmapLayer<B> {
	fn drop(&mut self) {
		// SAFETY: destroyed before the bitmap field drops, as for BitmapLayer
		unsafe {
			sys::layer_remove_child_layers(self.as_ptr());
			sys::rot_bitmap_layer_destroy(self.ptr);
		}
	}
}
