//! Panic handlers

use core::{
	ffi::{CStr, c_int},
	fmt::{self, Write},
	panic::PanicInfo,
};

use crate::app_log;

/// Bytes of message the firmware keeps per log line
const LOG_MSG_LEN: usize = 88;

/// "Handle" by calling UDF, causing the OS to kill us and clean us up.
/// Useful final step for better panics, but maxially minimal, potentialy a tiny binary size win for
/// release builds? Don't use this.
pub fn panic_halt(_: &PanicInfo) -> ! {
	// SAFETY: UnDeFined instruction makes the OS Kill Us, so it's effectively `std::process::exit()`
	unsafe {
		core::arch::asm!("udf 0", options(noreturn));
	};
}

fn location<'a>(info: &'a PanicInfo) -> (&'a CStr, c_int) {
	info.location().map_or((c"unknown_file", 0), |loc| {
		(loc.file_as_c_str(), loc.line() as c_int)
	})
}

struct LogBuf {
	buf: [u8; LOG_MSG_LEN],
	len: usize,
}

impl Write for LogBuf {
	#[inline(never)]
	fn write_str(&mut self, s: &str) -> fmt::Result {
		let free = &mut self.buf[self.len.min(LOG_MSG_LEN)..];
		let n = s.len().min(free.len());
		free[..n].copy_from_slice(&s.as_bytes()[..n]);
		self.len += n;
		Ok(())
	}
}

/// Log "Panic! <message>" at the panic's `file:line`, formatting the message, and halt. The
/// default for `#[gneiss::entry]`. The firmware shows the last 15 characters of the file's name,
/// and messages are cut at 88 bytes.
pub fn panic_fmt(info: &PanicInfo) -> ! {
	let (src_filename, src_line_number) = location(info);

	let mut msg = LogBuf {
		buf: [0; LOG_MSG_LEN],
		len: 0,
	};
	let _ = write!(msg, "{}", info.message());

	// SAFETY: `%.*s` reads exactly `msg.len` bytes, all within `msg.buf`
	unsafe {
		app_log(
			crate::sys_raw::AppLogLevel_APP_LOG_LEVEL_ERROR,
			src_filename.as_ptr(),
			src_line_number,
			c"Panic! %.*s".as_ptr(),
			msg.len as c_int,
			msg.buf.as_ptr(),
		);
	};
	panic_halt(info)
}
