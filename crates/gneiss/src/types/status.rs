//! Errors from the firmware calls that return a `status_t`.
use core::fmt;

use crate::sys;

/// Why a firmware call failed: the negative half of its `status_t`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum StatusError {
	Error,
	Unknown,
	Internal,
	InvalidArgument,
	OutOfMemory,
	/// Not enough persistent storage - not memory.
	OutOfStorage,
	OutOfResources,
	OutOfRange,
	DoesNotExist,
	InvalidOperation,
	Busy,
	TryAgain,
	/// A negative status the SDK doesn't name.
	Other(i32),
}

impl StatusError {
	/// `status` as its success value, zero or more, such as a count of bytes written. The error if
	/// it's negative.
	pub fn check(status: sys::status_t) -> Result<u32, Self> {
		u32::try_from(status).map_err(|_| Self::from_negative(status))
	}

	fn from_negative(status: sys::status_t) -> Self {
		match i8::try_from(status) {
			Ok(sys::StatusCode_E_ERROR) => Self::Error,
			Ok(sys::StatusCode_E_UNKNOWN) => Self::Unknown,
			Ok(sys::StatusCode_E_INTERNAL) => Self::Internal,
			Ok(sys::StatusCode_E_INVALID_ARGUMENT) => Self::InvalidArgument,
			Ok(sys::StatusCode_E_OUT_OF_MEMORY) => Self::OutOfMemory,
			Ok(sys::StatusCode_E_OUT_OF_STORAGE) => Self::OutOfStorage,
			Ok(sys::StatusCode_E_OUT_OF_RESOURCES) => Self::OutOfResources,
			Ok(sys::StatusCode_E_RANGE) => Self::OutOfRange,
			Ok(sys::StatusCode_E_DOES_NOT_EXIST) => Self::DoesNotExist,
			Ok(sys::StatusCode_E_INVALID_OPERATION) => Self::InvalidOperation,
			Ok(sys::StatusCode_E_BUSY) => Self::Busy,
			Ok(sys::StatusCode_E_AGAIN) => Self::TryAgain,
			_ => Self::Other(status),
		}
	}
}

impl fmt::Display for StatusError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(match self {
			Self::Error => "error",
			Self::Unknown => "unknown error",
			Self::Internal => "internal error",
			Self::InvalidArgument => "invalid argument",
			Self::OutOfMemory => "out of memory",
			Self::OutOfStorage => "out of storage",
			Self::OutOfResources => "out of resources",
			Self::OutOfRange => "argument out of range",
			Self::DoesNotExist => "does not exist",
			Self::InvalidOperation => "invalid operation",
			Self::Busy => "busy",
			Self::TryAgain => "try again",
			Self::Other(status) => return write!(f, "status {status}"),
		})
	}
}

impl core::error::Error for StatusError {}
