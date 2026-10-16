//! [`Arena`]: data that's built once and never changed, packed into one allocation.
use core::{
	alloc::{Allocator, Layout},
	ffi::CStr,
	marker::PhantomData,
	mem::ManuallyDrop,
	ptr::{self, NonNull},
};

use bytemuck::AnyBitPattern;

use super::PebbleAlloc;

const FOREIGN: &str = "handle from another arena";

/// One allocation holding data that's built once and never changed: strings, values and slices of
/// any `Copy` types, side by side. Filled through an [`ArenaFill`], read back through the
/// [`Handle`]s it hands out, and freed whole when dropped.
///
/// For data the firmware keeps pointers into, such as the strings of a menu, it's one block that
/// never moves, with no growth. Make one of two ways:
/// - [`Arena::new_with_capacity`], when you know, or can bound, how much it'll hold: one block of
///   that size, filled in one go. Room left over stays allocated, so size it closely.
/// - [`Arena::new_exact`], when you don't: it runs a fill function twice, once to measure and once
///   to write, into a block of exactly the size needed.
///
/// Each push is padded to its own alignment, so interleaving values of different alignments, such
/// as strings between structs, wastes up to `align - 1` bytes at each change. Push things of the
/// same alignment together, the most aligned first.
///
/// ```no_run
/// # use gneiss::alloc::Arena;
/// # fn main() {
/// let mut fill = Arena::new_with_capacity(32).expect("out of memory");
/// let title = fill.push_str("High scores");
/// let scores = fill.push_slice(&[120u32, 95, 80]);
/// let arena = fill.finish().expect("arena too small");
/// assert_eq!(arena.str(title), c"High scores");
/// assert_eq!(arena.slice(scores), [120, 95, 80]);
///
/// let (arena, title) = Arena::new_exact(|fill| fill.push_str("Settings")).expect("out of memory");
/// assert_eq!(arena.str(title), c"Settings");
/// # }
/// ```
pub struct Arena {
	ptr:    NonNull<u8>,
	layout: Layout,
}

/// Where something was put in an [`Arena`]: read it back with [`Arena::get`], [`Arena::slice`] or
/// [`Arena::str`], which panic if given a handle from a different arena, their `try_` versions,
/// which return `None` instead, or their `_unchecked` versions.
pub struct Handle<T: ?Sized> {
	offset: u32,
	// Elements of a slice, or bytes of a string including its NUL
	len:    u32,
	_ty:    PhantomData<fn() -> *const T>,
}

impl<T: ?Sized> Clone for Handle<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T: ?Sized> Copy for Handle<T> {}

impl<T> Handle<[T]> {
	pub(crate) fn len(self) -> usize {
		self.len as usize
	}
}

/// An [`Arena`] being filled: push into it, then [`finish`](Self::finish) it. Pushes that don't
/// fit fail the whole arena, so `finish` returns `None`, and the handles they return are
/// meaningless.
pub struct ArenaFill {
	// Null while measuring
	base:   *mut u8,
	// The allocated block
	layout: Layout,
	// Bytes used from base, padding included
	len:    usize,
	// The largest alignment pushed
	align:  usize,
	failed: bool,
}

impl Arena {
	/// Start filling an arena of `capacity` bytes. `None` if the firmware is out of memory.
	///
	/// [`ArenaFill::used`] gives the capacity that fits the same pushes again.
	pub fn new_with_capacity(capacity: usize) -> Option<ArenaFill> {
		ArenaFill::alloc(Layout::from_size_align(capacity, 4).ok()?)
	}

	/// Build an arena of exactly the size needed with `fill`, which pushes everything it holds and
	/// returns the handles it wants to keep. The second run's result is returned.
	///
	/// `fill` runs twice, first to measure and then to write, so it must push the same things in
	/// the same order both times, and any work it does is done twice. Being `Fn`, it can't consume
	/// or change what it captures, but it can still read things that change between calls, such as
	/// the time: keep it to pushing data you already have. For data that's costly to produce, or
	/// whose size you know, use [`new_with_capacity`](Self::new_with_capacity) instead.
	///
	/// `None` if the firmware is out of memory, or the two runs differ.
	pub fn new_exact<R>(fill: impl Fn(&mut ArenaFill) -> R) -> Option<(Self, R)> {
		let mut measure = ArenaFill::measuring();
		drop(fill(&mut measure));
		if measure.failed {
			return None;
		}
		let mut write =
			ArenaFill::alloc(Layout::from_size_align(measure.len, measure.align).ok()?)?;
		let result = fill(&mut write);
		if write.len != measure.len {
			return None;
		}
		Some((write.finish()?, result))
	}

	/// The value at `handle`.
	///
	/// # Panics
	/// If `handle` is from a different arena and doesn't fit this one.
	#[track_caller]
	pub fn get<T: AnyBitPattern>(&self, handle: Handle<T>) -> &T {
		self.try_get(handle).expect(FOREIGN)
	}

	/// The value at `handle`, or `None` if it's from a different arena and doesn't fit this one.
	pub fn try_get<T: AnyBitPattern>(&self, handle: Handle<T>) -> Option<&T> {
		self.fits(handle.offset, Some(size_of::<T>()), align_of::<T>())
			// SAFETY: checked in bounds and aligned; every byte is initialised, and any bits are a T
			.then(|| unsafe { self.get_unchecked(handle) })
	}

	/// The value at `handle`, without checking it fits.
	///
	/// # Safety
	/// `handle` must lie within this arena. All handles from this arena are safe, any from others
	/// may cause out-of-bounds reads and are almost certainly logically incorrect otherwise.
	pub unsafe fn get_unchecked<T: AnyBitPattern>(&self, handle: Handle<T>) -> &T {
		// SAFETY: in bounds and aligned, per the caller; the block is zeroed, so initialised
		unsafe { &*self.ptr(handle).cast::<T>() }
	}

	/// The slice at `handle`.
	///
	/// # Panics
	/// If `handle` is from a different arena and doesn't fit this one.
	#[track_caller]
	pub fn slice<T: AnyBitPattern>(&self, handle: Handle<[T]>) -> &[T] {
		self.try_slice(handle).expect(FOREIGN)
	}

	/// The slice at `handle`, or `None` if it's from a different arena and doesn't fit this one.
	pub fn try_slice<T: AnyBitPattern>(&self, handle: Handle<[T]>) -> Option<&[T]> {
		let size = size_of::<T>().checked_mul(handle.len as usize);
		self.fits(handle.offset, size, align_of::<T>())
			// SAFETY: as for try_get, for every element
			.then(|| unsafe { self.slice_unchecked(handle) })
	}

	/// The slice at `handle`, without checking it fits.
	///
	/// # Safety
	/// As for [`get_unchecked`](Self::get_unchecked), for every element.
	pub unsafe fn slice_unchecked<T: AnyBitPattern>(&self, handle: Handle<[T]>) -> &[T] {
		// SAFETY: as for get_unchecked
		unsafe { core::slice::from_raw_parts(self.ptr(handle).cast::<T>(), handle.len as usize) }
	}

	/// The string at `handle`.
	///
	/// # Panics
	/// If `handle` is from a different arena and doesn't fit this one.
	#[track_caller]
	pub fn str(&self, handle: Handle<CStr>) -> &CStr {
		self.try_str(handle).expect(FOREIGN)
	}

	/// The string at `handle`, or `None` if it's from a different arena and doesn't fit this one.
	pub fn try_str(&self, handle: Handle<CStr>) -> Option<&CStr> {
		let len = handle.len as usize;
		if !self.fits(handle.offset, Some(len), 1) {
			return None;
		}
		// SAFETY: checked in bounds, and every byte is initialised
		let bytes = unsafe { core::slice::from_raw_parts(self.ptr(handle), len) };
		(bytes.iter().position(|&b| b == 0) == len.checked_sub(1))
			// SAFETY: in bounds, ending in its only NUL
			.then(|| unsafe { self.str_unchecked(handle) })
	}

	/// The string at `handle`, without checking it fits.
	///
	/// # Safety
	/// `handle` must lie within this arena and end in its only NUL, as any string handle from this
	/// arena does.
	pub unsafe fn str_unchecked(&self, handle: Handle<CStr>) -> &CStr {
		// SAFETY: in bounds and NUL-terminated with no other NUL, per the caller
		unsafe {
			CStr::from_bytes_with_nul_unchecked(core::slice::from_raw_parts(
				self.ptr(handle),
				handle.len as usize,
			))
		}
	}

	/// Where `handle` points, for building structures that point into the arena. Only meaningful
	/// for a handle from this arena.
	pub fn ptr<T: ?Sized>(&self, handle: Handle<T>) -> *const u8 {
		self.ptr.as_ptr().wrapping_add(handle.offset as usize)
	}

	/// The arena's size in bytes.
	pub fn size(&self) -> usize {
		self.layout.size()
	}

	fn fits(&self, offset: u32, size: Option<usize>, align: usize) -> bool {
		let offset = offset as usize;
		let in_bounds = size
			.and_then(|size| offset.checked_add(size))
			.is_some_and(|end| end <= self.layout.size());
		in_bounds
			&& self
				.ptr
				.as_ptr()
				.wrapping_add(offset)
				.addr()
				.is_multiple_of(align)
	}
}

impl Drop for Arena {
	fn drop(&mut self) {
		// SAFETY: allocated by ArenaFill::alloc with this layout
		unsafe { PebbleAlloc.deallocate(self.ptr, self.layout) };
	}
}

impl ArenaFill {
	fn new(base: *mut u8, layout: Layout) -> Self {
		Self {
			base,
			layout,
			len: 0,
			align: 1,
			failed: false,
		}
	}

	pub(crate) fn measuring() -> Self {
		Self::new(ptr::null_mut(), Layout::new::<()>())
	}

	fn alloc(layout: Layout) -> Option<Self> {
		let block = PebbleAlloc.allocate(layout).ok()?.cast::<u8>();
		// Zeroed so every byte is initialised: Arena::get() can then read any bytes a handle points
		// at
		// SAFETY: the block is layout.size() bytes
		unsafe { block.write_bytes(0, layout.size()) };
		Some(Self::new(block.as_ptr(), layout))
	}

	/// The finished arena. `None` if anything pushed didn't fit.
	pub fn finish(self) -> Option<Arena> {
		if self.failed || self.is_measuring() {
			return None;
		}
		let this = ManuallyDrop::new(self);
		Some(Arena {
			// SAFETY: not measuring, so base is the allocated block
			ptr:    unsafe { NonNull::new_unchecked(this.base) },
			layout: this.layout,
		})
	}

	/// Whether this is [`Arena::new_exact`]'s first run, which only measures: nothing is written,
	/// and [`ptr`](Self::ptr) points nowhere.
	pub fn is_measuring(&self) -> bool {
		self.base.is_null()
	}

	/// The capacity for [`Arena::new_with_capacity`] to fit everything pushed so far: the bytes
	/// used, accounting for worst-case padding.
	pub fn used(&self) -> usize {
		self.len + self.align.saturating_sub(4)
	}

	/// Put `value` in the arena.
	pub fn push<T: Copy>(&mut self, value: T) -> Handle<T> {
		let offset = self.put(Layout::new::<T>(), (&raw const value).cast());
		Self::handle(offset, 1)
	}

	/// Put a copy of `values` in the arena.
	pub fn push_slice<T: Copy>(&mut self, values: &[T]) -> Handle<[T]> {
		let offset = match Layout::array::<T>(values.len()) {
			Ok(layout) => self.put(layout, values.as_ptr().cast()),
			Err(_) => self.fail(),
		};
		Self::handle(offset, values.len())
	}

	/// Put a copy of `s` in the arena.
	pub fn push_c_str(&mut self, s: &CStr) -> Handle<CStr> {
		let bytes = s.to_bytes_with_nul();
		let offset = self.put(Layout::for_value(bytes), bytes.as_ptr());
		Self::handle(offset, bytes.len())
	}

	/// Put a copy of `s` in the arena as a C string, dropping any NUL bytes.
	pub fn push_str(&mut self, s: &str) -> Handle<CStr> {
		let len = s.bytes().filter(|&b| b != 0).count() + 1;
		let offset = match Layout::array::<u8>(len) {
			Ok(layout) => self.reserve(layout),
			Err(_) => self.fail(),
		};
		if let Some(dest) = offset.and_then(|offset| self.dest(offset)) {
			for (i, b) in s.bytes().filter(|&b| b != 0).enumerate() {
				// SAFETY: reserve() made room for len bytes, and i < len - 1; the last stays the
				// zeroed NUL
				unsafe { dest.add(i).write(b) };
			}
		}
		Self::handle(offset, len)
	}

	// Room for `len` Ts, left zeroed, to write through ptr() while writing
	pub(crate) fn reserve_slice<T>(&mut self, len: usize) -> Handle<[T]> {
		let offset = match Layout::array::<T>(len) {
			Ok(layout) => self.reserve(layout),
			Err(_) => self.fail(),
		};
		Self::handle(offset, len)
	}

	/// Where `handle` points in the block being filled, for building structures that point into
	/// the arena. Points nowhere while [measuring](Self::is_measuring): only store it.
	pub fn ptr<T: ?Sized>(&self, handle: Handle<T>) -> *const u8 {
		self.base.wrapping_add(handle.offset as usize).cast_const()
	}

	// Reserve room for `piece` and copy its bytes from `src` while writing
	fn put(&mut self, piece: Layout, src: *const u8) -> Option<usize> {
		let offset = self.reserve(piece)?;
		if let Some(dest) = self.dest(offset) {
			// SAFETY: reserve() made room, and src is piece.size() readable bytes
			unsafe { ptr::copy_nonoverlapping(src, dest, piece.size()) };
		}
		Some(offset)
	}

	// The offset of room for `piece`, aligned in memory. None, failing the arena, if it doesn't fit
	fn reserve(&mut self, piece: Layout) -> Option<usize> {
		let base = self.base.addr();
		let start = base
			.checked_add(self.len)
			.and_then(|at| at.checked_next_multiple_of(piece.align()))
			.map(|at| at - base);
		let end = start.and_then(|start| start.checked_add(piece.size()));
		match (start, end) {
			(Some(start), Some(end))
				if !self.failed && (self.is_measuring() || end <= self.layout.size()) =>
			{
				self.len = end;
				self.align = self.align.max(piece.align());
				Some(start)
			}
			_ => self.fail(),
		}
	}

	// The block at a reserved `offset`. None while measuring
	fn dest(&self, offset: usize) -> Option<*mut u8> {
		// SAFETY: reserve() checked offset is within the block
		(!self.is_measuring()).then(|| unsafe { self.base.add(offset) })
	}

	fn fail(&mut self) -> Option<usize> {
		self.failed = true;
		None
	}

	fn handle<T: ?Sized>(offset: Option<usize>, len: usize) -> Handle<T> {
		let (offset, len) = offset.map_or((0, 0), |offset| (offset, len));
		Handle {
			offset: offset as u32,
			len:    len as u32,
			_ty:    PhantomData,
		}
	}
}

impl Drop for ArenaFill {
	fn drop(&mut self) {
		if !self.is_measuring() {
			// SAFETY: allocated by alloc() with this layout
			unsafe { PebbleAlloc.deallocate(NonNull::new_unchecked(self.base), self.layout) };
		}
	}
}
