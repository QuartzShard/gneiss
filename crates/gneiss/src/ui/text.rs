//! Text layers, and growable text for them to own.
extern crate alloc;
use alloc::{boxed::Box, string::String, vec::Vec};
use core::{ffi::CStr, fmt, ops::Deref};

use crate::{
	alloc::PebbleAlloc,
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
	pub fn push_str(&mut self, s: &str) {
		self.bytes.pop();
		for part in s.as_bytes().split(|&b| b == 0) {
			self.bytes.extend_from_slice(part);
		}
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

/// A layer that draws text it owns. The firmware keeps a pointer to the text and reads it at render
/// time; the text is boxed so moving the layer doesn't move it.
pub struct TextLayer<S: Deref<Target = CStr> = &'static CStr> {
	ptr:  *mut sys::TextLayer,
	text: Option<Box<S, PebbleAlloc>>,
}

impl<S: Deref<Target = CStr>> TextLayer<S> {
	/// Create a text layer with no text. `None` if the firmware is out of memory.
	pub fn new(frame: Rectangle) -> Option<Self> {
		// SAFETY: no preconditions; NULL on OOM
		let ptr = unsafe { sys::text_layer_create(grect(frame)).as_mut()? };
		Some(Self { ptr, text: None })
	}

	/// Replace the text.
	pub fn set_text(&mut self, text: impl Into<S>) {
		let text = text.into();
		match &mut self.text {
			Some(boxed) => **boxed = text,
			None => self.text = Some(Box::new_in(text, PebbleAlloc)),
		}
		self.repoint();
	}

	/// Edit the text in place. `None` if no text has been set.
	pub fn text_mut<R>(&mut self, f: impl FnOnce(&mut S) -> R) -> Option<R> {
		let result = f(self.text.as_deref_mut()?);
		self.repoint();
		Some(result)
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

	/// Set the font.
	///
	/// # Safety
	/// `font` must stay loaded for as long as this layer exists.
	pub unsafe fn set_font(&mut self, font: sys::GFont) {
		unsafe { sys::text_layer_set_font(self.ptr, font) };
	}

	/// Set the text's colour.
	pub fn set_text_color(&mut self, colour: Colour) {
		unsafe { sys::text_layer_set_text_color(self.ptr, colour.into()) };
	}

	/// Set the colour behind the text.
	pub fn set_background_color(&mut self, colour: Colour) {
		unsafe { sys::text_layer_set_background_color(self.ptr, colour.into()) };
	}

	/// Set how lines align within the layer.
	pub fn set_alignment(&mut self, alignment: TextAlignment) {
		unsafe { sys::text_layer_set_text_alignment(self.ptr, alignment as u8) };
	}

	/// The size the current text takes up when rendered.
	pub fn content_size(&self) -> Size {
		size(unsafe { sys::text_layer_get_content_size(self.ptr) })
	}
}

unsafe impl<S: Deref<Target = CStr>> AsLayer for TextLayer<S> {
	fn as_ptr(&self) -> *mut sys::Layer {
		unsafe { sys::text_layer_get_layer(self.ptr) }
	}
}

impl<S: Deref<Target = CStr>> Drop for TextLayer<S> {
	fn drop(&mut self) {
		// SAFETY: destroyed before the text field drops, so the firmware never reads freed text
		unsafe {
			sys::layer_remove_child_layers(self.as_ptr());
			sys::text_layer_destroy(self.ptr);
		}
	}
}
