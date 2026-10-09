//! Raw byte resources, read whole or streamed in chunks.
extern crate alloc;
use alloc::{boxed::Box, vec::Vec};
use core::{alloc::Allocator, ops::Deref};

use crate::{resource::RawBytesKey, sys};

#[derive(Clone, Copy)]
pub struct ResourceHandle {
	handle: *mut core::ffi::c_void,
	len:    usize,
}

#[allow(
	clippy::len_without_is_empty,
	reason = "Resource of len 0 is not correct"
)]
impl ResourceHandle {
	/// Get a handle to a resource.
	///
	/// This reads the resource to verify it's real and intact, so is not trivially cheap for large
	/// resources and should probably not be called in a loop
	pub fn load(key: RawBytesKey) -> Option<Self> {
		unsafe {
			let handle = sys::resource_get_handle(key.0.get());
			if handle.is_null() {
				None
			} else {
				Some(ResourceHandle {
					handle,
					len: sys::resource_size(handle),
				})
			}
		}
	}

	pub fn len(&self) -> usize {
		self.len
	}

	/// Loads min(`view.len()`, available) into view, returning how many bytes it read.
	/// Prefer [`ResourceHandle::iter`] to calling this in a loop, this is for random access
	pub fn load_slice(self, pos: usize, view: &mut [u8]) -> usize {
		let ResourceHandle { handle, len: _ } = self;
		if view.is_empty() {
			return 0;
		}
		unsafe { sys::resource_load_byte_range(handle, pos as u32, view.as_mut_ptr(), view.len()) }
	}

	/// Convenience for fixed-sized checked loads
	pub fn load_array<const SIZE: usize>(self, pos: usize) -> Option<[u8; SIZE]> {
		let mut array = [0u8; SIZE];
		if self.load_slice(pos, &mut array) < SIZE {
			None
		} else {
			Some(array)
		}
	}

	/// Returns an iterator that provides the entire resource sequentially, SIZE bytes at a time
	pub fn iter<const SIZE: usize>(&self) -> ResourceIter<SIZE> {
		const { assert!(SIZE > 0, "Chunks must be >0") };
		ResourceIter {
			handle: *self,
			offset: 0,
		}
	}
}

pub struct ResourceIter<const SIZE: usize> {
	handle: ResourceHandle,
	offset: usize,
}

pub struct Chunk<const SIZE: usize> {
	inner: [u8; SIZE],
	len:   usize,
}

impl<const SIZE: usize> Deref for Chunk<SIZE> {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.inner[..self.len]
	}
}

/// Load an entire resource.
///
/// ```
/// let res = load_all(resources::RAW_BYTES, PebbleAlloc).expect("RAW_BYTES declared");
/// ```
///
/// Beware that loading a resource larger than your available heap will hit OOM
///
/// # Panics
/// If handed an invalid `ResourceID` (but any resource type may be used as a `RawBytesKey`)
pub fn load_all<A: Allocator>(key: RawBytesKey, alloc: A) -> Option<Box<[u8], A>> {
	let ResourceHandle { handle, len } = ResourceHandle::load(key).expect("Invalid resource id");

	if len == 0 {
		return None;
	}

	let mut buf = Vec::with_capacity_in(len, alloc);

	if unsafe { sys::resource_load(handle, buf.as_mut_ptr(), len) } < len {
		None
	} else {
		unsafe { buf.set_len(len) }
		Some(buf.into_boxed_slice())
	}
}

impl<const SIZE: usize> Iterator for ResourceIter<SIZE> {
	type Item = Chunk<SIZE>;

	fn next(&mut self) -> Option<Self::Item> {
		let ResourceHandle { handle, len } = self.handle;
		let mut chunk = Chunk {
			inner: [0u8; SIZE],
			len:   0,
		};

		let read = unsafe {
			sys::resource_load_byte_range(
				handle,
				self.offset as u32,
				chunk.inner.as_mut_ptr(),
				chunk.inner.len(),
			)
		};

		if read == 0 || self.offset >= len {
			return None;
		}

		chunk.len = read;
		self.offset += read;

		Some(chunk)
	}
}

impl<const SIZE: usize> IntoIterator for Chunk<SIZE> {
	type IntoIter = core::iter::Take<core::array::IntoIter<u8, SIZE>>;
	type Item = u8;

	fn into_iter(self) -> Self::IntoIter {
		self.inner.into_iter().take(self.len)
	}
}
