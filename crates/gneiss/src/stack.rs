//! With `trace-stack`, logs the deepest the app stack went, when the app exits.
//!
//! The firmware puts the stack below the app image, and fills it with `0xA5` when it starts the
//! app. At exit, the lowest word that no longer holds the fill marks the deepest the stack went.

#[cfg(feature = "trace-stack")]
mod imp {
	use crate::sys;

	// The firmware reserves the bottom 32 B as a guard
	const STACK_SIZE: usize = if cfg!(any(feature = "emery", feature = "gabbro")) {
		4096 - 32
	} else {
		2048 - 32
	};
	const FILL: u32 = 0xA5A5_A5A5;
	/// How far below the header the stack ends: aplite's firmware puts its 6820 B of state for the
	/// app between the two.
	const BELOW_HEADER: usize = if cfg!(feature = "aplite") { 6820 } else { 0 };

	#[inline(never)]
	pub fn report() {
		let top: *const u32 =
			core::ptr::with_exposed_provenance((&raw const sys::PBL_HEADER).addr() - BELOW_HEADER);
		let mut word = top.wrapping_byte_sub(STACK_SIZE);
		// SAFETY: every word of the stack is app RAM
		while word < top && unsafe { word.read_volatile() } == FILL {
			word = word.wrapping_add(1);
		}
		let used = top.addr() - word.addr();
		crate::log::info!("stack peak {}/{}", used as u32, STACK_SIZE as u32);
	}
}

#[cfg(feature = "trace-stack")]
pub use imp::report;

#[cfg(not(feature = "trace-stack"))]
#[inline(always)]
pub fn report() {}
