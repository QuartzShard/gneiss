#![no_std]
#![deny(
	clippy::panic,
	clippy::panic_in_result_fn,
	clippy::string_slice,
	clippy::unchecked_time_subtraction
)]
#![warn(clippy::unreachable, clippy::unimplemented, clippy::todo)]

pub use gneiss_macros::{entry, service};
pub use pebble_sys as sys;
