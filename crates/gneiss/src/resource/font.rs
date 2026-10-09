//! Fonts: the firmware's system fonts, and font resources.
use core::ffi::CStr;

use crate::{resource::FontKey, sys};

/// A font, for a [`TextLayer`](crate::ui::TextLayer) or
/// [`GContext::draw_text`](crate::ui::graphics::GContext::draw_text). A font resource is unloaded
/// when its `Font` drops; system fonts belong to the firmware, so dropping one does nothing.
pub struct Font {
	ptr:    sys::GFont,
	custom: bool,
}

impl Font {
	/// The system font named by `key`, one of the `sys::FONT_KEY_*` constants. The firmware falls
	/// back to a default font for a key it doesn't know.
	pub fn system(key: &CStr) -> Self {
		Self {
			// SAFETY: the firmware only reads key, and returns a font it owns
			ptr:    unsafe { sys::fonts_get_system_font(key.as_ptr()) },
			custom: false,
		}
	}

	/// Load the font resource `key`. If it can't be loaded, the firmware logs a warning and hands
	/// back its Gothic 14 system font instead; there is no error to check.
	pub fn load(key: FontKey) -> Self {
		// SAFETY: key names one of this app's resources
		let ptr = unsafe { sys::fonts_load_custom_font(sys::resource_get_handle(key.0.get())) };
		Self { ptr, custom: true }
	}

	/// The raw font, valid while this `Font` lives.
	pub fn as_ptr(&self) -> sys::GFont {
		self.ptr
	}
}

impl Drop for Font {
	fn drop(&mut self) {
		if self.custom {
			// SAFETY: loaded by fonts_load_custom_font and unloaded once. If loading fell back to
			// Gothic 14, the firmware ignores the unload
			unsafe { sys::fonts_unload_custom_font(self.ptr) };
		}
	}
}
