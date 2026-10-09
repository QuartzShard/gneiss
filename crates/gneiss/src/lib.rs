//! A safe Rust SDK for Pebble watchapps and watchfaces, built on [`pebble_sys`].
#![no_std]
#![deny(
	clippy::panic,
	clippy::panic_in_result_fn,
	clippy::string_slice,
	clippy::unchecked_time_subtraction
)]
#![warn(clippy::unreachable, clippy::unimplemented, clippy::todo)]
#![feature(
	const_array,
	const_trait_impl,
	allocator_api,
	ptr_cast_slice,
	auto_traits,
	negative_impls,
	associated_type_defaults
)]

pub use gneiss_macros::{entry, service};
pub use linkme;
pub use pebble_sys as sys;
/// Present when building with `-Cpanic=immediate-abort`. `use gneiss::ufmt;` puts it in scope for
/// `ufmt`'s macros.
#[cfg(panic = "immediate-abort")]
pub use ufmt;

pub mod alloc;

pub mod log;
pub mod service;
pub mod ui;

pub mod link;

// Typed Resources
pub mod resource;

// Strong type wrappers for sys types
mod types;
pub use types::{accel, compass, connection, dict, focus, health, time, touch, wakeup};
