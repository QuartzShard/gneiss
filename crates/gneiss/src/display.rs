//! A multi-line status board: one `String` line per slot, painted into a
//! `TextLayer`. Held in a `static` (hence the single-task `unsafe impl Sync`) so
//! handlers can reach it. Needs `alloc`.
//
// TODO: port to the safe layer/window types once they exist; on raw
// `sys::TextLayer` for now.
extern crate alloc;

use alloc::{ffi::CString, string::String};
use core::cell::UnsafeCell;

use crate::sys;

/// `SLOTS` text lines painted into one `TextLayer`. Hold it in a `static`,
/// [`attach`](Self::attach) a layer, then [`set`](Self::set) slots.
pub struct SimpleTextDisplay<const SLOTS: usize> {
	inner: UnsafeCell<Inner<SLOTS>>,
}

struct Inner<const SLOTS: usize> {
	layer:   *mut sys::TextLayer,
	lines:   [Option<String>; SLOTS],
	// Keeps the displayed C string alive; the layer stores the pointer, not a copy.
	current: Option<CString>,
}

// SAFETY: only ever touched from the single cooperative app task; handlers run
// one at a time in the event loop and never reenter, so nothing accesses the
// `UnsafeCell` concurrently or overlapping.
unsafe impl<const SLOTS: usize> Sync for SimpleTextDisplay<SLOTS> {}

impl<const SLOTS: usize> SimpleTextDisplay<SLOTS> {
	pub const fn new() -> Self {
		Self {
			inner: UnsafeCell::new(Inner {
				layer:   core::ptr::null_mut(),
				lines:   [const { None }; SLOTS],
				current: None,
			}),
		}
	}

	/// Point the display at a `TextLayer`. Until this is called, [`set`](Self::set)
	/// is a no-op.
	pub fn attach(&self, layer: *mut sys::TextLayer) {
		// SAFETY: single-task, non-reentrant; see the type's Sync note.
		unsafe { &mut *self.inner.get() }.layer = layer;
	}

	/// Replace slot `slot`'s line and repaint. Panics if `slot >= SLOTS`.
	pub fn set(&self, slot: usize, line: impl Into<String>) {
		// SAFETY: single-task, non-reentrant; see the type's Sync note.
		let inner = unsafe { &mut *self.inner.get() };
		if inner.layer.is_null() {
			return;
		}
		inner.lines[slot] = Some(line.into());
		let mut text = String::new();
		for line in inner.lines.iter().flatten() {
			text.push_str(line);
			text.push('\n');
		}
		if let Ok(cs) = CString::new(text) {
			// Point the layer at the new buffer before dropping the old one.
			// SAFETY: valid layer; `cs` outlives this (moved into `current`).
			unsafe { sys::text_layer_set_text(inner.layer, cs.as_ptr()) };
			inner.current = Some(cs);
		}
	}
}

impl<const SLOTS: usize> Default for SimpleTextDisplay<SLOTS> {
	fn default() -> Self {
		Self::new()
	}
}
