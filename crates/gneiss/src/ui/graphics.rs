//! Drawing: the firmware's own primitives through [`GContext`], or `embedded-graphics` through
//! [`Framebuffer`].
//!
//! Drawing happens when the firmware redraws a layer. Give a [`DataLayer`](super::DataLayer) a
//! [`DrawFn`](super::DrawFn), and it's called with the layer's data, its bounds, and a
//! [`GContext`] whenever the layer needs drawing. [`mark_dirty`](super::Layer::mark_dirty) asks for
//! a redraw after the data changes. Coordinates are the layer's own, with `(0, 0)` at its top left.
//!
//! ```no_run
//! # use gneiss::ui::graphics::*;
//! # fn tip(_angle: i32, bounds: Rectangle) -> Point { bounds.top_left }
//! fn draw_hand(angle: &i32, bounds: Rectangle, ctx: &mut GContext<'_>) {
//!     ctx.set_stroke_colour(Colour::rgb(3, 0, 0));
//!     ctx.set_stroke_width(3);
//!     ctx.draw_line(bounds.center(), tip(*angle, bounds));
//! }
//! ```
//!
//! [`GContext`] draws shapes, text, bitmaps and draw command images with configured colours, stroke
//! width and compositing mode.
//!
//! # embedded-graphics
//!
//! [`GContext::with_framebuffer`] captures the screen as a [`Framebuffer`], an `embedded-graphics`
//! [`DrawTarget`] clipped to the layer, so anything that draws with `embedded-graphics` draws
//! straight onto the watch. gneiss uses `embedded-graphics`' [`Point`], [`Size`] and [`Rectangle`]
//! for geometry throughout.
//!
//! # Colour
//!
//! [`Colour`] (or [`Color`]) is the watch's native format: 2 bits each of alpha, red, green and
//! blue, for 64 opaque colours. It converts from `embedded-graphics`' colour types, rounding to the
//! nearest. Black and white watches draw each colour as black, white, or nothing.

use core::{convert::Infallible, ffi::CStr, marker::PhantomData};

use embedded_graphics_core::{
	Pixel,
	draw_target::DrawTarget,
	geometry::OriginDimensions,
	pixelcolor::{
		BinaryColor, PixelColor, Rgb565, Rgb888, RgbColor,
		raw::{RawData, RawU8},
	},
};
pub use embedded_graphics_core::{
	geometry::{Point, Size},
	primitives::Rectangle,
};

use crate::{
	resource::{
		font::Font,
		image::{DrawCommand, DrawCommandSequence, GBitMap},
	},
	sys,
	ui::TextAlignment,
};

pub(crate) fn gpoint(p: Point) -> sys::GPoint {
	sys::GPoint {
		x: clamp_i16(p.x),
		y: clamp_i16(p.y),
	}
}

pub(crate) fn gsize(s: Size) -> sys::GSize {
	sys::GSize {
		w: clamp_i16(s.width as i32),
		h: clamp_i16(s.height as i32),
	}
}

pub(crate) fn grect(r: Rectangle) -> sys::GRect {
	sys::GRect {
		origin: gpoint(r.top_left),
		size:   gsize(r.size),
	}
}

pub(crate) fn point(p: sys::GPoint) -> Point {
	Point::new(p.x.into(), p.y.into())
}

pub(crate) fn size(s: sys::GSize) -> Size {
	Size::new(s.w.max(0) as u32, s.h.max(0) as u32)
}

pub(crate) fn rect(r: sys::GRect) -> Rectangle {
	Rectangle::new(point(r.origin), size(r.size))
}

fn clamp_i16(v: i32) -> i16 {
	v.clamp(i16::MIN.into(), i16::MAX.into()) as i16
}

/// A colour in the watch's native format: 8-bit ARGB, 2 bits per channel.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct Colour {
	/// `0bAARRGGBB`: alpha in the top two bits, then red, green and blue.
	pub argb: u8,
}

/// [`Colour`], spelled the wrong way.
pub type Color = Colour;

impl From<Colour> for sys::GColor {
	fn from(colour: Colour) -> Self {
		sys::GColor8 { argb: colour.argb }
	}
}

impl Colour {
	/// From 2-bit channels, `0..=3`. Higher bits are dropped.
	pub const fn argb(a: u8, r: u8, g: u8, b: u8) -> Self {
		let a = (a & 0b11) << 6;
		let r = (r & 0b11) << 4;
		let g = (g & 0b11) << 2;
		let b = b & 0b11;
		Self {
			argb: a | r | g | b,
		}
	}

	/// From opaque 2-bit channels, `0..=3`. Higher bits are dropped.
	pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
		Self::argb(0b11, r, g, b)
	}

	/// From 8-bit channels, `0xAARRGGBB`, rounding each to the nearest 2-bit value.
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

	/// From opaque 8-bit channels, `0xRRGGBB`, rounding each to the nearest 2-bit value.
	pub const fn from_rgb8(rgb: u32) -> Self {
		Self::from_argb8(0xFF00_0000 | rgb)
	}
}

impl PixelColor for Colour {
	type Raw = RawU8;
}

impl From<RawU8> for Colour {
	fn from(raw: RawU8) -> Self {
		Self {
			argb: raw.into_inner(),
		}
	}
}

impl From<Rgb888> for Colour {
	fn from(c: Rgb888) -> Self {
		Self::from_rgb8(u32::from(c.r()) << 16 | u32::from(c.g()) << 8 | u32::from(c.b()))
	}
}

impl From<Rgb565> for Colour {
	fn from(c: Rgb565) -> Self {
		Rgb888::from(c).into()
	}
}

impl From<BinaryColor> for Colour {
	fn from(c: BinaryColor) -> Self {
		match c {
			BinaryColor::Off => Self::rgb(0, 0, 0),
			BinaryColor::On => Self::rgb(3, 3, 3),
		}
	}
}

impl Colour {
	const fn alpha(self) -> u8 {
		self.argb >> 6
	}

	const fn channel(self, shift: u8) -> u8 {
		(self.argb >> shift) & 0b11
	}

	/// `self` drawn over `dest` at `self`'s alpha. Matches the firmware's `gcolor_alpha_blend`
	/// lookup table exactly.
	fn blend_over(self, dest: Self) -> Self {
		let (s_weight, d_weight) = match self.alpha() {
			0 => return dest,
			1 => (1, 2),
			2 => (2, 1),
			_ => return Self::argb(3, self.channel(4), self.channel(2), self.channel(0)),
		};
		let mix = |shift| (s_weight * self.channel(shift) + d_weight * dest.channel(shift) + 1) / 3;
		Self::argb(3, mix(4), mix(2), mix(0))
	}

	/// `Some(true)` for white, `None` for transparent. Matches the firmware's `gcolor_get_bw`.
	const fn to_bw(self) -> Option<bool> {
		if self.alpha() == 0 {
			return None;
		}
		let luminance = (2126 * self.channel(4) as u32
			+ 7152 * self.channel(2) as u32
			+ 722 * self.channel(0) as u32)
			/ 3;
		Some(luminance >= 5000)
	}
}

/// How a drawn bitmap combines with what's under it.
#[derive(Clone, Copy)]
#[repr(u8)]
///
/// Only [`Assign`](Self::Assign) and [`Set`](Self::Set) work on colour bitmaps; the rest are for
/// 1-bit ones.
pub enum CompOp {
	/// Replace what's underneath. Ignores transparency.
	Assign         = sys::GCompOp_GCompOpAssign,
	/// Replace what's underneath with the inverted image.
	AssignInverted = sys::GCompOp_GCompOpAssignInverted,
	/// Paint the image's white pixels; its black pixels are clear.
	Or             = sys::GCompOp_GCompOpOr,
	/// Paint the image's black pixels; its white pixels are clear.
	And            = sys::GCompOp_GCompOpAnd,
	/// Paint black wherever the image is white.
	Clear          = sys::GCompOp_GCompOpClear,
	/// Draw the image with its transparency. On a 1-bit image, paint white wherever it's black.
	Set            = sys::GCompOp_GCompOpSet,
}

/// What text does when it doesn't fit its box.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum TextOverflow {
	WordWrap         = sys::GTextOverflowMode_GTextOverflowModeWordWrap,
	TrailingEllipsis = sys::GTextOverflowMode_GTextOverflowModeTrailingEllipsis,
	/// Like `TrailingEllipsis`, with newlines inside the text treated as spaces.
	Fill             = sys::GTextOverflowMode_GTextOverflowModeFill,
}

bitflags::bitflags! {
	/// Which corners of a rectangle to round.
	#[derive(Clone, Copy)]
	pub struct Corners: u8 {
		const TOP_LEFT     = sys::GCornerMask_GCornerTopLeft;
		const TOP_RIGHT    = sys::GCornerMask_GCornerTopRight;
		const BOTTOM_LEFT  = sys::GCornerMask_GCornerBottomLeft;
		const BOTTOM_RIGHT = sys::GCornerMask_GCornerBottomRight;
	}
}

/// The drawing context of one update proc call, in the layer's own coordinates.
pub struct GContext<'r> {
	ptr:     *mut sys::GContext,
	layer:   *mut sys::Layer,
	_render: PhantomData<&'r mut sys::GContext>,
}

impl GContext<'_> {
	/// # Safety
	/// `ctx` and `layer` must be the arguments of the update proc currently running, and the
	/// result must not outlive that call.
	pub(crate) unsafe fn new(ctx: *mut sys::GContext, layer: *mut sys::Layer) -> Self {
		Self {
			ptr: ctx,
			layer,
			_render: PhantomData,
		}
	}

	/// The raw context.
	pub fn as_ptr(&mut self) -> *mut sys::GContext {
		self.ptr
	}

	/// Set the colour shapes are filled with.
	pub fn set_fill_colour(&mut self, colour: Colour) {
		unsafe { sys::graphics_context_set_fill_color(self.as_ptr(), colour.into()) };
	}

	/// Set the colour of lines and outlines.
	pub fn set_stroke_colour(&mut self, colour: Colour) {
		unsafe { sys::graphics_context_set_stroke_color(self.as_ptr(), colour.into()) };
	}

	/// Set the colour text is drawn in.
	pub fn set_text_colour(&mut self, colour: Colour) {
		unsafe { sys::graphics_context_set_text_color(self.as_ptr(), colour.into()) };
	}

	/// Set the width of lines and outlines.
	pub fn set_stroke_width(&mut self, width: u8) {
		unsafe { sys::graphics_context_set_stroke_width(self.as_ptr(), width) };
	}

	/// Turn antialiasing of lines and shapes on or off.
	pub fn set_antialiased(&mut self, enable: bool) {
		unsafe { sys::graphics_context_set_antialiased(self.as_ptr(), enable) };
	}

	/// Applies to bitmaps drawn with [`draw_bitmap`](Self::draw_bitmap).
	pub fn set_compositing_mode(&mut self, mode: CompOp) {
		unsafe { sys::graphics_context_set_compositing_mode(self.as_ptr(), mode as u8) };
	}

	/// Draw one pixel in the stroke colour.
	pub fn draw_pixel(&mut self, at: Point) {
		unsafe { sys::graphics_draw_pixel(self.as_ptr(), gpoint(at)) };
	}

	/// Draw a line in the stroke colour.
	pub fn draw_line(&mut self, from: Point, to: Point) {
		unsafe { sys::graphics_draw_line(self.as_ptr(), gpoint(from), gpoint(to)) };
	}

	/// Outline `rect` in the stroke colour.
	pub fn draw_rect(&mut self, rect: Rectangle) {
		unsafe { sys::graphics_draw_rect(self.as_ptr(), grect(rect)) };
	}

	/// Fill `rect` with the fill colour.
	pub fn fill_rect(&mut self, rect: Rectangle) {
		self.fill_round_rect(rect, 0, Corners::empty());
	}

	/// Fill `rect` with the fill colour, rounding `corners` by `radius`.
	pub fn fill_round_rect(&mut self, rect: Rectangle, radius: u16, corners: Corners) {
		unsafe { sys::graphics_fill_rect(self.as_ptr(), grect(rect), radius, corners.bits()) };
	}

	/// Outline `rect` in the stroke colour, with all corners rounded by `radius`.
	pub fn draw_round_rect(&mut self, rect: Rectangle, radius: u16) {
		unsafe { sys::graphics_draw_round_rect(self.as_ptr(), grect(rect), radius) };
	}

	/// Outline a circle in the stroke colour.
	pub fn draw_circle(&mut self, centre: Point, radius: u16) {
		unsafe { sys::graphics_draw_circle(self.as_ptr(), gpoint(centre), radius) };
	}

	/// Fill a circle with the fill colour.
	pub fn fill_circle(&mut self, centre: Point, radius: u16) {
		unsafe { sys::graphics_fill_circle(self.as_ptr(), gpoint(centre), radius) };
	}

	/// Draw `bitmap` in `rect`, tiling it if `rect` is larger.
	pub fn draw_bitmap(&mut self, bitmap: &GBitMap, rect: Rectangle) {
		unsafe { sys::graphics_draw_bitmap_in_rect(self.as_ptr(), bitmap.as_ptr(), grect(rect)) };
	}

	/// Draw `image` with its origin at `offset`.
	pub fn draw_pdc(&mut self, image: &DrawCommand, offset: Point) {
		unsafe { sys::gdraw_command_image_draw(self.as_ptr(), image.as_ptr(), gpoint(offset)) };
	}

	/// Draw the frame of `sequence` that is showing `elapsed_ms` into playback. `false` if there is
	/// none, e.g. past the end of a sequence that doesn't loop.
	pub fn draw_pdc_sequence(
		&mut self,
		sequence: &DrawCommandSequence,
		elapsed_ms: u32,
		offset: Point,
	) -> bool {
		unsafe {
			let frame =
				sys::gdraw_command_sequence_get_frame_by_elapsed(sequence.as_ptr(), elapsed_ms);
			if frame.is_null() {
				return false;
			}
			sys::gdraw_command_frame_draw(self.as_ptr(), sequence.as_ptr(), frame, gpoint(offset));
		}
		true
	}

	/// Draw `text` in `rect`.
	pub fn draw_text(
		&mut self,
		text: &CStr,
		font: &Font,
		rect: Rectangle,
		overflow: TextOverflow,
		alignment: TextAlignment,
	) {
		// SAFETY: text and font are borrowed for the call, and the firmware doesn't keep either
		unsafe {
			sys::graphics_draw_text(
				self.as_ptr(),
				text.as_ptr(),
				font.as_ptr(),
				grect(rect),
				overflow as u8,
				alignment as u8,
				core::ptr::null_mut(),
			);
		}
	}

	/// Draw straight into the screen through `f`, e.g. with `embedded-graphics`. `None` if the
	/// framebuffer couldn't be captured.
	///
	/// Pixels are clipped to this layer's frame, but not to its ancestors'.
	pub fn with_framebuffer<R>(&mut self, f: impl FnOnce(&mut Framebuffer<'_>) -> R) -> Option<R> {
		let layer = self.layer;
		// SAFETY: the layer is the one being rendered, so it is alive for this call
		let (origin, bounds, frame) = unsafe {
			(
				point(sys::layer_convert_point_to_screen(layer, sys::GPoint {
					x: 0,
					y: 0,
				})),
				rect(sys::layer_get_bounds(layer)),
				rect(sys::layer_get_frame(layer)),
			)
		};
		let ctx = self.as_ptr();
		// SAFETY: ctx is the live context; released by Framebuffer's Drop before this returns
		let bitmap = unsafe { sys::graphics_capture_frame_buffer(ctx).as_mut()? };
		// SAFETY: the captured bitmap stays valid until released
		let (format, data, stride, screen) = unsafe {
			(
				sys::gbitmap_get_format(bitmap),
				sys::gbitmap_get_data(bitmap),
				usize::from(sys::gbitmap_get_bytes_per_row(bitmap)),
				rect(sys::gbitmap_get_bounds(bitmap)),
			)
		};
		let frame_on_screen = Rectangle::new(origin - bounds.top_left, frame.size);
		let mut fb = Framebuffer {
			ctx,
			bitmap,
			format,
			data,
			stride,
			origin,
			size: bounds.size,
			clip: frame_on_screen.intersection(&screen),
			row: None,
			_ctx: PhantomData,
		};
		Some(f(&mut fb))
	}
}

/// The screen, captured for direct drawing inside [`GContext::with_framebuffer`]. Coordinates are
/// the layer's own, as with [`GContext`].
pub struct Framebuffer<'f> {
	ctx:    *mut sys::GContext,
	bitmap: *mut sys::GBitmap,
	format: sys::GBitmapFormat,
	data:   *mut u8,
	stride: usize,
	origin: Point,
	size:   Size,
	clip:   Rectangle,
	row:    Option<(i32, sys::GBitmapDataRowInfo)>,
	_ctx:   PhantomData<&'f mut GContext<'f>>,
}

impl Framebuffer<'_> {
	/// Data and valid columns of screen row `y`.
	fn row(&mut self, y: i32) -> (*mut u8, i32, i32) {
		if self.format != sys::GBitmapFormat_GBitmapFormat8BitCircular {
			// SAFETY: y is inside the clip, which is inside the screen
			let data = unsafe { self.data.add(y as usize * self.stride) };
			return (data, 0, i32::MAX);
		}
		let info = match self.row {
			Some((cached, info)) if cached == y => info,
			_ => {
				// SAFETY: y is inside the clip, which is inside the bitmap
				let info = unsafe { sys::gbitmap_get_data_row_info(self.bitmap, y as u16) };
				self.row = Some((y, info));
				info
			}
		};
		(info.data, info.min_x.into(), info.max_x.into())
	}

	/// Write `colour` to screen columns `x0..x1` of screen row `y`, already clipped.
	fn span(&mut self, y: i32, x0: i32, x1: i32, colour: Colour) {
		let (data, min_x, max_x) = self.row(y);
		let (x0, x1) = (x0.max(min_x), x1.min(max_x.saturating_add(1)));
		for x in x0..x1 {
			// SAFETY: x is a valid column of this row
			unsafe {
				if self.format == sys::GBitmapFormat_GBitmapFormat1Bit {
					let Some(white) = colour.to_bw() else { return };
					let byte = data.add(x as usize / 8);
					let bit = 1 << (x % 8);
					*byte = if white { *byte | bit } else { *byte & !bit };
				} else {
					let pixel = data.add(x as usize).cast::<Colour>();
					*pixel = colour.blend_over(*pixel);
				}
			}
		}
	}
}

impl OriginDimensions for Framebuffer<'_> {
	fn size(&self) -> Size {
		self.size
	}
}

impl DrawTarget for Framebuffer<'_> {
	type Color = Colour;
	type Error = Infallible;

	fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Infallible>
	where
		I: IntoIterator<Item = Pixel<Colour>>,
	{
		for Pixel(at, colour) in pixels {
			let at = at + self.origin;
			if self.clip.contains(at) {
				self.span(at.y, at.x, at.x + 1, colour);
			}
		}
		Ok(())
	}

	fn fill_solid(&mut self, area: &Rectangle, colour: Colour) -> Result<(), Infallible> {
		let area = Rectangle::new(area.top_left + self.origin, area.size).intersection(&self.clip);
		let Some(bottom_right) = area.bottom_right() else {
			return Ok(());
		};
		for y in area.top_left.y..=bottom_right.y {
			self.span(y, area.top_left.x, bottom_right.x + 1, colour);
		}
		Ok(())
	}
}

impl Drop for Framebuffer<'_> {
	fn drop(&mut self) {
		// SAFETY: captured from this ctx in with_framebuffer, released exactly once
		unsafe { sys::graphics_release_frame_buffer(self.ctx, self.bitmap) };
	}
}
