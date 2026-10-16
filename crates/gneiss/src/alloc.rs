//! Allocation: [`PebbleAlloc`], a [`GlobalAlloc`](core::alloc::GlobalAlloc) +
//! [`Allocator`](core::alloc::Allocator) over the Pebble SDK malloc/free/realloc, and [`Arena`], one
//! allocation holding data built once.
//!
//! `#[gneiss::entry]` registers [`PebbleAlloc`] as the global allocator by default, so Rust's
//! standard collections just work:
//! ```ignore
//! extern crate alloc;
//! use alloc::vec::Vec;
//!
//! #[gneiss::entry]
//! fn main() {
//!     let mut vector = Vec::new();
//!     // ..
//! }
//! ```
//! To register a different global allocator instead, name it:
//! `#[gneiss::entry(global_alloc = my::BumpAlloc)]`.
//!
//! The firmware's `realloc` never resizes in place: it allocates anew, copies, and frees the old
//! block, shrinking included. A collection grown by pushing churns the heap; reserve its capacity
//! up front, or build data that never changes in an [`Arena`].
use core::ptr;

mod arena;
mod heap;

pub use arena::{Arena, ArenaFill, Handle};
pub use heap::PebbleAlloc;

/// On Cortex-M, this address is Forbidden (Private Peripheral Bus), touching it will cause a
/// `BusFault` - `PebbleOS` then kills the app. Used as the ZST sentinel. If you see this in a log,
/// somebody made a rather large oopsie.
const DANGLING: *mut u8 = ptr::without_provenance_mut(0xE000_0000);
