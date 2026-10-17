//! Vibration motor control
//!
//! Each call is ignored while a vibration is already playing. Vibrating doesn't check Quiet Time,
//! the user's do-not-disturb setting; an app can:
//!
//! ```no_run
//! # fn main() {
//! use gneiss::{system, vibes};
//!
//! if !system::quiet_time_active() {
//!     vibes::double_pulse();
//! }
//! # }
//! ```
use crate::sys;

pub fn short_pulse() {
	unsafe { sys::vibes_short_pulse() }
}

pub fn long_pulse() {
	unsafe { sys::vibes_long_pulse() }
}

pub fn double_pulse() {
	unsafe { sys::vibes_double_pulse() }
}

pub fn cancel() {
	unsafe { sys::vibes_cancel() }
}

/// Play `segments`: durations in milliseconds, alternating vibrating and pausing, starting with a
/// vibration. The firmware caps each at 10 000. It copies the pattern before this returns, so
/// `segments` can be a temporary.
///
/// ```no_run
/// # fn main() {
/// // On for 200 ms, off for 100, on for 400
/// gneiss::vibes::play(&[200, 100, 400]);
/// # }
/// ```
pub fn play(segments: &[u32]) {
	let pattern = sys::VibePattern {
		durations:    segments.as_ptr(),
		num_segments: segments.len() as u32,
	};
	// SAFETY: durations points to num_segments values, read before this returns
	unsafe { sys::vibes_enqueue_custom_pattern(pattern) }
}
