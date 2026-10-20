//! The status bar: the clock along the top of the screen.
use crate::{
	sys,
	ui::{AsLayer, graphics::Colour},
};

/// The status bar's height, separator included, on this platform.
pub const STATUS_BAR_HEIGHT: u32 = sys::STATUS_BAR_LAYER_HEIGHT;

/// The line along the bottom of a status bar.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Separator {
	None   = sys::StatusBarLayerSeparatorMode_StatusBarLayerSeparatorModeNone,
	Dotted = sys::StatusBarLayerSeparatorMode_StatusBarLayerSeparatorModeDotted,
}

/// A layer showing the time, which the firmware keeps current. It starts at the top of its parent,
/// screen-wide and [`STATUS_BAR_HEIGHT`] tall; [`Layer::set_frame`](crate::ui::Layer::set_frame)
/// moves or narrows it, such as to make room for an action bar.
pub struct StatusBarLayer {
	ptr: *mut sys::StatusBarLayer,
}

impl StatusBarLayer {
	/// Create a status bar. `None` if the firmware is out of memory.
	pub fn new() -> Option<Self> {
		// SAFETY: no preconditions; NULL on OOM
		let ptr = unsafe { sys::status_bar_layer_create().as_mut()? };
		Some(Self { ptr })
	}

	/// Set the colours behind and of the time.
	pub fn set_colours(&mut self, background: Colour, foreground: Colour) -> &mut Self {
		unsafe { sys::status_bar_layer_set_colors(self.ptr, background.into(), foreground.into()) };
		self
	}

	/// The colour behind the time.
	pub fn background_colour(&self) -> Colour {
		unsafe { sys::status_bar_layer_get_background_color(self.ptr) }.into()
	}

	/// The colour of the time.
	pub fn foreground_colour(&self) -> Colour {
		unsafe { sys::status_bar_layer_get_foreground_color(self.ptr) }.into()
	}

	/// Set the line along the bottom.
	pub fn set_separator(&mut self, separator: Separator) -> &mut Self {
		unsafe { sys::status_bar_layer_set_separator_mode(self.ptr, separator as u8) };
		self
	}
}

unsafe impl AsLayer for StatusBarLayer {
	fn as_ptr(&self) -> *mut sys::Layer {
		unsafe { sys::status_bar_layer_get_layer(self.ptr) }
	}
}

impl Drop for StatusBarLayer {
	fn drop(&mut self) {
		unsafe {
			sys::layer_remove_child_layers(self.as_ptr());
			sys::status_bar_layer_destroy(self.ptr);
		}
	}
}
