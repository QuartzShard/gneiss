//! Text layers, and growable text for them to own.
extern crate alloc;
use alloc::{boxed::Box, string::String, vec::Vec};
use core::{borrow::Borrow, ffi::CStr, fmt, ops::Deref};

use crate::{
	alloc::PebbleAlloc,
	resource::font::Font,
	sys,
	ui::{
		AsLayer,
		graphics::{Colour, Rectangle, Size, grect, size},
	},
};

/// Horizontal alignment of text in its box.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum TextAlignment {
	Left   = sys::GTextAlignment_GTextAlignmentLeft,
	Center = sys::GTextAlignment_GTextAlignmentCenter,
	Right  = sys::GTextAlignment_GTextAlignmentRight,
}

/// Growable text for a [`TextLayer`] whose text changes: edit it in place through
/// [`TextLayer::text_mut`], e.g. with `write!`. NUL bytes are dropped. With
/// `-Cpanic=immediate-abort` it is also a `ufmt::uWrite`, for `uwrite!` without `core::fmt`.
#[derive(Default)]
pub struct TextBuf {
	// empty, or NUL-terminated with no interior NUL
	bytes: Vec<u8>,
}

impl TextBuf {
	/// Empty text. Allocates nothing until written to.
	pub const fn new() -> Self {
		Self { bytes: Vec::new() }
	}

	/// Empty text with room for `capacity` bytes.
	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			bytes: Vec::with_capacity(capacity + 1),
		}
	}

	/// Empty the text, keeping the allocation.
	pub fn clear(&mut self) {
		self.bytes.clear();
	}

	/// Append `s`, dropping any NUL bytes.
	#[inline(never)]
	pub fn push_str(&mut self, s: &str) {
		self.bytes.pop();
		self.bytes.reserve(s.len() + 1);
		self.bytes.extend(s.bytes().filter(|&b| b != 0));
		self.bytes.push(0);
	}

	/// The text, NUL-terminated.
	pub fn as_c_str(&self) -> &CStr {
		if self.bytes.is_empty() {
			c""
		} else {
			// SAFETY: non-empty bytes end in the only NUL
			unsafe { CStr::from_bytes_with_nul_unchecked(&self.bytes) }
		}
	}
}

impl Deref for TextBuf {
	type Target = CStr;

	fn deref(&self) -> &CStr {
		self.as_c_str()
	}
}

impl fmt::Write for TextBuf {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		self.push_str(s);
		Ok(())
	}
}

#[cfg(panic = "immediate-abort")]
impl ufmt::uWrite for TextBuf {
	type Error = core::convert::Infallible;

	fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
		self.push_str(s);
		Ok(())
	}
}

impl From<&str> for TextBuf {
	fn from(s: &str) -> Self {
		let mut buf = Self::with_capacity(s.len());
		buf.push_str(s);
		buf
	}
}

impl From<String> for TextBuf {
	/// Reuses the string's allocation.
	fn from(s: String) -> Self {
		let mut bytes = s.into_bytes();
		bytes.retain(|&b| b != 0);
		bytes.push(0);
		Self { bytes }
	}
}

/// A layer that draws text it owns, in a font it holds. The firmware keeps pointers to both and
/// reads them at render time; the text is boxed so moving the layer doesn't move it.
///
/// `F` holds the font: a [`Font`], or e.g. an `Rc<Font>` to share one font resource between
/// layers.
pub struct TextLayer<S: Deref<Target = CStr> = &'static CStr, F: Borrow<Font> = Font> {
	ptr:  *mut sys::TextLayer,
	text: Option<Box<S, PebbleAlloc>>,
	font: Option<F>,
}

impl<S: Deref<Target = CStr>, F: Borrow<Font>> TextLayer<S, F> {
	/// Create a text layer with no text. `None` if the firmware is out of memory.
	pub fn new(frame: Rectangle) -> Option<Self> {
		// SAFETY: no preconditions; NULL on OOM
		let ptr = unsafe { sys::text_layer_create(grect(frame)).as_mut()? };
		Some(Self {
			ptr,
			text: None,
			font: None,
		})
	}

	/// Replace the text.
	pub fn set_text(&mut self, text: impl Into<S>) -> &mut Self {
		let text = text.into();
		match &mut self.text {
			Some(boxed) => **boxed = text,
			None => self.text = Some(Box::new_in(text, PebbleAlloc)),
		}
		self.repoint();
		self
	}

	/// The current text, if any.
	pub fn text(&self) -> Option<&CStr> {
		self.text.as_deref().map(Deref::deref)
	}

	/// Hand the firmware the text's current address, and mark the layer dirty.
	fn repoint(&mut self) {
		if let Some(text) = self.text.as_deref() {
			// SAFETY: the boxed S is not moved or mutated again without another repoint, and is
			// dropped only after the layer is destroyed. Nothing renders between a change and this
			// call
			unsafe { sys::text_layer_set_text(self.ptr, text.as_ptr()) };
		}
	}

	/// Draw the text in `font`, dropping the old font once the layer no longer points at it.
	pub fn set_font(&mut self, font: F) -> &mut Self {
		// SAFETY: the font lives as long as the Font, which self.font keeps alive; moving the
		// handle doesn't move it
		unsafe { sys::text_layer_set_font(self.ptr, font.borrow().as_ptr()) };
		self.font = Some(font);
		self
	}

	/// Set the text's colour.
	pub fn set_text_colour(&mut self, colour: Colour) -> &mut Self {
		unsafe { sys::text_layer_set_text_color(self.ptr, colour.into()) };
		self
	}

	/// Set the colour behind the text.
	pub fn set_background_colour(&mut self, colour: Colour) -> &mut Self {
		unsafe { sys::text_layer_set_background_color(self.ptr, colour.into()) };
		self
	}

	/// Set how lines align within the layer.
	pub fn set_alignment(&mut self, alignment: TextAlignment) -> &mut Self {
		unsafe { sys::text_layer_set_text_alignment(self.ptr, alignment as u8) };
		self
	}

	/// The size the current text takes up when rendered.
	pub fn content_size(&self) -> Size {
		size(unsafe { sys::text_layer_get_content_size(self.ptr) })
	}
}

impl<S: Deref<Target = CStr> + Default, F: Borrow<Font>> TextLayer<S, F> {
	/// Edit the text in place, starting from `S::default()` if no text has been set.
	pub fn text_mut<R>(&mut self, f: impl FnOnce(&mut S) -> R) -> R {
		let text = self
			.text
			.get_or_insert_with(|| Box::new_in(S::default(), PebbleAlloc));
		let result = f(text);
		self.repoint();
		result
	}
}
unsafe impl<S: Deref<Target = CStr>, F: Borrow<Font>> AsLayer for TextLayer<S, F> {
	fn as_ptr(&self) -> *mut sys::Layer {
		unsafe { sys::text_layer_get_layer(self.ptr) }
	}
}

impl<S: Deref<Target = CStr>, F: Borrow<Font>> Drop for TextLayer<S, F> {
	fn drop(&mut self) {
		// SAFETY: destroyed before the text and font fields drop, so the firmware never reads a
		// freed one
		unsafe {
			sys::layer_remove_child_layers(self.as_ptr());
			sys::text_layer_destroy(self.ptr);
		}
	}
}
