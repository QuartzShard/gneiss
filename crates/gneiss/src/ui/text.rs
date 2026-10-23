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
		graphics::{Colour, Rectangle, Size, TextOverflow, grect, gsize, size},
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

/// Growable text on the heap, for a [`TextLayer`] whose text changes: edit it in place through
/// [`TextLayer::text_mut`], e.g. with [`pbl_fmt_write!`](crate::pbl_fmt_write). NUL bytes are dropped.
///
/// For text built and used straight away, such as a menu row formatted while it's drawn,
/// [`StackText`] needs no allocation.
#[derive(Default)]
pub struct HeapText {
	// empty, or NUL-terminated with no interior NUL
	bytes: Vec<u8>,
}

impl HeapText {
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
		self.push_bytes(s.as_bytes());
	}

	/// Append `bytes`, dropping any NUL bytes.
	pub(crate) fn push_bytes(&mut self, bytes: &[u8]) {
		self.bytes.pop();
		self.bytes.reserve(bytes.len() + 1);
		self.bytes.extend(bytes.iter().filter(|&&b| b != 0));
		self.bytes.push(0);
	}

	/// Append what `fill` writes into the spare buffer it's given, as C text, `snprintf` style:
	/// `fill` returns the length it needed, and is called again with a buffer that fits if the
	/// first was too small. False if `fill` failed.
	pub(crate) fn append_c(&mut self, fill: impl Fn(&mut [u8]) -> i32) -> bool {
		let start = self.bytes.len().saturating_sub(1);
		let room = (self.bytes.capacity() - start).max(16);
		self.bytes.resize(start + room, 0);
		let Ok(needed) = usize::try_from(fill(&mut self.bytes[start..])) else {
			self.bytes.truncate(start);
			self.bytes.push(0);
			return false;
		};
		if needed >= room {
			self.bytes.resize(start + needed + 1, 0);
			fill(&mut self.bytes[start..]);
		}
		let len = start + terminate(&mut self.bytes[start..=start + needed]);
		self.bytes.truncate(len + 1);
		true
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

impl Deref for HeapText {
	type Target = CStr;

	fn deref(&self) -> &CStr {
		self.as_c_str()
	}
}

impl fmt::Write for HeapText {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		self.push_str(s);
		Ok(())
	}
}

impl From<&str> for HeapText {
	fn from(s: &str) -> Self {
		let mut buf = Self::with_capacity(s.len());
		buf.push_str(s);
		buf
	}
}

impl From<String> for HeapText {
	/// Reuses the string's allocation.
	fn from(s: String) -> Self {
		let mut bytes = s.into_bytes();
		bytes.retain(|&b| b != 0);
		bytes.push(0);
		Self { bytes }
	}
}

/// Text of up to `N` bytes, held inline, so it needs no allocation: for text built and used
/// straight away, such as a menu row formatted while it's drawn. Write to it with
/// [`pbl_fmt_write!`](crate::pbl_fmt_write).
///
/// Text that doesn't fit is cut at a character boundary, and the write returns an error. NUL bytes
/// are dropped.
#[repr(C)]
pub struct StackText<const N: usize> {
	// bytes and nul are contiguous, as N + 1 bytes: the text, then its NUL
	bytes: [u8; N],
	nul:   u8,
	len:   usize,
}

impl<const N: usize> StackText<N> {
	/// Empty text.
	pub const fn new() -> Self {
		Self {
			bytes: [0; N],
			nul:   0,
			len:   0,
		}
	}

	/// The text and NUL bytes, as one slice of N + 1.
	fn buf(&self) -> &[u8] {
		// SAFETY: repr(C) puts bytes at offset 0 and nul straight after it, with no padding as both
		// are bytes
		unsafe { core::slice::from_raw_parts((&raw const *self).cast(), N + 1) }
	}

	/// As for `buf`.
	fn buf_mut(&mut self) -> &mut [u8] {
		// SAFETY: as for buf
		unsafe { core::slice::from_raw_parts_mut((&raw mut *self).cast(), N + 1) }
	}

	/// Empty the text.
	pub fn clear(&mut self) {
		self.len = 0;
		self.buf_mut()[0] = 0;
	}

	/// Append as much of `s` as fits, dropping any NUL bytes. An error if some didn't fit.
	pub fn push_str(&mut self, s: &str) -> fmt::Result {
		let mut len = self.len;
		let result = push_inline(self.buf_mut(), &mut len, s);
		self.len = len;
		result
	}

	/// Replace the text with what `fill` writes into the whole buffer, its N + 1 bytes, as C text:
	/// the text ends at the first NUL, or is cut to N bytes without one.
	pub(crate) fn fill_c(&mut self, fill: impl FnOnce(&mut [u8])) {
		fill(self.buf_mut());
		self.len = terminate(self.buf_mut());
	}

	/// Append what `fill` writes into the spare buffer it's given, as C text, `snprintf` style:
	/// `fill` returns the length it needed. False if that didn't fit, cutting it short, or `fill`
	/// failed.
	pub(crate) fn append_c(&mut self, fill: impl FnOnce(&mut [u8]) -> i32) -> bool {
		let start = self.len;
		let needed = fill(&mut self.buf_mut()[start..]);
		self.len = start + terminate(&mut self.buf_mut()[start..]);
		usize::try_from(needed).is_ok_and(|needed| start + needed == self.len)
	}

	/// The text, NUL-terminated.
	pub fn as_c_str(&self) -> &CStr {
		// SAFETY: buf[len] is the only NUL in buf[..=len]
		unsafe { CStr::from_bytes_with_nul_unchecked(&self.buf()[..=self.len]) }
	}
}

// Not generic over N, so every StackText size shares it. The length of the C text in `buf`, after
// NUL-terminating its last byte
fn terminate(buf: &mut [u8]) -> usize {
	let last = buf.len() - 1;
	buf[last] = 0;
	buf.iter().position(|&b| b == 0).unwrap_or(last)
}

// Not generic over N, so every StackText size shares it
fn push_inline(bytes: &mut [u8], len: &mut usize, s: &str) -> fmt::Result {
	let mut end = s.len().min(bytes.len() - 1 - *len);
	while !s.is_char_boundary(end) {
		end -= 1;
	}
	for &b in &s.as_bytes()[..end] {
		if b != 0 {
			bytes[*len] = b;
			*len += 1;
		}
	}
	bytes[*len] = 0;
	if end == s.len() {
		Ok(())
	} else {
		Err(fmt::Error)
	}
}

impl<const N: usize> Default for StackText<N> {
	fn default() -> Self {
		Self::new()
	}
}

impl<const N: usize> Deref for StackText<N> {
	type Target = CStr;

	fn deref(&self) -> &CStr {
		self.as_c_str()
	}
}

impl<const N: usize> fmt::Write for StackText<N> {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		self.push_str(s)
	}
}

/// A layer that draws text it owns, in a font it holds. The firmware keeps pointers to both and
/// reads them at render time; the text is boxed so moving the layer doesn't move it.
///
/// `F` holds the font: a [`Font`], or `&'static Font` for one that lives as long as the app, or an
/// `Rc<Font>` to share one between windows that come and go. [`replace_font`](Self::replace_font)
/// hands the old font back, to swap between fonts without loading them again.
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

	/// Draw the text in `font`, dropping the old one.
	pub fn set_font(&mut self, font: F) -> &mut Self {
		self.replace_font(font);
		self
	}

	/// Draw the text in `font`. Returns the old font, if it had one, which the layer no longer
	/// points at.
	pub fn replace_font(&mut self, font: F) -> Option<F> {
		// SAFETY: the font lives as long as the Font, which self.font keeps alive; moving the
		// handle doesn't move it
		unsafe { sys::text_layer_set_font(self.ptr, font.borrow().as_ptr()) };
		self.font.replace(font)
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

	/// What happens to text that doesn't fit the layer.
	pub fn set_overflow(&mut self, overflow: TextOverflow) -> &mut Self {
		unsafe { sys::text_layer_set_overflow_mode(self.ptr, overflow as u8) };
		self
	}

	/// Resize the layer, keeping its origin.
	pub fn set_size(&mut self, size: Size) -> &mut Self {
		unsafe { sys::text_layer_set_size(self.ptr, gsize(size)) };
		self
	}

	/// The size the current text takes up when rendered.
	pub fn content_size(&self) -> Size {
		size(unsafe { sys::text_layer_get_content_size(self.ptr) })
	}

	/// Wrap lines along the edge of a round screen, `inset` pixels in from it. Inside a
	/// [`ScrollLayer`](super::ScrollLayer), it also lays the text out in pages, for
	/// [paged](super::ScrollLayer::set_paging) scrolling.
	///
	/// By convention, text flow and paging go together on round screens only: use
	/// `#[cfg(pbl_round)]`. A [`ScrollWindow`](super::ScrollWindow) already pages there.
	///
	/// Call it after adding the layer to a window. Before that, it does nothing. Measure
	/// [`content_size`](Self::content_size) afterwards.
	pub fn enable_screen_text_flow(&mut self, inset: u8) -> &mut Self {
		unsafe { sys::text_layer_enable_screen_text_flow_and_paging(self.ptr, inset) };
		self
	}

	/// Wrap lines to the layer again, undoing
	/// [`enable_screen_text_flow`](Self::enable_screen_text_flow).
	pub fn restore_default_text_flow(&mut self) -> &mut Self {
		unsafe { sys::text_layer_restore_default_text_flow_and_paging(self.ptr) };
		self
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
