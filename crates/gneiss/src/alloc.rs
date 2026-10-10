//! A [`GlobalAlloc`] + [`Allocator`] impl over the Pebble SDK malloc/free/realloc.
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
//! # On Allocations of Align > 4:
//! The `task_malloc()` provided by PebbleOS returns pointers with an Alignment of 4, so to allocate
//! types with a larger Align, we must allocate `size + align` bytes to ensure that we can fit the
//! type within our allocation correctly aligned. This cost will be amortized over large
//! collections, but be aware if you're allocating things of Align 8 and up (u64/f64, some Pointer
//! types), that the allocator will waste an extra `align` bytes in the allocation.
use core::{
	alloc::{AllocError, Allocator, GlobalAlloc, Layout},
	ffi::c_void,
	ptr::{self, NonNull},
};

use gneiss_sys::{free, malloc, realloc};

/// On Cortex-M, this address is Forbidden (Private Peripheral Bus), touching it will cause a
/// `BusFault` - `PebbleOS` then kills the app. Used as the ZST sentinel. If you see this in a log,
/// somebody made a rather large oopsie.
const DANGLING: *mut u8 = ptr::without_provenance_mut(0xE000_0000);

/// The app heap, through the firmware's `malloc`, `free` and `realloc`. The global allocator unless
/// `#[gneiss::entry]` names another, and usable as an [`Allocator`] either way
/// (`Box::new_in(x, PebbleAlloc)`).
pub struct PebbleAlloc;

impl PebbleAlloc {
	fn alloc(layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
		// Do NOT compile for targets where pointers aren't u32 (all watches are)
		const _: () = assert!(size_of::<usize>() == size_of::<u32>());

		let size = layout.size();
		let align = layout.align();

		if size == 0 {
			return Ok(unsafe { NonNull::new_unchecked(DANGLING) }.cast_slice(0));
		}

		let ptr = if align <= 4 {
			unsafe { NonNull::new(malloc(size as u32).cast()).ok_or(AllocError) }
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
				return Err(AllocError);
			}

			let offset = match base_ptr.align_offset(align) {
				// There must be memory we own preceeding the pointer, so offset to the next align
				0 => align,
				// Type could not be aligned, we should probably bail the allocation
				usize::MAX => return Err(AllocError),
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
				NonNull::new(aligned_ptr.cast()).ok_or(AllocError)
			}
		}?;

		Ok(ptr.cast_slice(size))
	}

	unsafe fn dealloc(ptr: NonNull<u8>, layout: Layout) {
		if layout.size() == 0 {
			return;
		}
		let align = layout.align();
		if align <= 4 {
			unsafe { free(ptr.as_ptr().cast()) }
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

	// # Safety:
	// ptr must have come from PebbleAlloc.
	// new_size must be >0.
	unsafe fn realloc(
		ptr: NonNull<u8>,
		layout: Layout,
		new_size: usize,
	) -> Result<NonNull<[u8]>, AllocError> {
		let align = layout.align();
		let ptr = if align <= 4 {
			unsafe {
				NonNull::new(realloc(ptr.as_ptr().cast(), new_size as u32).cast())
					.ok_or(AllocError)?
					.cast_slice(new_size)
			}
		} else {
			// Fall back to Alloc -> Copy -> Free.
			// Stashing abstracted within alloc/dealloc
			unsafe {
				let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
				let new_ptr = Self::alloc(new_layout)?;
				ptr::copy_nonoverlapping(
					ptr.as_ptr(),
					new_ptr.as_ptr().cast(),
					new_size.min(layout.size()),
				);
				Self::dealloc(ptr, layout);
				new_ptr
			}
		};
		Ok(ptr)
	}
}

unsafe impl GlobalAlloc for PebbleAlloc {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		PebbleAlloc::alloc(layout).map_or(ptr::null_mut(), |non_null| non_null.as_ptr().cast())
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		unsafe { PebbleAlloc::dealloc(NonNull::new_unchecked(ptr), layout) }
	}

	unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
		unsafe {
			PebbleAlloc::realloc(NonNull::new_unchecked(ptr), layout, new_size)
				.map_or(ptr::null_mut(), |non_null| non_null.as_ptr().cast())
		}
	}
}

unsafe impl Allocator for PebbleAlloc {
	fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
		Self::alloc(layout)
	}

	unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
		unsafe { Self::dealloc(ptr, layout) }
	}
}
