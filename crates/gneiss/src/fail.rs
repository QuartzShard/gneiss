//! Failing with a message: under `-Cpanic=immediate-abort` a panic prints no diagnostics, so
//! [`pbl_panic!`](crate::pbl_panic), [`pbl_assert!`](crate::pbl_assert) and [`PblUnwrap`] log
//! `Panic! <message>` at the caller's file and line before closing the app.
use core::{ffi::CStr, panic::Location};

use crate::{log::LogLevel, sys};

/// Log `Panic! <msg>` at `file:line`, then close the app.
#[cold]
#[inline(never)]
fn log_and_halt(msg: &CStr, file: &CStr, line: u32) -> ! {
	// SAFETY: both strings are NUL-terminated, for the firmware's %s and filename
	unsafe {
		sys::app_log(
			LogLevel::Error as u8,
			file.as_ptr(),
			line as i32,
			c"Panic! %s".as_ptr(),
			msg.as_ptr(),
		);
	}
	sys::panic::halt()
}

/// Log `Panic! <msg>` at `location`, then close the app.
#[doc(hidden)]
#[inline]
pub fn __fail(msg: &CStr, location: &Location<'_>) -> ! {
	log_and_halt(msg, location.file_as_c_str(), location.line())
}

/// Log `Panic! <msg>` at `gneiss:<line>`, then close the app: for failures inside gneiss, which
/// would otherwise put gneiss's long source paths in every app.
#[inline]
pub(crate) fn fail_in_gneiss(msg: &CStr, line: u32) -> ! {
	log_and_halt(msg, c"gneiss", line)
}

/// [`__fail`] at the caller.
#[doc(hidden)]
#[track_caller]
#[inline]
pub fn __fail_here(msg: &CStr) -> ! {
	__fail(msg, Location::caller())
}

/// `unwrap` and `expect` that log why the app closed. In the [prelude](crate::prelude).
///
/// ```no_run
/// # fn main() {
/// use gneiss::prelude::*;
///
/// let first = [3, 1, 2].iter().min().pbl_expect(c"no laps");
/// # }
/// ```
pub trait PblUnwrap<T> {
	/// The value, or if there is none, log `Panic! <msg>` at the caller and close the app.
	fn pbl_expect(self, msg: &CStr) -> T;
	/// The value, or if there is none, log `Panic! unwrapped None` (or `Err`) at the caller and
	/// close the app.
	fn pbl_unwrap(self) -> T;
}

impl<T> PblUnwrap<T> for Option<T> {
	#[track_caller]
	#[inline]
	fn pbl_expect(self, msg: &CStr) -> T {
		match self {
			Some(value) => value,
			None => __fail(msg, Location::caller()),
		}
	}

	#[track_caller]
	#[inline]
	fn pbl_unwrap(self) -> T {
		match self {
			Some(value) => value,
			None => __fail(c"unwrapped None", Location::caller()),
		}
	}
}

impl<T, E> PblUnwrap<T> for Result<T, E> {
	#[track_caller]
	#[inline]
	fn pbl_expect(self, msg: &CStr) -> T {
		match self {
			Ok(value) => value,
			Err(_) => __fail(msg, Location::caller()),
		}
	}

	#[track_caller]
	#[inline]
	fn pbl_unwrap(self) -> T {
		match self {
			Ok(value) => value,
			Err(_) => __fail(c"unwrapped Err", Location::caller()),
		}
	}
}

/// `msg`, a string literal, as a `&'static CStr`. A NUL in it is a compile error.
#[doc(hidden)]
#[macro_export]
macro_rules! __c_literal {
	($msg: expr) => {{
		#[expect(clippy::disallowed_macros, reason = "fails the build, not the app")]
		const MSG: &::core::ffi::CStr =
			match ::core::ffi::CStr::from_bytes_with_nul(concat!($msg, "\0").as_bytes()) {
				Ok(msg) => msg,
				Err(_) => panic!("pbl_panic! messages can't contain NUL"),
			};
		MSG
	}};
}

/// Log `Panic! <msg>` at the call site and close the app: `panic!` for
/// `-Cpanic=immediate-abort`, which prints no diagnostics for a `panic!`. `msg` is a string
/// literal, with no formatting. In the [prelude](crate::prelude).
///
/// ```no_run
/// # fn main() {
/// use gneiss::prelude::*;
///
/// # let level = 3;
/// if level > 9000 {
///     pbl_panic!("level out of range");
/// }
/// # }
/// ```
#[macro_export]
macro_rules! pbl_panic {
	() => {
		$crate::fail::__fail_here(c"explicit panic")
	};
	($msg: literal $(,)?) => {
		$crate::fail::__fail_here($crate::__c_literal!($msg))
	};
}

/// [`pbl_panic!`](crate::pbl_panic) unless `cond` holds: `assert!` for
/// `-Cpanic=immediate-abort`. Without a message, it logs `Panic! assertion failed: <cond>`. In
/// the [prelude](crate::prelude).
#[macro_export]
macro_rules! pbl_assert {
	($cond: expr $(,)?) => {
		if !$cond {
			$crate::fail::__fail_here($crate::__c_literal!(concat!(
				"assertion failed: ",
				stringify!($cond)
			)))
		}
	};
	($cond: expr, $msg: literal $(,)?) => {
		if !$cond {
			$crate::fail::__fail_here($crate::__c_literal!($msg))
		}
	};
}

/// [`pbl_panic!`](crate::pbl_panic), at `gneiss:<line>`
macro_rules! gneiss_panic {
	($msg: literal) => {
		$crate::fail::fail_in_gneiss($crate::__c_literal!($msg), line!())
	};
}

/// [`pbl_assert!`](crate::pbl_assert), at `gneiss:<line>`
macro_rules! gneiss_assert {
	($cond: expr, $msg: literal) => {
		if !$cond {
			$crate::fail::gneiss_panic!($msg)
		}
	};
}

/// `$opt`'s value, or [`gneiss_panic!`] with `$msg`
macro_rules! gneiss_expect {
	($opt: expr, $msg: literal) => {
		match $opt {
			Some(value) => value,
			None => $crate::fail::gneiss_panic!($msg),
		}
	};
}

pub(crate) use gneiss_assert;
pub(crate) use gneiss_expect;
pub(crate) use gneiss_panic;
