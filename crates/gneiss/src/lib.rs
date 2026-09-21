#![no_std]
#![deny(
	clippy::panic,
	clippy::panic_in_result_fn,
	clippy::string_slice,
	clippy::unchecked_time_subtraction
)]
#![warn(clippy::unreachable, clippy::unimplemented, clippy::todo)]

pub use gneiss_macros::{entry, service};
pub use linkme;
pub use pebble_sys as sys;

#[cfg(feature = "alloc")]
pub mod alloc;

pub mod log;
pub mod service;

// Strong type wrappers for sys types
pub mod accel;
pub mod compass;
pub mod connection;
pub mod focus;
pub mod health;
pub mod time;
pub mod touch;
pub mod wakeup;
