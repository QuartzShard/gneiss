//! Typed versions of the data the firmware hands to service handlers, and misc Type-level tools.
pub mod accel;
pub mod battery;
pub mod compass;
pub mod connection;
pub mod dict;
pub mod focus;
pub mod health;
pub mod time;
pub mod touch;
pub mod wakeup;

extern crate alloc;
use alloc::boxed::Box;
use core::{
	cell::Cell,
	marker::{PhantomData, Tuple},
	mem,
	ops::Deref,
	ptr::NonNull,
};

use crate::alloc::PebbleAlloc;

/// A [`Cell`] that can be held in a `static`, for state shared between callbacks. It derefs to the
/// `Cell`, so it has all of `Cell`'s methods.
///
/// ```no_run
/// # use gneiss::SyncCell;
/// static PRESSES: SyncCell<u32> = SyncCell::new(0);
///
/// fn on_press() {
///     PRESSES.set(PRESSES.get() + 1);
/// }
/// ```
///
/// It is [`Sync`] because PebbleOS runs all of an app's code on a single task, so a shared
/// reference never reaches another thread. It is not an atomic: read-modify-write atomics never
/// complete on the Pebble Time 2.
#[repr(transparent)]
pub struct SyncCell<T: ?Sized>(Cell<T>);

// SAFETY: PebbleOS runs all app code on a single task.
unsafe impl<T: ?Sized> Sync for SyncCell<T> {}

impl<T> SyncCell<T> {
	pub const fn new(value: T) -> Self {
		Self(Cell::new(value))
	}
}

impl<T: ?Sized> Deref for SyncCell<T> {
	type Target = Cell<T>;

	fn deref(&self) -> &Cell<T> {
		&self.0
	}
}

impl<T: Default> Default for SyncCell<T> {
	fn default() -> Self {
		Self::new(T::default())
	}
}

/// One allocation holding a typed header `H` and an erased value `T`, freed by an erased drop when
/// this drops. `T` is only known where the box was made, and in shims made there.
pub(crate) struct ErasedBox<'t, H = ()> {
	ptr:  ErasedPtr<H>, // -> BoxState<H, T>
	drop: Option<unsafe fn(ErasedPtr<H>)>,
	_t:   PhantomData<&'t ()>,
}

/// An [`ErasedBox`]'s allocation. The header comes first, so a pointer to the whole is also a
/// pointer to the header, readable without knowing `T`. `value` is only reached by casting back to
/// `BoxState<H, T>` where `T` is known: its offset depends on `T`'s alignment.
#[repr(C)]
pub(crate) struct BoxState<H, T> {
	header: H, // Metadata from the box's owner
	value:  T, // The erased value, such as a closure and its captures, inline
}

impl<'t, H> ErasedBox<'t, H> {
	pub(crate) fn new<T: 't>(header: H, value: T) -> Self {
		unsafe fn drop_box<H, T>(ptr: ErasedPtr<H>) {
			// SAFETY: ptr is the BoxState<H, T> made in new, dropped once
			drop(unsafe { Box::from_raw_in(ptr.0.cast::<BoxState<H, T>>().as_ptr(), PebbleAlloc) });
		}

		// No allocation when BoxState<H, T> is a ZST: the pointer is dangling, aligned for it
		let (ptr, _) =
			Box::into_raw_with_allocator(Box::new_in(BoxState { header, value }, PebbleAlloc));
		let needs_drop = size_of::<BoxState<H, T>>() != 0 || mem::needs_drop::<BoxState<H, T>>();

		Self {
			// SAFETY: from a Box, so not null
			ptr:  ErasedPtr(unsafe { NonNull::new_unchecked(ptr) }.cast()),
			drop: needs_drop.then_some(drop_box::<H, T> as unsafe fn(ErasedPtr<H>)),
			_t:   PhantomData,
		}
	}
}

impl<H> Drop for ErasedBox<'_, H> {
	fn drop(&mut self) {
		if let Some(drop) = self.drop {
			// SAFETY: ptr and drop were paired in new, and this is the only drop
			unsafe { drop(self.ptr) };
		}
	}
}

/// Crate internal utility for `Box<dyn Fn>` shaped things, smaller than a vtable approach: an
/// [`ErasedBox`] whose value is the closure, and the shim that calls it.
pub(crate) struct ErasedFn<'f, S: Sig, H = ()> {
	state: ErasedBox<'f, H>,
	// rust-call spreads the argument tuple over registers instead of passing a pointer to it
	call:  for<'a, 'b> unsafe extern "rust-call" fn(ErasedPtr<H>, S::Args<'a, 'b>) -> S::Output,
	_sig:  PhantomData<fn(S)>,
}

/// The signature of an [`ErasedFn`]. `'b` is a short borrow of a context living for `'a`, such as
/// `&'b mut Nav<'a>`
pub(crate) trait Sig {
	type Args<'a: 'b, 'b>: Tuple;
	type Output;
}

/// Calling an `F` with a [`Sig`]'s arguments. One blanket impl per signature, over the matching
/// `FnMut`.
pub(crate) trait Callable<S: Sig> {
	fn call<'a: 'b, 'b>(&mut self, args: S::Args<'a, 'b>) -> S::Output;
}

impl<'f, S: Sig, H> ErasedFn<'f, S, H> {
	pub(crate) fn new<F: Callable<S> + 'f>(header: H, f: F) -> Self {
		Self {
			state: ErasedBox::new(header, f),
			call:  ErasedPtr::<H>::call::<S, F>,
			_sig:  PhantomData,
		}
	}

	pub(crate) fn call<'a: 'b, 'b>(&mut self, args: S::Args<'a, 'b>) -> S::Output {
		// SAFETY: the box and call were paired in new, and &mut self keeps the box live
		unsafe { (self.call)(self.state.ptr, args) }
	}
}

/// Points to an [`ErasedBox`]'s `BoxState<H, T>`. An [`ErasedFn`]'s call shim is a method on it so
/// it can use the `rust-call` ABI, which takes only `self` and a tuple.
#[repr(transparent)]
pub(crate) struct ErasedPtr<H>(NonNull<H>);
impl<H> Clone for ErasedPtr<H> {
	fn clone(&self) -> Self {
		*self
	}
}
impl<H> Copy for ErasedPtr<H> {}
impl<H> ErasedPtr<H> {
	unsafe extern "rust-call" fn call<S: Sig, F: Callable<S>>(
		self,
		args: S::Args<'_, '_>,
	) -> S::Output {
		// SAFETY: the pointer is the BoxState<H, F> made in ErasedBox::new, live until drop. Only the
		// value is borrowed, so the header stays readable while it runs
		let f = unsafe { &mut (*self.0.cast::<BoxState<H, F>>().as_ptr()).value };
		f.call(args)
	}
}
