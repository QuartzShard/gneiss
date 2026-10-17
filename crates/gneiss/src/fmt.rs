//! Formatting text through the firmware's `snprintf`.
//!
//! [`pbl_fmt_write!`](crate::pbl_fmt_write) appends to a [`StackText`] or a [`HeapText`], as
//! `write!` would, and [`pbl_fmt_alloc!`](crate::pbl_fmt_alloc) returns a new `HeapText`, as
//! `format!` would. Both are in the [prelude](crate::prelude). They take `format!`'s strings
//!  and turn each into one `snprintf` call with an ordinary C format, built at compile time.
//!
//! ```no_run
//! # fn main() {
//! use gneiss::{prelude::*, ui::StackText};
//!
//! let (lap, ms, name) = (3, 81_920_u32, "Lewis");
//! let mut time = StackText::<16>::new();
//! let _ = pbl_fmt_write!(time, "Lap {}: {}:{:02}.{:03}", lap, ms / 60_000, ms / 1000 % 60, ms % 1000);
//! // time is "Lap 3: 1:21.920"
//!
//! let line = pbl_fmt_alloc!("{name:>8} {:+} {:#x}", -4, 255u8);
//! // line is "   Lewis -4 0xff"
//! # }
//! ```
//!
//! # Accepted arguments
//!
//! `impl` [`Arg`]: Integers of any size, `str`, `String`, `CStr`, `bool`, [`StackText`] and
//! [`HeapText`].
//! The trait is open but `unsafe`, incorrect implementation will cause UB in the system `snprintf`
//!
//! Format with:
//! - `{}`, `{:x}`, `{:X}`, `{:o}` and `{:#x}`
//! - a width, aligned `<` or `>`, padded with spaces, or with zeros as `{:05}`; strings align left
//!   unless told otherwise, as in `format!`
//! - `{:+}` for signed integers
//! - positional, named and captured arguments: `{0}`, `{name}`, `name = value`
//!
//! Not `{:?}`, floats, `char`, centring, fills other than spaces, precision, or widths taken from
//! arguments: each is a compile error. Widths count bytes, not characters.
//!
//! # `{:s}` and `{:d}`
//!
//! For each argument formatted only by `{}`, the macro writes both a string and integer version
//! of the firmware call and the type picks one - since the type is not known during macro-expansion.
//! The other compiles away, but each such argument doubles the code compiled: 5 or more warn, 9 or more
//! are an error.
//! Mark an argument `{:s}` (a string) or `{:d}` (an integer) to avoid compile times growing by
//! 2^(args).
//!
//! # Errors
//!
//! Format string errors are reported as `format!` reports them. Type errors, such as `{:x}` on a
//! string, are checked once the code is compiled, so `cargo build` reports them and `cargo check`
//! doesn't.
//!
//! ```compile_fail
//! # fn main() {
//! # let mut text = gneiss::ui::StackText::<8>::new();
//! // pbl_fmt_write! has no `Debug` formatting
//! let _ = gneiss::pbl_fmt_write!(text, "{:?}", 1);
//! # }
//! ```
//!
//! # Results
//!
//! `pbl_fmt_write!` returns `Err` if the text didn't fit, keeping what did: a `StackText` cuts it
//! short, while a `HeapText` grows to fit.
extern crate alloc;
use core::ffi::{CStr, c_char};

use crate::ui::{HeapText, StackText};

/// A value [`pbl_fmt_write!`](crate::pbl_fmt_write) can format: a string, passed to `snprintf`
/// as its length and a pointer for `%.*s`, or an integer, passed as its value for `%ld`, `%llx`
/// and so on.
///
/// gneiss implements it for the integers, `str`, `String`, `CStr`, `bool`, [`StackText`] and
/// [`HeapText`], and references to them. An app can implement it for its own types, as one or the other:
///
/// ```no_run
/// # fn main() {
/// use core::ffi::c_char;
///
/// use gneiss::{fmt::Arg, prelude::*};
///
/// struct Percent(u8);
///
/// // SAFETY: an integer, passed as a u32 for `l`
/// unsafe impl Arg for Percent {
///     const IS_STR: bool = false;
///     type Int = u32;
///
///     fn int(&self) -> u32 {
///         u32::from(self.0)
///     }
///
///     fn str(&self) -> (i32, *const c_char) {
///         (0, core::ptr::null())
///     }
/// }
///
/// let line = pbl_fmt_alloc!("Battery: {}%", Percent(80));
/// // line is "Battery: 80%"
/// # }
/// ```
///
/// # Safety
/// `pbl_fmt_write!` passes these values through C varargs, unchecked:
///
/// - A string (`IS_STR`) must return from [`str`](Self::str) a length of at least 0, and a
///   pointer to that many readable bytes, or to fewer ending in a NUL.
/// - An integer must make [`Int`](Self::Int) the size [`MODIFIER`](Self::MODIFIER) tells
///   `snprintf` to read: `i32` or `u32` for `l`, `i64` or `u64` for `ll`.
///
/// `pbl_fmt_write!` calls only the method for the type's kind, but both must exist.
#[diagnostic::on_unimplemented(
	message = "`{Self}` can't be formatted by `pbl_fmt_write!`",
	label = "not a type `pbl_fmt_write!` formats",
	note = "`pbl_fmt_write!` formats integers, `str`, `String`, `CStr`, `bool`, `StackText` and `HeapText`"
)]
pub unsafe trait Arg {
	/// Whether it's a string, formatted by `{}` and `{:s}`, rather than an integer, formatted by
	/// `{}`, `{:d}`, `{:x}`, `{:X}` and `{:o}`.
	const IS_STR: bool;
	/// An integer's size, as a `snprintf` length modifier: `l` for 32 bits, `ll` for 64.
	const MODIFIER: &'static [u8] = b"l";
	/// Whether an integer is signed: printed by `{}` as `%d` rather than `%u`, and allowed `{:+}`.
	const SIGNED: bool = false;
	/// The integer passed to `snprintf`, the size `MODIFIER` gives. A string names one too, as
	/// `i32`, since the code for the kind it isn't still has to compile.
	type Int;
	/// An integer's value, as `Int`. Called only if it isn't a string.
	fn int(&self) -> Self::Int;
	/// A string's length and pointer, for `%.*s`. Called only if it's a string.
	fn str(&self) -> (i32, *const c_char);
}

macro_rules! int {
	($($t: ty => $c: ty, $l: literal, $signed: literal;)*) => {$(
		// SAFETY: the value, widened to the size MODIFIER names
		unsafe impl Arg for $t {
			const IS_STR: bool = false;
			const MODIFIER: &'static [u8] = $l;
			const SIGNED: bool = $signed;

			type Int = $c;

			#[inline]
			fn int(&self) -> $c {
				*self as $c
			}

			#[inline]
			fn str(&self) -> (i32, *const c_char) {
				(0, core::ptr::null())
			}
		}
	)*};
}

int! {
	i8 => i32, b"l", true;
	i16 => i32, b"l", true;
	i32 => i32, b"l", true;
	isize => i32, b"l", true;
	u8 => u32, b"l", false;
	u16 => u32, b"l", false;
	u32 => u32, b"l", false;
	usize => u32, b"l", false;
	i64 => i64, b"ll", true;
	u64 => u64, b"ll", false;
}

/// `bytes` as `%.*s` reads them. The firmware stops at a NUL, so it never reads past the end
fn c_str_arg(bytes: &[u8]) -> (i32, *const c_char) {
	(
		i32::try_from(bytes.len()).unwrap_or(i32::MAX),
		bytes.as_ptr().cast(),
	)
}

macro_rules! string {
	($([$($g: tt)*] $t: ty => |$s: ident| $bytes: expr;)*) => {$(
		// SAFETY: the bytes' length and pointer
		unsafe impl<$($g)*> Arg for $t {
			const IS_STR: bool = true;

			type Int = i32;

			#[inline]
			fn int(&self) -> i32 {
				0
			}

			#[inline]
			fn str(&self) -> (i32, *const c_char) {
				let $s = self;
				c_str_arg($bytes)
			}
		}
	)*};
}

string! {
	[] str => |s| s.as_bytes();
	[] CStr => |s| s.to_bytes();
	[] bool => |b| if *b { b"true" } else { b"false" };
	[const N: usize] StackText<N> => |s| s.as_c_str().to_bytes();
	[] HeapText => |s| s.as_c_str().to_bytes();
	[] alloc::string::String => |s| s.as_bytes();
}

// SAFETY: as for T
unsafe impl<T: Arg + ?Sized> Arg for &T {
	type Int = T::Int;

	const IS_STR: bool = T::IS_STR;
	const MODIFIER: &'static [u8] = T::MODIFIER;
	const SIGNED: bool = T::SIGNED;

	#[inline]
	fn int(&self) -> T::Int {
		(**self).int()
	}

	#[inline]
	fn str(&self) -> (i32, *const c_char) {
		(**self).str()
	}
}

/// Text [`pbl_fmt_write!`](crate::pbl_fmt_write) can append to. Implement it to format into
/// your own buffer.
pub trait Sink {
	/// Append what `fill` writes, and return whether all of it fit.
	///
	/// `fill` works like `snprintf`: hand it the spare room after the text, including space for
	/// the NUL, and it writes as much as fits, NUL-terminated. It returns the length it needed, not
	/// counting the NUL, or a negative number on failure. A sink may call it again, such as with
	/// more room.
	fn append_formatted(&mut self, fill: impl Fn(&mut [u8]) -> i32) -> bool;
}

impl<const N: usize> Sink for StackText<N> {
	fn append_formatted(&mut self, fill: impl Fn(&mut [u8]) -> i32) -> bool {
		self.append_c(fill)
	}
}

impl Sink for HeapText {
	fn append_formatted(&mut self, fill: impl Fn(&mut [u8]) -> i32) -> bool {
		self.append_c(fill)
	}
}

impl<S: Sink + ?Sized> Sink for &mut S {
	fn append_formatted(&mut self, fill: impl Fn(&mut [u8]) -> i32) -> bool {
		(**self).append_formatted(fill)
	}
}

/// `parts` joined into `L` bytes, NUL-padded. Called in a const block by
/// [`pbl_fmt_write!`](crate::pbl_fmt_write).
#[doc(hidden)]
pub const fn __build<const L: usize>(parts: &[&[u8]]) -> [u8; L] {
	let mut out = [0; L];
	let mut at = 0;
	let mut i = 0;
	while i < parts.len() {
		let part = parts[i];
		let mut j = 0;
		while j < part.len() {
			out[at] = part[j];
			at += 1;
			j += 1;
		}
		i += 1;
	}
	assert!(at < L, "pbl_fmt_write!: format longer than its bound");
	out
}

/// [`pbl_fmt_write!`](crate::pbl_fmt_write) into a new [`HeapText`], returned: `format!` for
/// `snprintf`. In the [prelude](crate::prelude).
///
/// ```no_run
/// # fn main() {
/// use gneiss::prelude::*;
///
/// # let charge = 80;
/// let line = pbl_fmt_alloc!("Battery: {}%", charge);
/// // line is "Battery: 80%"
/// # }
/// ```
#[macro_export]
macro_rules! pbl_fmt_alloc {
	($($args: tt)*) => {{
		let mut text = $crate::ui::HeapText::new();
		let _ = $crate::pbl_fmt_write!(text, $($args)*);
		text
	}};
}
