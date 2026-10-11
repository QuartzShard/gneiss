//! [`SyncCell`], for state shared between callbacks.
use core::{cell::Cell, ops::Deref};

/// A [`Cell`] that can be held in a `static`, for state shared between callbacks. It derefs to the
/// `Cell`, so it has all of `Cell`'s methods.
///
/// ```no_run
/// # use gneiss::cell::SyncCell;
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
