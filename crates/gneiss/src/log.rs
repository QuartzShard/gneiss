use crate::sys;

#[repr(u8)]
pub enum LogLevel {
	Info    = sys::AppLogLevel_APP_LOG_LEVEL_INFO as u8,
	Warn    = sys::AppLogLevel_APP_LOG_LEVEL_WARNING as u8,
	Error   = sys::AppLogLevel_APP_LOG_LEVEL_ERROR as u8,
	Debug   = sys::AppLogLevel_APP_LOG_LEVEL_DEBUG as u8,
	Verbose = sys::AppLogLevel_APP_LOG_LEVEL_DEBUG_VERBOSE as u8,
}

#[doc(hidden)]
pub const fn __macro_check_log_level(l: LogLevel) -> u8 {
	l as u8
}

/// Log a `&str` at an explicit [`LogLevel`]. Binds `$msg` as a `&str` (one eval,
/// and a type gate), then `%.*s` performs a bounded read so no nul terminator is
/// needed. `file!()`/`line!()` resolve to the call site, and the `app_log` call
/// is sound, so it wraps its own `unsafe`.
#[macro_export]
macro_rules! log {
	($level:expr, $msg:expr $(,)?) => {{
		let msg: &str = $msg;
		unsafe {
			$crate::sys::app_log(
				$crate::log::__macro_check_log_level($level),
				// const concat a nul-byte onto the str == valid cstr literal
				concat!(file!(), "\0").as_ptr(),
				line!() as i32,
				// %.*s takes a len and performs a bounded read, no nul-terminal needed
				c"%.*s".as_ptr(),
				msg.len() as i32,
				msg.as_ptr(),
			);
		}
	}};
}
pub use log;

/// Log a `&str` at [`LogLevel::Info`].
#[macro_export]
macro_rules! info {
	($msg:expr $(,)?) => {
		$crate::log!($crate::log::LogLevel::Info, $msg)
	};
}
pub use info;

/// Log a `&str` at [`LogLevel::Warn`].
#[macro_export]
macro_rules! warn {
	($msg:expr $(,)?) => {
		$crate::log!($crate::log::LogLevel::Warn, $msg)
	};
}
// conflicts with a builtin ;-;
pub use super::warn;

/// Log a `&str` at [`LogLevel::Error`].
#[macro_export]
macro_rules! error {
	($msg:expr $(,)?) => {
		$crate::log!($crate::log::LogLevel::Error, $msg)
	};
}
pub use error;

/// Log a `&str` at [`LogLevel::Debug`].
#[macro_export]
macro_rules! debug {
	($msg:expr $(,)?) => {
		$crate::log!($crate::log::LogLevel::Debug, $msg)
	};
}
pub use debug;

/// Log a `&str` at [`LogLevel::Verbose`].
#[macro_export]
macro_rules! verbose {
	($msg:expr $(,)?) => {
		$crate::log!($crate::log::LogLevel::Verbose, $msg)
	};
}
pub use verbose;

/// Call through to the variadic sys::app_log for formatting runtime values.
///
/// Literal `&'static str` are NUL-terminated at compile time.
///
/// # Safety
/// You must ensure that `fmt` is a valid `vsnprintf` fmt string (see upstream Pebble docs), and
/// that you have passed the correct number and type of varargs.
/// Pay special attention to the interaction between `%s` and rust `&str`:
/// ```
/// let name_str: &str = "quartzshard";
/// let name_cstr: &CStr = c"quartzshard";
/// unsafe {
///     // OK: cstr is NUL terminated
///     log_sys_fmt!(LogLevel::Info, "Hello %s!", name_cstr);
///     // OK: %.*s is a bounded read
///     log_sys_fmt!(LogLevel::Info, "Hello %.*s!", name_str.len(), name_str.as_ptr());
///     // BAD: Str is not NUL terminated and this can read past the end. UB!
///     log_sys_fmt!(LogLevel::Info, "Hello %s!", name_str);
///     // BAD: &str is (ptr,len), so actually passes ptr, len to the internal call.
///     // We also can't rely on this behaviour because that decomp order is not stable
///     // This actually prints "Hello name! (name len as hex)", not "Hello name! beef"
///     log_sys_fmt!(LogLevel::Info, "Hello %.*s! %x", name_str.len(), name_str, 0xBEEF);
///
/// }
/// ```
///
/// If you'd rather not deal with all of this, pass it &CStr - those are NUL terminated and %s reads
/// them no-fuss
#[macro_export]
macro_rules! log_sys_fmt {
	($level:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {
		$crate::sys::app_log(
			$crate::log::__macro_check_log_level($level),
			// const concat a nul-byte onto the str == valid cstr literal
			concat!(file!(), "\0").as_ptr(),
			line!() as i32,
			// const concat a nul-byte onto the str == valid cstr literal
			concat!($fmt, "\0").as_ptr(),
			$($arg),*
		)
	};
}
pub use log_sys_fmt;
