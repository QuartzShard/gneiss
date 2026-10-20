//! Logging from the watch
//!
//! [`info!`], [`warn!`](macro@warn), [`error!`], [`debug!`] and [`verbose!`] log a message,
//! tagged with the caller's file and line: a format string and its arguments, in
//! [`pbl_fmt_write!`](crate::pbl_fmt_write)'s format language (see [`fmt`](crate::fmt)), or a
//! single string, logged as it is. Each is one `app_log` call, so no `core::fmt`.
//!
//! ```no_run
//! # fn _example() {
//! # use gneiss::service::Peekable;
//! # let charge = gneiss::service::BatteryService::peek(());
//! use gneiss::log;
//!
//! log::info!("App started");
//! log::warn!("Battery at {}%", charge.charge_percent);
//! // logs "Battery at 80%", with the caller's file and line
//! # let name = "Lewis";
//! log::info!(name);
//! # }
//! ```
//!
//! A single argument that isn't a string literal is logged as a string, braces and all:
//! `info!(name)` is `info!("{:s}", name)`. `concat!(..)` isn't a literal to a macro, so it's
//! logged that way too, rather than read as a format.
//!
//! Without the `logs` feature, every level compiles to nothing; without `debug-logs`, [`debug!`] and
//! [`verbose!`] do. Panics log either way (see [`fail`](crate::fail)).
//!
//! The firmware truncates messages to 88 bytes, and filenames to the last 15 characters.
use crate::sys;

#[repr(u8)]
pub enum LogLevel {
	Info    = sys::AppLogLevel_APP_LOG_LEVEL_INFO,
	Warn    = sys::AppLogLevel_APP_LOG_LEVEL_WARNING,
	Error   = sys::AppLogLevel_APP_LOG_LEVEL_ERROR,
	Debug   = sys::AppLogLevel_APP_LOG_LEVEL_DEBUG,
	Verbose = sys::AppLogLevel_APP_LOG_LEVEL_DEBUG_VERBOSE,
}

pub const LOGS: bool = cfg!(feature = "logs");
pub const DEBUG: bool = cfg!(feature = "debug-logs");

#[doc(hidden)]
pub const fn __macro_check_log_level(l: LogLevel) -> u8 {
	l as u8
}

/// Log at an explicit [`LogLevel`], if the `logs` feature is on: a format string and its arguments,
/// in [`pbl_fmt_write!`](crate::pbl_fmt_write)'s format language, or a single string, logged as it
/// is.
#[macro_export]
macro_rules! log {
	($level: expr, $fmt: literal $(, $($args: tt)*)?) => {
		if $crate::log::LOGS {
			$crate::__pbl_log!(
				$crate::log::__macro_check_log_level($level),
				concat!(file!(), "\0"),
				line!() as i32,
				$fmt $(, $($args)*)?
			)
		}
	};
	($level: expr, $msg: expr $(,)?) => {
		$crate::log!($level, "{:s}", $msg)
	};
}
pub use log;

/// [`log!`](crate::log!) at [`LogLevel::Info`].
#[macro_export]
macro_rules! info {
	($($args: tt)*) => {
		$crate::log!($crate::log::LogLevel::Info, $($args)*)
	};
}
pub use info;

/// [`log!`](crate::log!) at [`LogLevel::Warn`].
#[macro_export]
macro_rules! warn {
	($($args: tt)*) => {
		$crate::log!($crate::log::LogLevel::Warn, $($args)*)
	};
}
// conflicts with a builtin ;-;
pub use super::warn;

/// [`log!`](crate::log!) at [`LogLevel::Error`].
#[macro_export]
macro_rules! error {
	($($args: tt)*) => {
		$crate::log!($crate::log::LogLevel::Error, $($args)*)
	};
}
pub use error;

/// [`log!`](crate::log!) at [`LogLevel::Debug`], if the `debug-logs` feature is on.
#[macro_export]
macro_rules! debug {
	($($args: tt)*) => {
		if $crate::log::DEBUG {
			$crate::log!($crate::log::LogLevel::Debug, $($args)*)
		}
	};
}
pub use debug;

/// [`log!`](crate::log!) at [`LogLevel::Verbose`], if the `debug-logs` feature is on.
#[macro_export]
macro_rules! verbose {
	($($args: tt)*) => {
		if $crate::log::DEBUG {
			$crate::log!($crate::log::LogLevel::Verbose, $($args)*)
		}
	};
}
pub use verbose;

/// [`log!`](crate::log!) for gneiss's own messages, logged as from `gneiss`: `file!()` would put
/// gneiss's long source paths into every app. It follows the `logs` and `debug-logs` features, as
/// [`log!`](crate::log!) and [`debug!`](crate::log::debug) do.
macro_rules! gneiss_log {
	(Debug, $($args: tt)*) => {
		if $crate::log::DEBUG {
			$crate::log::gneiss_log!(@ Debug, $($args)*)
		}
	};
	(Verbose, $($args: tt)*) => {
		if $crate::log::DEBUG {
			$crate::log::gneiss_log!(@ Verbose, $($args)*)
		}
	};
	(@ $level: ident, $fmt: literal $(, $($args: tt)*)?) => {
		if $crate::log::LOGS {
			$crate::__pbl_log!(
				$crate::log::__macro_check_log_level($crate::log::LogLevel::$level),
				c"gneiss",
				line!() as i32,
				$fmt $(, $($args)*)?
			)
		}
	};
	($level: ident, $($args: tt)*) => {
		$crate::log::gneiss_log!(@ $level, $($args)*)
	};
}
pub(crate) use gneiss_log;
