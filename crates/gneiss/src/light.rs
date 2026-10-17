//! The backlight.
use crate::sys;
#[cfg(pbl_rgb_backlight)]
use crate::ui::graphics::Colour;

/// Light the backlight for a moment, turning it off again after the user's usual delay.
/// The way to light the screen briefly, such as for an alert.
pub fn interaction() {
	unsafe { sys::light_enable_interaction() }
}

/// Keep the backlight on, or with `false`, hand it back to the system. Kept on, it drains the
/// battery quickly.
pub fn force_on(on: bool) {
	unsafe { sys::light_enable(on) }
}

/// Whether the backlight is lit at all, including while it fades out.
#[cfg(not(pbl_sdk_frozen))]
pub fn is_on() -> bool {
	unsafe { sys::light_is_on() }
}

/// Tint the backlight `colour` until the app exits or [`reset_colour`].
#[cfg(pbl_rgb_backlight)]
pub fn set_colour(colour: Colour) {
	unsafe { sys::light_set_color(colour.into()) }
}

/// Tint the backlight a 24-bit `0xRRGGBB` colour, finer than a [`Colour`], until the app exits or
/// [`reset_colour`].
#[cfg(pbl_rgb_backlight)]
pub fn set_colour_rgb8(rgb: u32) {
	unsafe { sys::light_set_color_rgb888(rgb) }
}

/// Put the backlight back to the user's colour.
#[cfg(pbl_rgb_backlight)]
pub fn reset_colour() {
	unsafe { sys::light_set_system_color() }
}
