//! `AppMessage` dictionary layer: keys, a read/write dictionary handle, tuple
//! iteration, and conversion of tuples to and from Rust values.

use core::{ffi::CStr, marker::PhantomData, ptr};

use crate::sys;

/// From name declarations, or a specific index into a `name[N]` declaration.
#[derive(Clone, Copy, Debug)]
pub struct SingleKey {
	index: u32,
}

impl SingleKey {
	/// Usually emitted as a `message_keys::<name>` const by the build tooling.
	pub const fn new(index: u32) -> Self {
		Self { index }
	}
}

/// From `name[N]` declarations.
#[derive(Clone, Copy, Debug)]
pub struct ArrayKey {
	base: u32,
	len:  u32,
}

impl ArrayKey {
	/// Usually emitted as a `message_keys::<name>` const by the build tooling.
	pub const fn new(base: u32, len: u32) -> Self {
		Self { base, len }
	}

	pub fn get(&self, i: u32) -> Option<SingleKey> {
		if i < self.len {
			Some(SingleKey {
				index: self.base + i,
			})
		} else {
			None
		}
	}
}

/// Failures when fetching a value from a dictionary.
pub enum FetchError {
	Missing,
	WrongType,
	OutOfBounds,
}

bitflags::bitflags! {
	/// Empty means `DICT_OK`.
	pub struct DictionaryError : u8 {
		const NOT_ENOUGH_STORAGE = sys::DictionaryResult_DICT_NOT_ENOUGH_STORAGE;
		const INVALID_ARGS = sys::DictionaryResult_DICT_INVALID_ARGS;
		const INTERNAL_INCONSISTENCY = sys::DictionaryResult_DICT_INTERNAL_INCONSISTENCY;
		const MALLOC_FAILED = sys::DictionaryResult_DICT_MALLOC_FAILED;
	}
}
impl From<sys::DictionaryResult> for DictionaryError {
	fn from(value: sys::DictionaryResult) -> Self {
		Self::from_bits_retain(value)
	}
}

/// A dictionary behind a raw pointer valid for `'a`. `get`/`iter` read on
/// `&self`; `set` is `pub(crate)`, so outside the crate only `Outbox` writes.
pub struct DictHandle<'a> {
	inner: *mut sys::DictionaryIterator,
	_a:    PhantomData<&'a mut sys::DictionaryIterator>,
}

impl DictHandle<'_> {
	/// # Safety
	/// `ptr` must be a valid, non-null `DictionaryIterator` for the whole of `'a`.
	pub unsafe fn from_raw(ptr: *mut sys::DictionaryIterator) -> Self {
		debug_assert!(!ptr.is_null());
		Self {
			inner: ptr,
			_a:    PhantomData,
		}
	}

	/// Fetch a `T: FromTuple` by its key.
	pub fn get<'t, T: FromTuple<'t>>(&'t self, key: SingleKey) -> Result<T, FetchError> {
		// SAFETY: inner is a valid dict iterator; dict_find takes it by const ptr.
		let ptr: *mut sys::Tuple = unsafe { sys::dict_find(self.inner, key.index) };

		// SAFETY: dict_find returns NULL or a valid Tuple, alive for 't.
		let tuple: &'t sys::Tuple = unsafe { ptr.as_ref() }.ok_or(FetchError::Missing)?;

		let value = TupleValue::try_from(tuple).map_err(|_| FetchError::WrongType)?;
		T::from_tuple(value).ok_or(FetchError::WrongType)
	}

	/// Fetch the `index`th slot of an array key.
	pub fn get_array_slot<'t, T: FromTuple<'t>>(
		&'t self,
		key: ArrayKey,
		index: u32,
	) -> Result<T, FetchError> {
		self.get(key.get(index).ok_or(FetchError::OutOfBounds)?)
	}

	/// Iterate every tuple. Items borrow `self`.
	pub fn iter(&self) -> Refs<'_> {
		Refs {
			// SAFETY: inner is valid; DictionaryIterator is Copy, so this takes a
			// private copy whose cursor we advance without touching `self`.
			iter:    unsafe { *self.inner },
			started: false,
			_a:      PhantomData,
		}
	}

	/// Write a value under `key`.
	pub(crate) fn set<T: ToTuple + ?Sized>(
		&mut self,
		key: SingleKey,
		value: &T,
	) -> Result<(), DictionaryError> {
		// SAFETY: inner is valid; the transient &mut lives only for this write,
		// which calls neither begin nor send, so nothing aliases it.
		let err = value
			.to_tuple()
			.write(unsafe { self.inner.as_mut_unchecked() }, key.index);
		if err.is_empty() { Ok(()) } else { Err(err) }
	}
}

/// Iterator over a dictionary's tuples, borrowing the [`DictHandle`]. Walks a
/// private copy of the iterator, so the handle's cursor is untouched.
pub struct Refs<'a> {
	iter:    sys::DictionaryIterator,
	started: bool,
	_a:      PhantomData<&'a sys::DictionaryIterator>,
}

impl<'a> IntoIterator for &'a DictHandle<'_> {
	type IntoIter = Refs<'a>;
	type Item = &'a sys::Tuple;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a> Iterator for Refs<'a> {
	type Item = &'a sys::Tuple;

	fn next(&mut self) -> Option<&'a sys::Tuple> {
		let tuple = if self.started {
			// SAFETY: our own iterator copy; advances only the local cursor.
			unsafe { sys::dict_read_next(&raw mut self.iter) }
		} else {
			self.started = true;
			unsafe { sys::dict_read_first(&raw mut self.iter) }
		};
		// SAFETY: NULL at the end, else a valid Tuple in the buffer, alive for 'a.
		unsafe { tuple.as_ref() }
	}
}

/// A decoded tuple: the wire tag and value together. `Bytes`/`CStr` borrow the
/// message buffer; scalars are by value.
pub enum TupleValue<'a> {
	U8(u8),
	U16(u16),
	U32(u32),
	I8(i8),
	I16(i16),
	I32(i32),
	Bytes(&'a [u8]),
	CStr(&'a CStr),
}

/// A `sys::Tuple` whose tag or length matches no representable value.
pub struct MalformedTuple;

impl<'a> TryFrom<&'a sys::Tuple> for TupleValue<'a> {
	type Error = MalformedTuple;

	fn try_from(t: &'a sys::Tuple) -> Result<Self, MalformedTuple> {
		let value: *const u8 = ptr::addr_of!(t.value).cast();

		// SAFETY: the value spans `length` bytes in the buffer-backed tuple,
		// alive for 'a; a byte slice needs no alignment. `length` is trusted as
		// the firmware provides it.
		let bytes = unsafe { core::slice::from_raw_parts(value, t.length as usize) };

		Ok(match t.type_() {
			sys::TupleType_TUPLE_UINT => match bytes.len() {
				1 => Self::U8(bytes[0]),
				2 => Self::U16(u16::from_le_bytes(
					bytes.try_into().map_err(|_| MalformedTuple)?,
				)),
				4 => Self::U32(u32::from_le_bytes(
					bytes.try_into().map_err(|_| MalformedTuple)?,
				)),
				_ => return Err(MalformedTuple),
			},
			sys::TupleType_TUPLE_INT => match bytes.len() {
				1 => Self::I8(bytes[0] as i8),
				2 => Self::I16(i16::from_le_bytes(
					bytes.try_into().map_err(|_| MalformedTuple)?,
				)),
				4 => Self::I32(i32::from_le_bytes(
					bytes.try_into().map_err(|_| MalformedTuple)?,
				)),
				_ => return Err(MalformedTuple),
			},
			sys::TupleType_TUPLE_BYTE_ARRAY => Self::Bytes(bytes),
			// Stored length includes the NUL; reject if not exactly one-NUL-terminated.
			sys::TupleType_TUPLE_CSTRING => {
				Self::CStr(CStr::from_bytes_with_nul(bytes).map_err(|_| MalformedTuple)?)
			}
			_ => return Err(MalformedTuple),
		})
	}
}

impl TupleValue<'_> {
	/// Write this value into an outbox iterator under `key`.
	fn write(&self, iter: &mut sys::DictionaryIterator, key: u32) -> DictionaryError {
		// SAFETY: `iter` is a valid outbox iterator; a borrowed variant's slice
		// or cstring is valid for the duration of the call.
		let result = unsafe {
			match *self {
				Self::U8(x) => sys::dict_write_uint8(iter, key, x),
				Self::U16(x) => sys::dict_write_uint16(iter, key, x),
				Self::U32(x) => sys::dict_write_uint32(iter, key, x),
				Self::I8(x) => sys::dict_write_int8(iter, key, x),
				Self::I16(x) => sys::dict_write_int16(iter, key, x),
				Self::I32(x) => sys::dict_write_int32(iter, key, x),
				Self::Bytes(b) => sys::dict_write_data(iter, key, b.as_ptr(), b.len() as u16),
				Self::CStr(c) => sys::dict_write_cstring(iter, key, c.as_ptr()),
			}
		};
		DictionaryError::from_bits_retain(result)
	}
}

/// Build a value from a decoded [`TupleValue`]; `None` on a tag/width mismatch.
pub trait FromTuple<'a>: Sized {
	fn from_tuple(v: TupleValue<'a>) -> Option<Self>;
}

/// Describe a value as a [`TupleValue`] ready to write.
pub trait ToTuple {
	fn to_tuple(&self) -> TupleValue<'_>;
}

macro_rules! impl_scalar_tuple {
	($ty:ty, $variant:ident) => {
		impl FromTuple<'_> for $ty {
			fn from_tuple(v: TupleValue<'_>) -> Option<Self> {
				match v {
					TupleValue::$variant(x) => Some(x),
					_ => None,
				}
			}
		}
		impl ToTuple for $ty {
			fn to_tuple(&self) -> TupleValue<'_> {
				TupleValue::$variant(*self)
			}
		}
	};
}

impl_scalar_tuple!(u8, U8);
impl_scalar_tuple!(u16, U16);
impl_scalar_tuple!(u32, U32);
impl_scalar_tuple!(i8, I8);
impl_scalar_tuple!(i16, I16);
impl_scalar_tuple!(i32, I32);

impl<'a> FromTuple<'a> for &'a [u8] {
	fn from_tuple(v: TupleValue<'a>) -> Option<Self> {
		match v {
			TupleValue::Bytes(b) => Some(b),
			_ => None,
		}
	}
}

impl<'a> FromTuple<'a> for &'a CStr {
	fn from_tuple(v: TupleValue<'a>) -> Option<Self> {
		match v {
			TupleValue::CStr(c) => Some(c),
			_ => None,
		}
	}
}

impl<'a> FromTuple<'a> for &'a str {
	fn from_tuple(v: TupleValue<'a>) -> Option<Self> {
		<&CStr>::from_tuple(v)?.to_str().ok()
	}
}

impl FromTuple<'_> for bool {
	// Pebble has no bool tuple; `TupletBoolean` is written as a uint8.
	fn from_tuple(v: TupleValue<'_>) -> Option<Self> {
		u8::from_tuple(v).map(|x| x != 0)
	}
}

impl ToTuple for bool {
	fn to_tuple(&self) -> TupleValue<'_> {
		TupleValue::U8(u8::from(*self))
	}
}

impl ToTuple for [u8] {
	fn to_tuple(&self) -> TupleValue<'_> {
		TupleValue::Bytes(self)
	}
}

impl ToTuple for CStr {
	fn to_tuple(&self) -> TupleValue<'_> {
		TupleValue::CStr(self)
	}
}
