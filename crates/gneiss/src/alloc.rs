//! A [`GlobalAlloc`] impl over the Pebble SDK malloc/free/realloc.
//!
//! Enables the use of Rust's standard collections:
//! ```ignore
//! extern crate alloc;
//! use alloc::vec::Vec;
//!use gneiss
//! #[gneiss::entry]
//! fn main() {
//!     let mut vector = Vec::new();
//!     // ..
//! }
//! ```
//! Costs 5KiB of binary size - fine in most cases, but be aware.
//!
//! # On Allocations of Align > 4:
//! The `task_malloc()` provided by PebbleOS returns pointers with an Alignment of 4, so to allocate
//! types with a larger Align, we must allocate `size + align` bytes to ensure that we can fit the
//! type within our allocation correctly aligned. This cost will be amortized over large
//! collections, but be aware if you're allocating things of Align 8 and up (u64/f64, some Pointer
//! types), that the allocator will waste an extra `align` bytes in the allocation.
use core::{
	alloc::{GlobalAlloc, Layout},
	ffi::c_void,
	ptr,
};

use pebble_sys::{free, malloc, realloc};

#[global_allocator]
pub static ALLOC: PebbleAlloc = PebbleAlloc;

pub struct PebbleAlloc;

unsafe impl GlobalAlloc for PebbleAlloc {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		// Do NOT compile for targets where pointers aren't u32 (all watches are)
		const _: () = assert!(size_of::<usize>() == size_of::<u32>());

		let size = layout.size();
		let align = layout.align();

		if align <= 4 {
			unsafe { malloc(size as u32).cast() }
		} else {
			// task_malloc() returns align 4 (unsigned long), so we must over-allocate.
			//
			// We must ensure that a valid alignment exists in the returned memory, and we stash the
			// base pointer *in tbe padding* at (aligned as *mut u32) - 1. For cases where the base
			// pointer was already aligned (worst), that means we need to allocate +align extra bytes
			// to offset it to the next valid addr - padding bytes are in the [4, align) range.
			let alloc_size = size + align;
			let base_ptr = unsafe { malloc(alloc_size as u32) };

			// OOM
			if base_ptr.is_null() {
				return ptr::null_mut();
			}

			let offset = match base_ptr.align_offset(align) {
				// There must be memory we own preceeding the pointer, so offset to the next align
				0 => align,
				// Type could not be aligned, we should probably bail the allocation
				usize::MAX => return ptr::null_mut(),
				off => off,
			};

			#[allow(
				clippy::cast_ptr_alignment,
				reason = "ptr from malloc() is always at least align(4)"
			)]
			unsafe {
				let aligned_ptr = base_ptr.add(offset);
				// Stash the base 1 word before the aligned. This is a surprise tool that will help
				// us later
				aligned_ptr.cast::<*mut c_void>().sub(1).write(base_ptr);
				aligned_ptr.cast()
			}
		}
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		let align = layout.align();
		if align <= 4 {
			unsafe { free(ptr.cast()) }
		} else {
			unsafe {
				// Recover the base pointer stashed 1 word behind the aligned ptr.
				#[allow(
					clippy::cast_ptr_alignment,
					reason = "ptr from malloc() is always at least align(4)"
				)]
				let base_ptr = ptr.cast::<*mut c_void>().sub(1).read();
				free(base_ptr);
			}
		}
	}

	unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
		let align = layout.align();
		if align <= 4 {
			unsafe { realloc(ptr.cast(), new_size as u32).cast() }
		} else {
			// Fall back to Alloc -> Copy -> Free.
			// Stashing abstracted within alloc/dealloc
			unsafe {
				let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
				let new_ptr = self.alloc(new_layout);
				if !new_ptr.is_null() {
					ptr::copy_nonoverlapping(ptr, new_ptr, new_size.min(layout.size()));
					self.dealloc(ptr, layout);
				}
				new_ptr
			}
		}
	}
}
