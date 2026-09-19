//! Panic handlers

use core::panic::PanicInfo;

use crate::app_log;

/// "Handle" by calling UDF, causing the OS to kill us and clean us up.
/// Useful final step for better panics, but maxially minimal, potentialy a tiny binary size win for
/// release builds? Don't use this.
pub fn panic_halt(_: &PanicInfo) -> ! {
	// SAFETY: UnDeFined instruction makes the OS Kill Us, so it's effectively `std::process::exit()`
	unsafe {
		core::arch::asm!("udf 0", options(noreturn));
	};
}

/// Log "Panic at `file:line`\n <message>", and halt. Does not format, so won't display panic messages with arguments.
/// Use this if you're avoiding `core::fmt` and the binary bloat it brings.
pub fn panic_minimal(info: &PanicInfo) -> ! {
	let (src_filename, src_line_number) = info.location().map_or((c"unknown_file", 0), |loc| {
		(loc.file_as_c_str(), loc.line() as core::ffi::c_int)
	});

	// No-fmt case: truncate and null-terminate.
	let fmt_msg = c"<message required fmt>";
	let mut buf = [0; 88];
	let msg_buf: *const core::ffi::c_char = if let Some(message) = info.message().as_str() {
		let mut message = message.as_bytes();
		let buf_view = &mut buf[..86];
		if message.len() > 86 {
			message = &message[..86];
		}
		buf_view[..message.len()].copy_from_slice(message);
		buf.as_ptr().cast()
	} else {
		fmt_msg.as_ptr()
	};

	unsafe {
		app_log(
			crate::sys_raw::AppLogLevel_APP_LOG_LEVEL_ERROR,
			src_filename.as_ptr(),
			src_line_number,
			c"Panic! %s".as_ptr(),
			msg_buf,
		);
	};
	panic_halt(info)
}

// Full-fat panic handler. Prints formatted message(s), file:line, and halts. If in doubt, or in
// dev, use this.
// TODO:: Implement this
//
//pub fn panic_fmt(info: &PanicInfo) -> ! {
//	panic_halt(info)
//}
