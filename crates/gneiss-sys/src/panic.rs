//! Panic handlers

use core::panic::PanicInfo;

/// End the app: the firmware closes it on the undefined instruction. Prints nothing, so it's the
/// handler for `-Cpanic=immediate-abort`, where core never calls the handler at all and only needs
/// one to link.
pub fn panic_halt(_: &PanicInfo) -> ! {
	halt()
}

/// End the app, as [`panic_halt`] does.
pub fn halt() -> ! {
	// SAFETY: UnDeFined instruction makes the OS Kill Us, so it's effectively `std::process::exit()`
	unsafe {
		core::arch::asm!("udf 0", options(noreturn));
	};
}
