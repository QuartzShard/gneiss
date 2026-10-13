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
	ffi,
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

/// Crate internal `Box<dyn FnMut>`, smaller than a vtable approach, for a closure gneiss calls
/// through this handle, such as a click handler. The call and drop shims are written into the handle
/// at runtime, so they need no load-time relocation, and a closure that captures nothing needs no
/// allocation.
///
/// A closure called by the firmware, which only hands back a data pointer, needs [`ErasedCallback`].
pub(crate) struct ErasedFn<'f, S: Sig, Header = ()> {
	ptr:  ErasedPtr<Header>, // -> BoxState<Header, F>
	// rust-call spreads the argument tuple over registers instead of passing a pointer to it
	call: for<'a, 'b> unsafe extern "rust-call" fn(ErasedPtr<Header>, S::Args<'a, 'b>) -> S::Output,
	drop: Option<unsafe fn(ErasedPtr<Header>)>,
	_f:   PhantomData<&'f ()>,
}

/// Crate internal `Box<dyn FnMut>` for a closure the firmware calls back: the firmware is handed
/// [`trampoline`](Self::trampoline), a C function made per closure type by the signature's
/// [`CallbackSig`], and [`as_raw`](Self::as_raw) as its data, the pair a firmware registration takes.
/// The trampoline gets the closure back from [`ErasedPtr`] alone, with [`ErasedPtr::call`] and
/// [`ErasedPtr::free`]. Always allocates: the firmware holds the allocation's address.
pub(crate) struct ErasedCallback<'f, S: CallbackSig, Header> {
	ptr:        ErasedPtr<Header>, // -> BoxState<Header, F>
	trampoline: unsafe extern "C" fn(*mut ffi::c_void),
	drop:       Option<unsafe fn(ErasedPtr<Header>)>,
	_f:         PhantomData<(&'f (), fn(S))>,
}

/// The allocation behind an [`ErasedFn`] or [`ErasedCallback`]. The header comes first, so a pointer to
/// the whole is also a pointer to the header, readable without knowing `F`. `fun` is only reached by
/// casting back to `BoxState<Header, F>` where `F` is known: its offset depends on `F`'s alignment.
#[repr(C)]
struct BoxState<Header, F> {
	header: Header, // Metadata from the closure's owner
	fun:    F,      // The closure and its captures, inline
}

/// The signature of an [`ErasedFn`] or [`ErasedCallback`]. `'b` is a short borrow of a context living
/// for `'a`, such as `&'b mut Nav<'a>`
pub(crate) trait Sig {
	type Args<'a: 'b, 'b>: Tuple;
	type Output;
}

/// A signature whose closures the firmware calls back, through a C trampoline made per closure
/// type. Tying the trampoline to the signature means [`ErasedCallback::new`] instantiates it with the
/// same `F` it boxes, so the two can't be mismatched.
pub(crate) trait CallbackSig: Sig + Sized {
	/// The firmware's callback for an `F`, given an [`ErasedCallback::as_raw`] pointer as its data.
	///
	/// # Safety
	/// `data` must be a live [`ErasedCallback`]'s allocation holding an `F`.
	unsafe extern "C" fn trampoline<F: Callable<Self>>(data: *mut ffi::c_void);
}

/// Calling an `F` with a [`Sig`]'s arguments. One blanket impl per signature, over the matching
/// `FnMut`.
pub(crate) trait Callable<S: Sig> {
	fn call<'a: 'b, 'b>(&mut self, args: S::Args<'a, 'b>) -> S::Output;
}

/// Box `fun` behind `header`, returning its pointer and the drop that frees it. A `BoxState` that is
/// a ZST isn't allocated: the pointer is dangling, aligned for it, and there's nothing to drop.
fn boxed<Header, F>(
	header: Header,
	fun: F,
) -> (ErasedPtr<Header>, Option<unsafe fn(ErasedPtr<Header>)>) {
	let (ptr, _) = Box::into_raw_with_allocator(Box::new_in(BoxState { header, fun }, PebbleAlloc));
	let needs_drop =
		size_of::<BoxState<Header, F>>() != 0 || mem::needs_drop::<BoxState<Header, F>>();
	(
		// SAFETY: from a Box, so not null
		ErasedPtr(unsafe { NonNull::new_unchecked(ptr) }.cast()),
		needs_drop.then_some(ErasedPtr::<Header>::free::<F> as unsafe fn(ErasedPtr<Header>)),
	)
}

impl<'f, S: Sig, Header> ErasedFn<'f, S, Header> {
	pub(crate) fn new<F: Callable<S> + 'f>(header: Header, f: F) -> Self {
		let (ptr, drop) = boxed(header, f);
		Self {
			ptr,
			call: ErasedPtr::<Header>::call_shim::<S, F>,
			drop,
			_f: PhantomData,
		}
	}

	pub(crate) fn call<'a: 'b, 'b>(&mut self, args: S::Args<'a, 'b>) -> S::Output {
		// SAFETY: the allocation and call were paired in new, and &mut self keeps the allocation live
		// and excludes any other borrow of the closure
		unsafe { (self.call)(self.ptr, args) }
	}
}

impl<S: Sig, Header> Drop for ErasedFn<'_, S, Header> {
	fn drop(&mut self) {
		if let Some(drop) = self.drop {
			// SAFETY: the allocation and drop were paired in new, and this is the only drop
			unsafe { drop(self.ptr) };
		}
	}
}

impl<'f, S: CallbackSig, Header> ErasedCallback<'f, S, Header> {
	pub(crate) fn new<F: Callable<S> + 'f>(header: Header, f: F) -> Self {
		let (ptr, drop) = boxed(header, f);
		Self {
			ptr,
			trampoline: S::trampoline::<F>,
			drop,
			_f: PhantomData,
		}
	}

	/// The C function to register with the firmware, alongside [`as_raw`](Self::as_raw).
	pub(crate) fn trampoline(&self) -> unsafe extern "C" fn(*mut ffi::c_void) {
		self.trampoline
	}

	/// The header. The closure is only ever borrowed alone, so it stays readable while that runs.
	pub(crate) fn header(&self) -> &Header {
		// SAFETY: &self keeps the allocation live, and nothing borrows the header mutably
		unsafe { self.ptr.header() }
	}

	/// The allocation's pointer as a firmware callback's data, for the trampoline to turn back into
	/// an [`ErasedPtr`] with [`ErasedPtr::from_raw`].
	pub(crate) fn as_raw(&self) -> *mut ffi::c_void {
		self.ptr.0.as_ptr().cast()
	}
}

impl<S: CallbackSig, Header> Drop for ErasedCallback<'_, S, Header> {
	fn drop(&mut self) {
		if let Some(drop) = self.drop {
			// SAFETY: the allocation and drop were paired in new, and this is the only drop
			unsafe { drop(self.ptr) };
		}
	}
}

/// Points to an [`ErasedFn`]'s or [`ErasedCallback`]'s allocation: reaches the header without knowing
/// the closure's type, and the closure itself where the type is known.
#[repr(transparent)]
pub(crate) struct ErasedPtr<Header>(NonNull<Header>);
impl<Header> Clone for ErasedPtr<Header> {
	fn clone(&self) -> Self {
		*self
	}
}
impl<Header> Copy for ErasedPtr<Header> {}
impl<Header> ErasedPtr<Header> {
	/// The pointer back from a firmware callback's data.
	///
	/// # Safety
	/// `raw` must come from [`ErasedCallback::as_raw`] on an `ErasedCallback<_, _, Header>`.
	pub(crate) unsafe fn from_raw(raw: *mut ffi::c_void) -> Self {
		// SAFETY: ErasedCallback::as_raw only ever gives a non-null pointer
		Self(unsafe { NonNull::new_unchecked(raw.cast()) })
	}

	/// The header.
	///
	/// # Safety
	/// The allocation must stay live for `'a`, and nothing may borrow the header mutably meanwhile.
	pub(crate) unsafe fn header<'a>(self) -> &'a Header {
		// SAFETY: the header is at the start of the allocation, which the caller keeps live
		unsafe { self.0.as_ref() }
	}

	/// Call the closure, borrowing it alone so the header stays readable.
	///
	/// # Safety
	/// The allocation must be live and hold an `F`, with no other borrow of the closure while it
	/// runs.
	pub(crate) unsafe fn call<S: Sig, F: Callable<S>>(self, args: S::Args<'_, '_>) -> S::Output {
		// SAFETY: the caller vouches this is a live BoxState<Header, F> with its closure unborrowed
		let f = unsafe { &mut (*self.0.cast::<BoxState<Header, F>>().as_ptr()).fun };
		f.call(args)
	}

	/// Drop the header and closure, and free the allocation.
	///
	/// # Safety
	/// The allocation must hold an `F`, be live with no borrow into it, and never be used again.
	pub(crate) unsafe fn free<F>(self) {
		// SAFETY: the caller vouches this is the BoxState<Header, F> made by `boxed`, freed once
		drop(unsafe {
			Box::from_raw_in(self.0.cast::<BoxState<Header, F>>().as_ptr(), PebbleAlloc)
		});
	}

	/// [`call`](Self::call) as an [`ErasedFn`] stores it: a method, so it can use the `rust-call`
	/// ABI, which takes only `self` and a tuple.
	unsafe extern "rust-call" fn call_shim<S: Sig, F: Callable<S>>(
		self,
		args: S::Args<'_, '_>,
	) -> S::Output {
		// SAFETY: ErasedFn::new paired this shim with its BoxState<Header, F>, and ErasedFn::call's
		// &mut self excludes any other borrow of the closure
		unsafe { self.call::<S, F>(args) }
	}
}
