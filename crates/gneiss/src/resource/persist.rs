//! Persistent storage for data between app launches.
//!
//! Every key is declared with [`persist_keys!`](crate::persist_keys), once per app, which numbers
//! them so no two collide and gives each its type.
//!
//! Each key is its own flash write and is atomic. Separate keys have their own r/w cycles and their
//! own headers (~12B). Writing a large struct repeatedly to change one field is hard-wearing on the
//! flash, however - so consider keeping oft-changed settings in their own keys.
//! General structs can be atomically saved/loaded via [`make_persistible`](crate::make_persistible),
//! or any other [`Pod`] value through [`AsBytes`].
//!
//! ```no_run,standalone_crate
//! use gneiss::resource::persist::{Persist, Recover};
//!
//! gneiss::persist_keys! {
//!     SETTINGS_VOLUME: i32,
//!     SETTINGS_PALETTE: i32,
//!     SETTINGS_VIBRATE: bool,
//!     LAUNCHES: i32,
//!     HIGH_SCORES: HighScores,
//!     /// How many notes are saved: a tail has no end of its own, so keep track of what you wrote
//!     NOTE_COUNT: i32,
//!     /// One key per saved note, as many as the app saves
//!     NOTES: [Note; ..],
//! }
//!
//! #[gneiss::make_persistible]
//! struct HighScores {
//!     top: [u32; 5],
//! }
//!
//! #[gneiss::make_persistible]
//! struct Note {
//!     created: u32,
//!     text:    [u8; 60],
//! }
//!
//! fn set_volume(volume: i32) {
//!     // Saves this setting alone
//!     let _ = volume.persist(SETTINGS_VOLUME);
//! }
//!
//! fn launched() {
//!     // Unset reads as 0
//!     let _ = (i32::recover_or_default(LAUNCHES) + 1).persist(LAUNCHES);
//! }
//!
//! fn scored(score: u32) {
//!     // Highest first
//!     let mut scores = HighScores::recover(HIGH_SCORES).unwrap_or(HighScores { top: [0; 5] });
//!     if let Some(rank) = scores.top.iter().position(|&top| score > top) {
//!         // Shift the lower scores down, dropping the last
//!         let last = scores.top.len() - 1;
//!         scores.top.copy_within(rank..last, rank + 1);
//!         scores.top[rank] = score;
//!         // The whole table, in one write
//!         let _ = scores.persist(HIGH_SCORES);
//!     }
//! }
//!
//! fn add_note(note: Note) {
//!     let count = i32::recover_or_default(NOTE_COUNT);
//!     if let Some(key) = NOTES.get(count as u32)
//!         && note.persist(key).is_ok()
//!     {
//!         let _ = (count + 1).persist(NOTE_COUNT);
//!     }
//! }
//!
//! fn notes() -> impl Iterator<Item = Note> {
//!     (0..i32::recover_or_default(NOTE_COUNT) as u32)
//!         .filter_map(|index| Note::recover(NOTES.get(index)?))
//! }
//! ```

use core::marker::PhantomData;

use bytemuck::Pod;

use crate::{status::StatusError, sys};

/// How many bytes of persistent storage the app may use: 4 KB on the frozen-SDK watches, and what
/// the firmware reports on the rest. Each key costs about 12 bytes of overhead.
pub fn max_persist() -> usize {
	#[cfg(pbl_sdk_frozen)]
	return 4096;
	#[cfg(not(pbl_sdk_frozen))]
	unsafe {
		sys::persist_get_max_size()
	}
}

/// A key holding a `T`, declared with [`persist_keys!`](crate::persist_keys).
pub struct PersistKey<T: Persist + ?Sized> {
	key:  u32,
	data: PhantomData<T>,
}

impl<T: Persist + ?Sized> Clone for PersistKey<T> {
	fn clone(&self) -> Self {
		*self
	}
}
/// Ignore the Copy-ness of T, since access to the T is a Cloning-load from the firmware and copying
/// this handle is like copying a ref
impl<T: Persist + ?Sized> Copy for PersistKey<T> {}

impl<T: Persist + ?Sized> PersistKey<T> {
	/// Called by the code [`persist_keys!`](crate::persist_keys) generates.
	#[doc(hidden)]
	pub const fn __new(key: u32) -> Self {
		Self {
			key,
			data: PhantomData,
		}
	}

	/// Read the data in this slot as `U`, analogous to `ptr::cast()`. Useful for migrating data
	/// from older versiond of your app.
	/// Reading a key as another type than it holds can't cause undefined behaviour, but can return
	/// a garbage value or error if T -> U is not a valid cast.
	pub const fn retype<U: Persist + ?Sized>(self) -> PersistKey<U> {
		PersistKey::__new(self.key)
	}

	/// The raw key the firmware stores under.
	pub const fn key(self) -> u32 {
		self.key
	}

	/// Whether the key holds a value. An empty one doesn't count: the firmware stores none.
	pub fn exists(self) -> bool {
		unsafe { sys::persist_exists(self.key) }
	}

	/// Ignores firmware return, no-ops "not set"
	pub fn delete(self) {
		unsafe { sys::persist_delete(self.key) };
	}

	/// How many bytes the key holds. `DoesNotExist` if unset.
	pub fn size(self) -> Result<u32, StatusError> {
		StatusError::check(unsafe { sys::persist_get_size(self.key) })
	}
}

/// Store a value in persistent storage.
pub trait Persist {
	/// `Ok(bytes written)`, or `StatusError`
	fn persist(&self, key: PersistKey<Self>) -> Result<u32, StatusError>;
}

/// Read back a value of a fixed size.
///
/// Stored data outlives the app version that wrote it, so a key can hold bytes written as any type.
/// An implementation must be sound whatever bytes it reads back: it may return a meaningless value,
/// but never an invalid one. [`PersistKey::retype`] relies on this too.
pub trait Recover: Persist + Sized {
	/// The value at `key`, or `None` if it's unset.
	fn recover(key: PersistKey<Self>) -> Option<Self>;

	/// The value at `key`, or [`Default`] if it's unset. Skips telling the two apart where the
	/// firmware's own answer for an unset key is already the default, so it can do less work than
	/// [`recover`](Self::recover).
	fn recover_or_default(key: PersistKey<Self>) -> Self
	where
		Self: Default,
	{
		Self::recover(key).unwrap_or_default()
	}
}

/// Read back a value without a fixed size, into the caller's buffer. Like [`Recover`], an
/// implementation must be sound whatever bytes it reads back.
pub trait RecoverSlice: Persist {
	/// The value at `key`, read into `buf`, as a view of it and the bytes read. `DoesNotExist` if
	/// unset. A value longer than `buf` is cut short.
	fn recover(key: PersistKey<Self>, buf: &mut [u8]) -> Result<&Self, StatusError>;
}

impl Persist for bool {
	fn persist(&self, key: PersistKey<Self>) -> Result<u32, StatusError> {
		StatusError::check(unsafe { sys::persist_write_bool(key.key, *self) })
	}
}

impl Recover for bool {
	fn recover(key: PersistKey<Self>) -> Option<Self> {
		key.exists().then(|| Self::recover_or_default(key))
	}

	/// One read: an unset key reads as `false`.
	fn recover_or_default(key: PersistKey<Self>) -> Self {
		// Not persist_read_bool: it hands back the first stored byte as-is, which is not a valid
		// rust `bool` if the key holds anything but one. This is the idea, but truncating
		let mut byte = 0u8;
		unsafe { sys::persist_read_data(key.key, (&raw mut byte).cast(), 1) };
		byte != 0
	}
}

impl Persist for i32 {
	fn persist(&self, key: PersistKey<Self>) -> Result<u32, StatusError> {
		StatusError::check(unsafe { sys::persist_write_int(key.key, *self) })
	}
}
impl Recover for i32 {
	fn recover(key: PersistKey<Self>) -> Option<Self> {
		key.exists().then(|| Self::recover_or_default(key))
	}

	/// One read: the firmware gives 0 for an unset key.
	fn recover_or_default(key: PersistKey<Self>) -> Self {
		unsafe { sys::persist_read_int(key.key) }
	}
}

/// Writing an empty slice deletes the key: it reads back as unset.
impl Persist for [u8] {
	fn persist(&self, key: PersistKey<Self>) -> Result<u32, StatusError> {
		if self.len() <= sys::PERSIST_DATA_MAX_LENGTH as usize {
			StatusError::check(unsafe {
				sys::persist_write_data(key.key, self.as_ptr().cast(), self.len())
			})
		} else {
			Err(StatusError::OutOfRange)
		}
	}
}

impl RecoverSlice for [u8] {
	fn recover(key: PersistKey<Self>, buf: &mut [u8]) -> Result<&Self, StatusError> {
		let bytes = StatusError::check(unsafe {
			sys::persist_read_data(key.key, buf.as_mut_ptr().cast(), buf.len())
		})?;
		// The firmware reads at most buf.len()
		let read = buf.get(..bytes as usize).ok_or(StatusError::Internal)?;
		Ok(read)
	}
}

impl Persist for core::ffi::CStr {
	fn persist(&self, key: PersistKey<Self>) -> Result<u32, StatusError> {
		if self.count_bytes() < sys::PERSIST_STRING_MAX_LENGTH as usize {
			StatusError::check(unsafe { sys::persist_write_string(key.key, self.as_ptr()) })
		} else {
			Err(StatusError::OutOfRange)
		}
	}
}

impl RecoverSlice for core::ffi::CStr {
	/// `Range` if `buf` is empty, with no room for the NUL. The bytes read include the NUL.
	fn recover(key: PersistKey<Self>, buf: &mut [u8]) -> Result<&Self, StatusError> {
		// The firmware terminates the string by writing buf[bytes - 1], unchecked
		if buf.is_empty() {
			return Err(StatusError::OutOfRange);
		}
		let bytes = StatusError::check(unsafe {
			sys::persist_read_string(key.key, buf.as_mut_ptr().cast(), buf.len())
		})?;
		let read = buf.get(..bytes as usize).ok_or(StatusError::Internal)?;
		// The firmware NUL-terminated it
		let string =
			core::ffi::CStr::from_bytes_until_nul(read).map_err(|_| StatusError::Internal)?;
		Ok(string)
	}
}

/// A [`Pod`] value stored as its bytes, in one key: one write, so it can't be left half updated.
/// At most [`PERSIST_DATA_MAX_LENGTH`](sys::PERSIST_DATA_MAX_LENGTH) bytes, checked at compile time.
/// For a struct of your own, [`make_persistible`](crate::make_persistible) does the same without the
/// wrapper.
///
/// Reading back stored bytes is only sound because every bit pattern of a `Pod` type is valid. A
/// change to `T`'s size between app versions reads back as unset.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct AsBytes<T: Pod>(pub T);

impl<T: Pod> Default for AsBytes<T> {
	fn default() -> Self {
		Self(T::zeroed())
	}
}

impl<T: Pod> Persist for AsBytes<T> {
	fn persist(&self, key: PersistKey<Self>) -> Result<u32, StatusError> {
		__persist_pod(&self.0, key.key)
	}
}

impl<T: Pod> Recover for AsBytes<T> {
	/// `None` if unset, or stored at another size.
	fn recover(key: PersistKey<Self>) -> Option<Self> {
		__recover_pod(key.key).map(Self)
	}
}

/// Write `value`'s bytes to `key`. Called by [`AsBytes`] and the code
/// [`make_persistible`](crate::make_persistible) generates.
#[doc(hidden)]
pub fn __persist_pod<T: Pod>(value: &T, key: u32) -> Result<u32, StatusError> {
	const {
		assert!(
			size_of::<T>() <= sys::PERSIST_DATA_MAX_LENGTH as usize,
			"persisted value is over PERSIST_DATA_MAX_LENGTH"
		);
	}
	let bytes = bytemuck::bytes_of(value);
	StatusError::check(unsafe { sys::persist_write_data(key, bytes.as_ptr().cast(), bytes.len()) })
}

/// The `T` stored at `key`. `None` if it's unset or stored at another size. Called by [`AsBytes`]
/// and the code [`make_persistible`](crate::make_persistible) generates.
#[doc(hidden)]
pub fn __recover_pod<T: Pod>(key: u32) -> Option<T> {
	let size = size_of::<T>();
	if usize::try_from(unsafe { sys::persist_get_size(key) }) != Ok(size) {
		return None;
	}
	let mut value = T::zeroed();
	let bytes = bytemuck::bytes_of_mut(&mut value);
	let read = unsafe { sys::persist_read_data(key, bytes.as_mut_ptr().cast(), size) };
	(usize::try_from(read) == Ok(size)).then_some(value)
}

/// A run of keys each holding a `T`, declared as `NAME: [T; len]` in
/// [`persist_keys!`](crate::persist_keys).
pub struct PersistBlock<T: Persist + ?Sized> {
	first: u32,
	len:   u32,
	data:  PhantomData<T>,
}

impl<T: Persist + ?Sized> Clone for PersistBlock<T> {
	fn clone(&self) -> Self {
		*self
	}
}
impl<T: Persist + ?Sized> Copy for PersistBlock<T> {}

impl<T: Persist + ?Sized> PersistBlock<T> {
	/// Called by the code [`persist_keys!`](crate::persist_keys) generates.
	#[doc(hidden)]
	pub const fn __new(first: u32, len: u32) -> Self {
		Self {
			first,
			len,
			data: PhantomData,
		}
	}

	/// How many keys the block holds.
	pub const fn len(self) -> u32 {
		self.len
	}

	pub const fn is_empty(self) -> bool {
		self.len == 0
	}

	/// The `index`th key, or `None` past the end.
	pub const fn get(self, index: u32) -> Option<PersistKey<T>> {
		if index < self.len {
			Some(PersistKey::__new(self.first + index))
		} else {
			None
		}
	}

	/// Every key in the block, in order.
	pub fn keys(self) -> impl Iterator<Item = PersistKey<T>> {
		(self.first..self.first + self.len).map(PersistKey::__new)
	}
}

/// Every key after the rest each holding a `T`, declared last as `NAME: [T; ..]` in
/// [`persist_keys!`](crate::persist_keys), for data saved under as many keys as it needs.
pub struct PersistTail<T: Persist + ?Sized> {
	first: u32,
	data:  PhantomData<T>,
}

impl<T: Persist + ?Sized> Clone for PersistTail<T> {
	fn clone(&self) -> Self {
		*self
	}
}
impl<T: Persist + ?Sized> Copy for PersistTail<T> {}

impl<T: Persist + ?Sized> PersistTail<T> {
	/// Called by the code [`persist_keys!`](crate::persist_keys) generates.
	#[doc(hidden)]
	pub const fn __new(first: u32) -> Self {
		Self {
			first,
			data: PhantomData,
		}
	}

	/// The `index`th key, or `None` past the end of the key space.
	pub const fn get(self, index: u32) -> Option<PersistKey<T>> {
		match self.first.checked_add(index) {
			Some(key) => Some(PersistKey::__new(key)),
			None => None,
		}
	}
}
