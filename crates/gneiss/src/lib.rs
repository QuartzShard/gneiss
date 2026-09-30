#![no_std]
#![deny(
	clippy::panic,
	clippy::panic_in_result_fn,
	clippy::string_slice,
	clippy::unchecked_time_subtraction
)]
#![warn(clippy::unreachable, clippy::unimplemented, clippy::todo)]
#![feature(const_array, const_trait_impl, allocator_api, ptr_cast_slice)]

pub use gneiss_macros::{entry, service};
pub use linkme;
pub use pebble_sys as sys;

pub mod alloc;

pub mod display;

pub mod log;
pub mod service;

// Message keys
pub mod dict;
pub mod link;

// Typed Resources
pub mod resource;

// Strong type wrappers for sys types
pub mod accel;
pub mod compass;
pub mod connection;
pub mod focus;
pub mod health;
pub mod time;
pub mod touch;
pub mod wakeup;
