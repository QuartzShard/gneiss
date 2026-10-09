//! SPDX-License-Identifier: Apache-2.0
//! This module is generated from `pebble.h` by bindgen + a shim-mapping step in the `regen-sys` xtask. Do not hand-edit
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct __BindgenBitfieldUnit<Storage> {
	storage: Storage,
}
impl<Storage> __BindgenBitfieldUnit<Storage> {
	#[inline]
	pub const fn new(storage: Storage) -> Self {
		Self { storage }
	}
}
impl<Storage> __BindgenBitfieldUnit<Storage>
where
	Storage: AsRef<[u8]> + AsMut<[u8]>,
{
	#[inline]
	fn extract_bit(byte: u8, index: usize) -> bool {
		let bit_index = if cfg!(target_endian = "big") {
			7 - (index % 8)
		} else {
			index % 8
		};
		let mask = 1 << bit_index;
		byte & mask == mask
	}

	#[inline]
	pub fn get_bit(&self, index: usize) -> bool {
		debug_assert!(index / 8 < self.storage.as_ref().len());
		let byte_index = index / 8;
		let byte = self.storage.as_ref()[byte_index];
		Self::extract_bit(byte, index)
	}

	#[inline]
	pub unsafe fn raw_get_bit(this: *const Self, index: usize) -> bool {
		debug_assert!(index / 8 < core::mem::size_of::<Storage>());
		let byte_index = index / 8;
		let byte = unsafe {
			*(core::ptr::addr_of!((*this).storage) as *const u8).offset(byte_index as isize)
		};
		Self::extract_bit(byte, index)
	}

	#[inline]
	fn change_bit(byte: u8, index: usize, val: bool) -> u8 {
		let bit_index = if cfg!(target_endian = "big") {
			7 - (index % 8)
		} else {
			index % 8
		};
		let mask = 1 << bit_index;
		if val { byte | mask } else { byte & !mask }
	}

	#[inline]
	pub fn set_bit(&mut self, index: usize, val: bool) {
		debug_assert!(index / 8 < self.storage.as_ref().len());
		let byte_index = index / 8;
		let byte = &mut self.storage.as_mut()[byte_index];
		*byte = Self::change_bit(*byte, index, val);
	}

	#[inline]
	pub unsafe fn raw_set_bit(this: *mut Self, index: usize, val: bool) {
		debug_assert!(index / 8 < core::mem::size_of::<Storage>());
		let byte_index = index / 8;
		let byte = unsafe {
			(core::ptr::addr_of_mut!((*this).storage) as *mut u8).offset(byte_index as isize)
		};
		unsafe { *byte = Self::change_bit(*byte, index, val) };
	}

	#[inline]
	pub fn get(&self, bit_offset: usize, bit_width: u8) -> u64 {
		debug_assert!(bit_width <= 64);
		debug_assert!(bit_offset / 8 < self.storage.as_ref().len());
		debug_assert!((bit_offset + (bit_width as usize) + 7) / 8 <= self.storage.as_ref().len(),);
		if bit_width == 0 {
			return 0;
		}
		let mut val = 0u64;
		let storage = self.storage.as_ref();
		let start_byte = bit_offset / 8;
		let bit_shift = bit_offset % 8;
		let bytes_needed = (bit_width as usize + bit_shift + 7) / 8;
		if cfg!(target_endian = "big") {
			for i in 0..bytes_needed {
				val |= (storage[start_byte + i].reverse_bits() as u64) << (i * 8);
			}
		} else {
			for i in 0..bytes_needed {
				val |= (storage[start_byte + i] as u64) << (i * 8);
			}
		}
		val >>= bit_shift;
		if bit_width < 64 {
			val &= (1u64 << bit_width) - 1;
		}
		if cfg!(target_endian = "big") {
			val = val.reverse_bits() >> (64 - bit_width as usize);
		}
		val
	}

	#[inline]
	pub unsafe fn raw_get(this: *const Self, bit_offset: usize, bit_width: u8) -> u64 {
		debug_assert!(bit_width <= 64);
		debug_assert!(bit_offset / 8 < core::mem::size_of::<Storage>());
		debug_assert!(
			(bit_offset + (bit_width as usize) + 7) / 8 <= core::mem::size_of::<Storage>(),
		);
		if bit_width == 0 {
			return 0;
		}
		let mut val = 0u64;
		let start_byte = bit_offset / 8;
		let bit_shift = bit_offset % 8;
		let bytes_needed = (bit_width as usize + bit_shift + 7) / 8;
		let storage_ptr = unsafe { core::ptr::addr_of!((*this).storage) as *const u8 };
		if cfg!(target_endian = "big") {
			for i in 0..bytes_needed {
				let byte = unsafe { *storage_ptr.add(start_byte + i) };
				val |= (byte.reverse_bits() as u64) << (i * 8);
			}
		} else {
			for i in 0..bytes_needed {
				let byte = unsafe { *storage_ptr.add(start_byte + i) };
				val |= (byte as u64) << (i * 8);
			}
		}
		val >>= bit_shift;
		if bit_width < 64 {
			val &= (1u64 << bit_width) - 1;
		}
		if cfg!(target_endian = "big") {
			val = val.reverse_bits() >> (64 - bit_width as usize);
		}
		val
	}

	#[inline]
	pub fn set(&mut self, bit_offset: usize, bit_width: u8, val: u64) {
		debug_assert!(bit_width <= 64);
		debug_assert!(bit_offset / 8 < self.storage.as_ref().len());
		debug_assert!((bit_offset + (bit_width as usize) + 7) / 8 <= self.storage.as_ref().len(),);
		if bit_width == 0 {
			return;
		}
		let mut val = val;
		if bit_width < 64 {
			val &= (1u64 << bit_width) - 1;
		}
		if cfg!(target_endian = "big") {
			val = val.reverse_bits() >> (64 - bit_width as usize);
		}
		let storage = self.storage.as_mut();
		let start_byte = bit_offset / 8;
		let bit_shift = bit_offset % 8;
		let bytes_needed = (bit_width as usize + bit_shift + 7) / 8;
		val <<= bit_shift;
		let field_mask = if bit_width as usize + bit_shift >= 64 {
			!0u64 << bit_shift
		} else {
			((1u64 << bit_width) - 1) << bit_shift
		};
		for i in 0..bytes_needed {
			let byte_val = (val >> (i * 8)) as u8;
			let byte_mask = (field_mask >> (i * 8)) as u8;
			if cfg!(target_endian = "big") {
				let byte = storage[start_byte + i].reverse_bits();
				let new_byte = (byte & !byte_mask) | (byte_val & byte_mask);
				storage[start_byte + i] = new_byte.reverse_bits();
			} else {
				storage[start_byte + i] =
					(storage[start_byte + i] & !byte_mask) | (byte_val & byte_mask);
			}
		}
	}

	#[inline]
	pub unsafe fn raw_set(this: *mut Self, bit_offset: usize, bit_width: u8, val: u64) {
		debug_assert!(bit_width <= 64);
		debug_assert!(bit_offset / 8 < core::mem::size_of::<Storage>());
		debug_assert!(
			(bit_offset + (bit_width as usize) + 7) / 8 <= core::mem::size_of::<Storage>(),
		);
		if bit_width == 0 {
			return;
		}
		let mut val = val;
		if bit_width < 64 {
			val &= (1u64 << bit_width) - 1;
		}
		if cfg!(target_endian = "big") {
			val = val.reverse_bits() >> (64 - bit_width as usize);
		}
		let start_byte = bit_offset / 8;
		let bit_shift = bit_offset % 8;
		let bytes_needed = (bit_width as usize + bit_shift + 7) / 8;
		val <<= bit_shift;
		let field_mask = if bit_width as usize + bit_shift >= 64 {
			!0u64 << bit_shift
		} else {
			((1u64 << bit_width) - 1) << bit_shift
		};
		let storage_ptr = unsafe { core::ptr::addr_of_mut!((*this).storage) as *mut u8 };
		for i in 0..bytes_needed {
			let byte_val = (val >> (i * 8)) as u8;
			let byte_mask = (field_mask >> (i * 8)) as u8;
			let byte_ptr = unsafe { storage_ptr.add(start_byte + i) };
			if cfg!(target_endian = "big") {
				let byte = unsafe { (*byte_ptr).reverse_bits() };
				let new_byte = (byte & !byte_mask) | (byte_val & byte_mask);
				unsafe { *byte_ptr = new_byte.reverse_bits() };
			} else {
				unsafe { *byte_ptr = (*byte_ptr & !byte_mask) | (byte_val & byte_mask) };
			}
		}
	}
}
/// Const-generic methods for efficient bitfield access when offset and width
/// are known at compile time.
impl<const N: usize> __BindgenBitfieldUnit<[u8; N]> {
	/// Get a field using const generics for compile-time optimization.
	/// Uses native word size operations when the field fits in usize.
	#[inline]
	pub const fn get_const<const BIT_OFFSET: usize, const BIT_WIDTH: u8>(&self) -> u64 {
		debug_assert!(BIT_WIDTH <= 64);
		debug_assert!(BIT_OFFSET / 8 < N);
		debug_assert!((BIT_OFFSET + (BIT_WIDTH as usize) + 7) / 8 <= N);
		if BIT_WIDTH == 0 {
			return 0;
		}
		let start_byte = BIT_OFFSET / 8;
		let bit_shift = BIT_OFFSET % 8;
		let bytes_needed = (BIT_WIDTH as usize + bit_shift + 7) / 8;
		if BIT_WIDTH as usize + bit_shift <= usize::BITS as usize {
			let mut val = 0usize;
			if cfg!(target_endian = "big") {
				let mut i = 0;
				while i < bytes_needed {
					val |= (self.storage[start_byte + i].reverse_bits() as usize) << (i * 8);
					i += 1;
				}
			} else {
				let mut i = 0;
				while i < bytes_needed {
					val |= (self.storage[start_byte + i] as usize) << (i * 8);
					i += 1;
				}
			}
			val >>= bit_shift;
			if (BIT_WIDTH as u32) < usize::BITS {
				val &= (1usize << BIT_WIDTH) - 1;
			}
			if cfg!(target_endian = "big") {
				val = val.reverse_bits() >> (usize::BITS as usize - BIT_WIDTH as usize);
			}
			val as u64
		} else {
			let mut val = 0u64;
			if cfg!(target_endian = "big") {
				let mut i = 0;
				while i < bytes_needed {
					val |= (self.storage[start_byte + i].reverse_bits() as u64) << (i * 8);
					i += 1;
				}
			} else {
				let mut i = 0;
				while i < bytes_needed {
					val |= (self.storage[start_byte + i] as u64) << (i * 8);
					i += 1;
				}
			}
			val >>= bit_shift;
			if BIT_WIDTH < 64 {
				val &= (1u64 << BIT_WIDTH) - 1;
			}
			if cfg!(target_endian = "big") {
				val = val.reverse_bits() >> (64 - BIT_WIDTH as usize);
			}
			val
		}
	}

	/// Set a field using const generics for compile-time optimization.
	/// Uses native word size operations when the field fits in usize.
	#[inline]
	pub fn set_const<const BIT_OFFSET: usize, const BIT_WIDTH: u8>(&mut self, val: u64) {
		debug_assert!(BIT_WIDTH <= 64);
		debug_assert!(BIT_OFFSET / 8 < N);
		debug_assert!((BIT_OFFSET + (BIT_WIDTH as usize) + 7) / 8 <= N);
		if BIT_WIDTH == 0 {
			return;
		}
		let start_byte = BIT_OFFSET / 8;
		let bit_shift = BIT_OFFSET % 8;
		let bytes_needed = (BIT_WIDTH as usize + bit_shift + 7) / 8;
		if BIT_WIDTH as usize + bit_shift <= usize::BITS as usize {
			let mut val = val as usize;
			if (BIT_WIDTH as u32) < usize::BITS {
				val &= (1usize << BIT_WIDTH) - 1;
			}
			if cfg!(target_endian = "big") {
				val = val.reverse_bits() >> (usize::BITS as usize - BIT_WIDTH as usize);
			}
			val <<= bit_shift;
			let field_mask = if BIT_WIDTH as usize + bit_shift >= usize::BITS as usize {
				!0usize << bit_shift
			} else {
				((1usize << BIT_WIDTH) - 1) << bit_shift
			};
			let mut i = 0;
			while i < bytes_needed {
				let byte_val = (val >> (i * 8)) as u8;
				let byte_mask = (field_mask >> (i * 8)) as u8;
				if cfg!(target_endian = "big") {
					let byte = self.storage[start_byte + i].reverse_bits();
					let new_byte = (byte & !byte_mask) | (byte_val & byte_mask);
					self.storage[start_byte + i] = new_byte.reverse_bits();
				} else {
					self.storage[start_byte + i] =
						(self.storage[start_byte + i] & !byte_mask) | (byte_val & byte_mask);
				}
				i += 1;
			}
		} else {
			let mut val = val;
			if BIT_WIDTH < 64 {
				val &= (1u64 << BIT_WIDTH) - 1;
			}
			if cfg!(target_endian = "big") {
				val = val.reverse_bits() >> (64 - BIT_WIDTH as usize);
			}
			val <<= bit_shift;
			let field_mask = if BIT_WIDTH as usize + bit_shift >= 64 {
				!0u64 << bit_shift
			} else {
				((1u64 << BIT_WIDTH) - 1) << bit_shift
			};
			let mut i = 0;
			while i < bytes_needed {
				let byte_val = (val >> (i * 8)) as u8;
				let byte_mask = (field_mask >> (i * 8)) as u8;
				if cfg!(target_endian = "big") {
					let byte = self.storage[start_byte + i].reverse_bits();
					let new_byte = (byte & !byte_mask) | (byte_val & byte_mask);
					self.storage[start_byte + i] = new_byte.reverse_bits();
				} else {
					self.storage[start_byte + i] =
						(self.storage[start_byte + i] & !byte_mask) | (byte_val & byte_mask);
				}
				i += 1;
			}
		}
	}

	/// Raw pointer get using const generics for compile-time optimization.
	/// Uses native word size operations when the field fits in usize.
	#[inline]
	pub const unsafe fn raw_get_const<const BIT_OFFSET: usize, const BIT_WIDTH: u8>(
		this: *const Self,
	) -> u64 {
		debug_assert!(BIT_WIDTH <= 64);
		debug_assert!(BIT_OFFSET / 8 < N);
		debug_assert!((BIT_OFFSET + (BIT_WIDTH as usize) + 7) / 8 <= N);
		if BIT_WIDTH == 0 {
			return 0;
		}
		let start_byte = BIT_OFFSET / 8;
		let bit_shift = BIT_OFFSET % 8;
		let bytes_needed = (BIT_WIDTH as usize + bit_shift + 7) / 8;
		let storage_ptr = unsafe { core::ptr::addr_of!((*this).storage) as *const u8 };
		if BIT_WIDTH as usize + bit_shift <= usize::BITS as usize {
			let mut val = 0usize;
			if cfg!(target_endian = "big") {
				let mut i = 0;
				while i < bytes_needed {
					let byte = unsafe { *storage_ptr.add(start_byte + i) };
					val |= (byte.reverse_bits() as usize) << (i * 8);
					i += 1;
				}
			} else {
				let mut i = 0;
				while i < bytes_needed {
					let byte = unsafe { *storage_ptr.add(start_byte + i) };
					val |= (byte as usize) << (i * 8);
					i += 1;
				}
			}
			val >>= bit_shift;
			if (BIT_WIDTH as u32) < usize::BITS {
				val &= (1usize << BIT_WIDTH) - 1;
			}
			if cfg!(target_endian = "big") {
				val = val.reverse_bits() >> (usize::BITS as usize - BIT_WIDTH as usize);
			}
			val as u64
		} else {
			let mut val = 0u64;
			if cfg!(target_endian = "big") {
				let mut i = 0;
				while i < bytes_needed {
					let byte = unsafe { *storage_ptr.add(start_byte + i) };
					val |= (byte.reverse_bits() as u64) << (i * 8);
					i += 1;
				}
			} else {
				let mut i = 0;
				while i < bytes_needed {
					let byte = unsafe { *storage_ptr.add(start_byte + i) };
					val |= (byte as u64) << (i * 8);
					i += 1;
				}
			}
			val >>= bit_shift;
			if BIT_WIDTH < 64 {
				val &= (1u64 << BIT_WIDTH) - 1;
			}
			if cfg!(target_endian = "big") {
				val = val.reverse_bits() >> (64 - BIT_WIDTH as usize);
			}
			val
		}
	}

	/// Raw pointer set using const generics for compile-time optimization.
	/// Uses native word size operations when the field fits in usize.
	#[inline]
	pub unsafe fn raw_set_const<const BIT_OFFSET: usize, const BIT_WIDTH: u8>(
		this: *mut Self,
		val: u64,
	) {
		debug_assert!(BIT_WIDTH <= 64);
		debug_assert!(BIT_OFFSET / 8 < N);
		debug_assert!((BIT_OFFSET + (BIT_WIDTH as usize) + 7) / 8 <= N);
		if BIT_WIDTH == 0 {
			return;
		}
		let start_byte = BIT_OFFSET / 8;
		let bit_shift = BIT_OFFSET % 8;
		let bytes_needed = (BIT_WIDTH as usize + bit_shift + 7) / 8;
		let storage_ptr = this.cast::<[u8; N]>().cast::<u8>();
		if BIT_WIDTH as usize + bit_shift <= usize::BITS as usize {
			let mut val = val as usize;
			if (BIT_WIDTH as u32) < usize::BITS {
				val &= (1usize << BIT_WIDTH) - 1;
			}
			if cfg!(target_endian = "big") {
				val = val.reverse_bits() >> (usize::BITS as usize - BIT_WIDTH as usize);
			}
			val <<= bit_shift;
			let field_mask = if BIT_WIDTH as usize + bit_shift >= usize::BITS as usize {
				!0usize << bit_shift
			} else {
				((1usize << BIT_WIDTH) - 1) << bit_shift
			};
			let mut i = 0;
			while i < bytes_needed {
				let byte_val = (val >> (i * 8)) as u8;
				let byte_mask = (field_mask >> (i * 8)) as u8;
				let byte_ptr = unsafe { storage_ptr.add(start_byte + i) };
				if cfg!(target_endian = "big") {
					let byte = unsafe { (*byte_ptr).reverse_bits() };
					let new_byte = (byte & !byte_mask) | (byte_val & byte_mask);
					unsafe { *byte_ptr = new_byte.reverse_bits() };
				} else {
					unsafe { *byte_ptr = (*byte_ptr & !byte_mask) | (byte_val & byte_mask) };
				}
				i += 1;
			}
		} else {
			let mut val = val;
			if BIT_WIDTH < 64 {
				val &= (1u64 << BIT_WIDTH) - 1;
			}
			if cfg!(target_endian = "big") {
				val = val.reverse_bits() >> (64 - BIT_WIDTH as usize);
			}
			val <<= bit_shift;
			let field_mask = if BIT_WIDTH as usize + bit_shift >= 64 {
				!0u64 << bit_shift
			} else {
				((1u64 << BIT_WIDTH) - 1) << bit_shift
			};
			let mut i = 0;
			while i < bytes_needed {
				let byte_val = (val >> (i * 8)) as u8;
				let byte_mask = (field_mask >> (i * 8)) as u8;
				let byte_ptr = unsafe { storage_ptr.add(start_byte + i) };
				if cfg!(target_endian = "big") {
					let byte = unsafe { (*byte_ptr).reverse_bits() };
					let new_byte = (byte & !byte_mask) | (byte_val & byte_mask);
					unsafe { *byte_ptr = new_byte.reverse_bits() };
				} else {
					unsafe { *byte_ptr = (*byte_ptr & !byte_mask) | (byte_val & byte_mask) };
				}
				i += 1;
			}
		}
	}
}
#[repr(C)]
#[derive(Default)]
pub struct __IncompleteArrayField<T>(::core::marker::PhantomData<T>, [T; 0]);
impl<T> __IncompleteArrayField<T> {
	#[inline]
	pub const fn new() -> Self {
		__IncompleteArrayField(::core::marker::PhantomData, [])
	}

	#[inline]
	pub fn as_ptr(&self) -> *const T {
		self as *const _ as *const T
	}

	#[inline]
	pub fn as_mut_ptr(&mut self) -> *mut T {
		self as *mut _ as *mut T
	}

	#[inline]
	pub unsafe fn as_slice(&self, len: usize) -> &[T] {
		::core::slice::from_raw_parts(self.as_ptr(), len)
	}

	#[inline]
	pub unsafe fn as_mut_slice(&mut self, len: usize) -> &mut [T] {
		::core::slice::from_raw_parts_mut(self.as_mut_ptr(), len)
	}
}
impl<T> ::core::fmt::Debug for __IncompleteArrayField<T> {
	fn fmt(&self, fmt: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
		fmt.write_str("__IncompleteArrayField")
	}
}
#[repr(C)]
pub struct __BindgenUnionField<T>(::core::marker::PhantomData<T>);
impl<T> __BindgenUnionField<T> {
	#[inline]
	pub const fn new() -> Self {
		__BindgenUnionField(::core::marker::PhantomData)
	}

	#[inline]
	pub const unsafe fn as_ref(&self) -> &T {
		unsafe { ::core::mem::transmute(self) }
	}

	#[inline]
	pub const unsafe fn as_mut(&mut self) -> &mut T {
		unsafe { ::core::mem::transmute(self) }
	}
}
impl<T> ::core::default::Default for __BindgenUnionField<T> {
	#[inline]
	fn default() -> Self {
		Self::new()
	}
}
impl<T> ::core::clone::Clone for __BindgenUnionField<T> {
	#[inline]
	fn clone(&self) -> Self {
		*self
	}
}
impl<T> ::core::marker::Copy for __BindgenUnionField<T> {}
impl<T> ::core::fmt::Debug for __BindgenUnionField<T> {
	fn fmt(&self, fmt: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
		fmt.write_str("__BindgenUnionField")
	}
}
impl<T> ::core::hash::Hash for __BindgenUnionField<T> {
	fn hash<H: ::core::hash::Hasher>(&self, _state: &mut H) {}
}
impl<T> ::core::cmp::PartialEq for __BindgenUnionField<T> {
	fn eq(&self, _other: &__BindgenUnionField<T>) -> bool {
		true
	}
}
impl<T> ::core::cmp::Eq for __BindgenUnionField<T> {}
pub const FONT_KEY_FONT_FALLBACK_INTERNAL: &::core::ffi::CStr =
	c"RESOURCE_ID_FONT_FALLBACK_INTERNAL";
pub const FONT_KEY_GOTHIC_18_BOLD: &::core::ffi::CStr = c"RESOURCE_ID_GOTHIC_18_BOLD";
pub const FONT_KEY_GOTHIC_24: &::core::ffi::CStr = c"RESOURCE_ID_GOTHIC_24";
pub const FONT_KEY_GOTHIC_09: &::core::ffi::CStr = c"RESOURCE_ID_GOTHIC_09";
pub const FONT_KEY_GOTHIC_14: &::core::ffi::CStr = c"RESOURCE_ID_GOTHIC_14";
pub const FONT_KEY_GOTHIC_14_BOLD: &::core::ffi::CStr = c"RESOURCE_ID_GOTHIC_14_BOLD";
pub const FONT_KEY_GOTHIC_18: &::core::ffi::CStr = c"RESOURCE_ID_GOTHIC_18";
pub const FONT_KEY_GOTHIC_24_BOLD: &::core::ffi::CStr = c"RESOURCE_ID_GOTHIC_24_BOLD";
pub const FONT_KEY_GOTHIC_28: &::core::ffi::CStr = c"RESOURCE_ID_GOTHIC_28";
pub const FONT_KEY_GOTHIC_28_BOLD: &::core::ffi::CStr = c"RESOURCE_ID_GOTHIC_28_BOLD";
pub const FONT_KEY_BITHAM_30_BLACK: &::core::ffi::CStr = c"RESOURCE_ID_BITHAM_30_BLACK";
pub const FONT_KEY_BITHAM_42_BOLD: &::core::ffi::CStr = c"RESOURCE_ID_BITHAM_42_BOLD";
pub const FONT_KEY_BITHAM_42_LIGHT: &::core::ffi::CStr = c"RESOURCE_ID_BITHAM_42_LIGHT";
pub const FONT_KEY_BITHAM_42_MEDIUM_NUMBERS: &::core::ffi::CStr =
	c"RESOURCE_ID_BITHAM_42_MEDIUM_NUMBERS";
pub const FONT_KEY_BITHAM_34_MEDIUM_NUMBERS: &::core::ffi::CStr =
	c"RESOURCE_ID_BITHAM_34_MEDIUM_NUMBERS";
pub const FONT_KEY_BITHAM_34_LIGHT_SUBSET: &::core::ffi::CStr =
	c"RESOURCE_ID_BITHAM_34_LIGHT_SUBSET";
pub const FONT_KEY_BITHAM_18_LIGHT_SUBSET: &::core::ffi::CStr =
	c"RESOURCE_ID_BITHAM_18_LIGHT_SUBSET";
pub const FONT_KEY_ROBOTO_CONDENSED_21: &::core::ffi::CStr = c"RESOURCE_ID_ROBOTO_CONDENSED_21";
pub const FONT_KEY_ROBOTO_BOLD_SUBSET_49: &::core::ffi::CStr = c"RESOURCE_ID_ROBOTO_BOLD_SUBSET_49";
pub const FONT_KEY_DROID_SERIF_28_BOLD: &::core::ffi::CStr = c"RESOURCE_ID_DROID_SERIF_28_BOLD";
pub const FONT_KEY_LECO_20_BOLD_NUMBERS: &::core::ffi::CStr = c"RESOURCE_ID_LECO_20_BOLD_NUMBERS";
pub const FONT_KEY_LECO_26_BOLD_NUMBERS_AM_PM: &::core::ffi::CStr =
	c"RESOURCE_ID_LECO_26_BOLD_NUMBERS_AM_PM";
pub const FONT_KEY_LECO_32_BOLD_NUMBERS: &::core::ffi::CStr = c"RESOURCE_ID_LECO_32_BOLD_NUMBERS";
pub const FONT_KEY_LECO_36_BOLD_NUMBERS: &::core::ffi::CStr = c"RESOURCE_ID_LECO_36_BOLD_NUMBERS";
pub const FONT_KEY_LECO_38_BOLD_NUMBERS: &::core::ffi::CStr = c"RESOURCE_ID_LECO_38_BOLD_NUMBERS";
pub const FONT_KEY_LECO_42_NUMBERS: &::core::ffi::CStr = c"RESOURCE_ID_LECO_42_NUMBERS";
pub const FONT_KEY_LECO_28_LIGHT_NUMBERS: &::core::ffi::CStr = c"RESOURCE_ID_LECO_28_LIGHT_NUMBERS";
pub const FONT_KEY_FONT_FALLBACK: &::core::ffi::CStr = c"RESOURCE_ID_FONT_FALLBACK";
pub const __bool_true_false_are_defined: u32 = 1;
pub const true_: u32 = 1;
pub const false_: u32 = 0;
pub const TRIG_MAX_RATIO: u32 = 65535;
pub const TRIG_MAX_ANGLE: u32 = 65536;
pub const TIMEZONE_NAME_LENGTH: u32 = 32;
pub const UUID_SIZE: u32 = 16;
pub const UUID_STRING_BUFFER_LENGTH: u32 = 39;
pub const APP_MESSAGE_INBOX_SIZE_MINIMUM: u32 = 124;
pub const APP_MESSAGE_OUTBOX_SIZE_MINIMUM: u32 = 636;
pub const kModdableCreationFlagLogInstrumentation: u32 = 1;
pub const kModdableCreationFlagDebug: u32 = 2;
pub const PERSIST_DATA_MAX_LENGTH: u32 = 256;
pub const PERSIST_STRING_MAX_LENGTH: u32 = 256;
pub const SMARTSTRAP_TIMEOUT_DEFAULT: u32 = 250;
pub const SMARTSTRAP_RAW_DATA_SERVICE_ID: u32 = 0;
pub const SMARTSTRAP_RAW_DATA_ATTRIBUTE_ID: u32 = 0;
pub const ANIMATION_NORMALIZED_MIN: u32 = 0;
pub const ANIMATION_NORMALIZED_MAX: u32 = 65535;
pub const NUM_ACTION_BAR_ITEMS: u32 = 3;
pub const SPEAKER_MAX_NOTES: u32 = 256;
pub const SPEAKER_MAX_TRACKS: u32 = 4;
pub const SPEAKER_MAX_SAMPLE_BYTES_TOTAL: u32 = 16384;
pub const TZ_LEN: u32 = 6;
pub const SECONDS_PER_MINUTE: u32 = 60;
pub const MINUTES_PER_HOUR: u32 = 60;
pub const SECONDS_PER_HOUR: u32 = 3600;
pub const HOURS_PER_DAY: u32 = 24;
pub const MINUTES_PER_DAY: u32 = 1440;
pub const SECONDS_PER_DAY: u32 = 86400;
pub type wchar_t = ::core::ffi::c_uint;
/** malloc
  -> Added in 0
gneiss: Vtable Slot 161, Offset 644*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.malloc")]
pub unsafe extern "C" fn malloc(arg1: ::core::ffi::c_uint) -> *mut ::core::ffi::c_void {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =644", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/** free
  -> Added in 0
gneiss: Vtable Slot 99, Offset 396*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.free")]
pub unsafe extern "C" fn free(arg1: *mut ::core::ffi::c_void) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =396", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/** realloc
  -> Added in 12
gneiss: Vtable Slot 323, Offset 1292*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.realloc")]
pub unsafe extern "C" fn realloc(
	arg1: *mut ::core::ffi::c_void,
	arg2: ::core::ffi::c_uint,
) -> *mut ::core::ffi::c_void {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1292", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
pub type int_least64_t = i64;
pub type uint_least64_t = u64;
pub type int_fast64_t = i64;
pub type uint_fast64_t = u64;
pub type int_least32_t = i32;
pub type uint_least32_t = u32;
pub type int_fast32_t = i32;
pub type uint_fast32_t = u32;
pub type int_least16_t = i16;
pub type uint_least16_t = u16;
pub type int_fast16_t = i16;
pub type uint_fast16_t = u16;
pub type int_least8_t = i8;
pub type uint_least8_t = u8;
pub type int_fast8_t = i8;
pub type uint_fast8_t = u8;
pub type intmax_t = ::core::ffi::c_longlong;
pub type uintmax_t = ::core::ffi::c_ulonglong;
/** memcpy
  -> Added in 0
gneiss: Vtable Slot 162, Offset 648*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.memcpy")]
pub unsafe extern "C" fn memcpy(
	arg1: *mut ::core::ffi::c_void,
	arg2: *const ::core::ffi::c_void,
	arg3: ::core::ffi::c_uint,
) -> *mut ::core::ffi::c_void {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =648", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/** memmove
  -> Added in 0
gneiss: Vtable Slot 163, Offset 652*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.memmove")]
pub unsafe extern "C" fn memmove(
	arg1: *mut ::core::ffi::c_void,
	arg2: *const ::core::ffi::c_void,
	arg3: ::core::ffi::c_uint,
) -> *mut ::core::ffi::c_void {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =652", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/** memset
  -> Added in 0
gneiss: Vtable Slot 164, Offset 656*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.memset")]
pub unsafe extern "C" fn memset(
	arg1: *mut ::core::ffi::c_void,
	arg2: ::core::ffi::c_int,
	arg3: ::core::ffi::c_uint,
) -> *mut ::core::ffi::c_void {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =656", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
pub type time_t = ::core::ffi::c_long;
///! Back button
pub const ButtonId_BUTTON_ID_BACK: ButtonId = 0;
///! Up button
pub const ButtonId_BUTTON_ID_UP: ButtonId = 1;
///! Select (middle) button
pub const ButtonId_BUTTON_ID_SELECT: ButtonId = 2;
///! Down button
pub const ButtonId_BUTTON_ID_DOWN: ButtonId = 3;
///! Total number of buttons
pub const ButtonId_NUM_BUTTONS: ButtonId = 4;
/**! Button ID values
! @see \ref click_recognizer_get_button_id()*/
pub type ButtonId = ::core::ffi::c_uchar;
/**! Get the ISO locale name for the language currently set on the watch
! @return A string containing the ISO locale name (e.g. "fr", "en_US", ...)
! @note It is possible for the locale to change while your app is running.
! And thus, two calls to i18n_get_system_locale may return different values.*/
/** i18n_get_system_locale
  -> Added in 26
gneiss: Vtable Slot 360, Offset 1440*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.i18n_get_system_locale")]
pub unsafe extern "C" fn i18n_get_system_locale() -> *const ::core::ffi::c_char {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1440", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///!< Unknown model
pub const WatchInfoModel_WATCH_INFO_MODEL_UNKNOWN: WatchInfoModel = 0;
///!< Original Pebble
pub const WatchInfoModel_WATCH_INFO_MODEL_PEBBLE_ORIGINAL: WatchInfoModel = 1;
///!< Pebble Steel
pub const WatchInfoModel_WATCH_INFO_MODEL_PEBBLE_STEEL: WatchInfoModel = 2;
///!< Pebble Time
pub const WatchInfoModel_WATCH_INFO_MODEL_PEBBLE_TIME: WatchInfoModel = 3;
///!< Pebble Time Steel
pub const WatchInfoModel_WATCH_INFO_MODEL_PEBBLE_TIME_STEEL: WatchInfoModel = 4;
///!< Pebble Time Round, 14mm lug size
pub const WatchInfoModel_WATCH_INFO_MODEL_PEBBLE_TIME_ROUND_14: WatchInfoModel = 5;
///!< Pebble Time Round, 20mm lug size
pub const WatchInfoModel_WATCH_INFO_MODEL_PEBBLE_TIME_ROUND_20: WatchInfoModel = 6;
///!< Pebble 2 HR
pub const WatchInfoModel_WATCH_INFO_MODEL_PEBBLE_2_HR: WatchInfoModel = 7;
///!< Pebble 2 SE
pub const WatchInfoModel_WATCH_INFO_MODEL_PEBBLE_2_SE: WatchInfoModel = 8;
///!< Pebble Time 2
pub const WatchInfoModel_WATCH_INFO_MODEL_PEBBLE_TIME_2: WatchInfoModel = 9;
///!< CoreDevices P2D (Pebble 2 Duo)
pub const WatchInfoModel_WATCH_INFO_MODEL_COREDEVICES_P2D: WatchInfoModel = 10;
///!< CoreDevices PT2 (Pebble Time 2)
pub const WatchInfoModel_WATCH_INFO_MODEL_COREDEVICES_PT2: WatchInfoModel = 11;
///!< CoreDevices PR2 (Pebble Round 2)
pub const WatchInfoModel_WATCH_INFO_MODEL_COREDEVICES_PR2: WatchInfoModel = 12;
pub const WatchInfoModel_WATCH_INFO_MODEL__MAX: WatchInfoModel = 13;
///! The different watch models.
pub type WatchInfoModel = ::core::ffi::c_uchar;
///!< Unknown color
pub const WatchInfoColor_WATCH_INFO_COLOR_UNKNOWN: WatchInfoColor = 0;
///!< Black
pub const WatchInfoColor_WATCH_INFO_COLOR_BLACK: WatchInfoColor = 1;
///!< White
pub const WatchInfoColor_WATCH_INFO_COLOR_WHITE: WatchInfoColor = 2;
///!< Red
pub const WatchInfoColor_WATCH_INFO_COLOR_RED: WatchInfoColor = 3;
///!< Orange
pub const WatchInfoColor_WATCH_INFO_COLOR_ORANGE: WatchInfoColor = 4;
///!< Gray
pub const WatchInfoColor_WATCH_INFO_COLOR_GRAY: WatchInfoColor = 5;
///!< Stainless Steel
pub const WatchInfoColor_WATCH_INFO_COLOR_STAINLESS_STEEL: WatchInfoColor = 6;
///!< Matte Black
pub const WatchInfoColor_WATCH_INFO_COLOR_MATTE_BLACK: WatchInfoColor = 7;
///!< Blue
pub const WatchInfoColor_WATCH_INFO_COLOR_BLUE: WatchInfoColor = 8;
///!< Green
pub const WatchInfoColor_WATCH_INFO_COLOR_GREEN: WatchInfoColor = 9;
///!< Pink
pub const WatchInfoColor_WATCH_INFO_COLOR_PINK: WatchInfoColor = 10;
///!< Time White
pub const WatchInfoColor_WATCH_INFO_COLOR_TIME_WHITE: WatchInfoColor = 11;
///!< Time Black
pub const WatchInfoColor_WATCH_INFO_COLOR_TIME_BLACK: WatchInfoColor = 12;
///!< Time Red
pub const WatchInfoColor_WATCH_INFO_COLOR_TIME_RED: WatchInfoColor = 13;
///!< Time Steel Silver
pub const WatchInfoColor_WATCH_INFO_COLOR_TIME_STEEL_SILVER: WatchInfoColor = 14;
///!< Time Steel Black
pub const WatchInfoColor_WATCH_INFO_COLOR_TIME_STEEL_BLACK: WatchInfoColor = 15;
///!< Time Steel Gold
pub const WatchInfoColor_WATCH_INFO_COLOR_TIME_STEEL_GOLD: WatchInfoColor = 16;
///!< Time Round 14mm lug size, Silver
pub const WatchInfoColor_WATCH_INFO_COLOR_TIME_ROUND_SILVER_14: WatchInfoColor = 17;
///!< Time Round 14mm lug size, Black
pub const WatchInfoColor_WATCH_INFO_COLOR_TIME_ROUND_BLACK_14: WatchInfoColor = 18;
///!< Time Round 20mm lug size, Silver
pub const WatchInfoColor_WATCH_INFO_COLOR_TIME_ROUND_SILVER_20: WatchInfoColor = 19;
///!< Time Round 20mm lug size, Black
pub const WatchInfoColor_WATCH_INFO_COLOR_TIME_ROUND_BLACK_20: WatchInfoColor = 20;
///!< Time Round 14mm lug size, Rose Gold
pub const WatchInfoColor_WATCH_INFO_COLOR_TIME_ROUND_ROSE_GOLD_14: WatchInfoColor = 21;
///!< Pebble 2 HR, Black / Charcoal
pub const WatchInfoColor_WATCH_INFO_COLOR_PEBBLE_2_HR_BLACK: WatchInfoColor = 25;
///!< Pebble 2 HR, Charcoal / Sorbet Green
pub const WatchInfoColor_WATCH_INFO_COLOR_PEBBLE_2_HR_LIME: WatchInfoColor = 27;
///!< Pebble 2 HR, Charcoal / Red
pub const WatchInfoColor_WATCH_INFO_COLOR_PEBBLE_2_HR_FLAME: WatchInfoColor = 28;
///!< Pebble 2 HR, White / Gray
pub const WatchInfoColor_WATCH_INFO_COLOR_PEBBLE_2_HR_WHITE: WatchInfoColor = 29;
///!< Pebble 2 HR, White / Turquoise
pub const WatchInfoColor_WATCH_INFO_COLOR_PEBBLE_2_HR_AQUA: WatchInfoColor = 30;
///!< Pebble 2 SE, Black / Charcoal
pub const WatchInfoColor_WATCH_INFO_COLOR_PEBBLE_2_SE_BLACK: WatchInfoColor = 24;
///!< Pebble 2 SE, White / Gray
pub const WatchInfoColor_WATCH_INFO_COLOR_PEBBLE_2_SE_WHITE: WatchInfoColor = 26;
///!< Pebble Time 2, Black
pub const WatchInfoColor_WATCH_INFO_COLOR_PEBBLE_TIME_2_BLACK: WatchInfoColor = 31;
///!< Pebble Time 2, Silver
pub const WatchInfoColor_WATCH_INFO_COLOR_PEBBLE_TIME_2_SILVER: WatchInfoColor = 32;
///!< Pebble Time 2, Gold
pub const WatchInfoColor_WATCH_INFO_COLOR_PEBBLE_TIME_2_GOLD: WatchInfoColor = 33;
///!< CoreDevices P2D, Black
pub const WatchInfoColor_WATCH_INFO_COLOR_COREDEVICES_P2D_BLACK: WatchInfoColor = 34;
///!< CoreDevices P2D, White
pub const WatchInfoColor_WATCH_INFO_COLOR_COREDEVICES_P2D_WHITE: WatchInfoColor = 35;
///!< CoreDevices PT2, Black/Grey
pub const WatchInfoColor_WATCH_INFO_COLOR_COREDEVICES_PT2_BLACK_GREY: WatchInfoColor = 36;
///!< CoreDevices PT2, Black/Red
pub const WatchInfoColor_WATCH_INFO_COLOR_COREDEVICES_PT2_BLACK_RED: WatchInfoColor = 37;
///!< CoreDevices PT2, Silver/Blue
pub const WatchInfoColor_WATCH_INFO_COLOR_COREDEVICES_PT2_SILVER_BLUE: WatchInfoColor = 38;
///!< CoreDevices PT2, Silver/Grey
pub const WatchInfoColor_WATCH_INFO_COLOR_COREDEVICES_PT2_SILVER_GREY: WatchInfoColor = 39;
///!< CoreDevices PR2, Black (20mm)
pub const WatchInfoColor_WATCH_INFO_COLOR_COREDEVICES_PR2_BLACK_20: WatchInfoColor = 40;
///!< CoreDevices PR2, Silver (20mm)
pub const WatchInfoColor_WATCH_INFO_COLOR_COREDEVICES_PR2_SILVER_20: WatchInfoColor = 41;
///!< CoreDevices PR2, Gold (14mm)
pub const WatchInfoColor_WATCH_INFO_COLOR_COREDEVICES_PR2_GOLD_14: WatchInfoColor = 42;
///!< CoreDevices PR2, Silver (14mm)
pub const WatchInfoColor_WATCH_INFO_COLOR_COREDEVICES_PR2_SILVER_14: WatchInfoColor = 43;
pub const WatchInfoColor_WATCH_INFO_COLOR__MAX: WatchInfoColor = 44;
///! The different watch colors.
pub type WatchInfoColor = ::core::ffi::c_uchar;
/**! Data structure containing the version of the firmware running on the watch.
! The version of the firmware has the form X.[X.[X]]. If a version number is not present it will be 0.
! For example: the version numbers of 2.4.1 are 2, 4, and 1. The version numbers of 2.4 are 2, 4, and 0.*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct WatchInfoVersion {
	///!< Major version number
	pub major: u8,
	///!< Minor version number
	pub minor: u8,
	///!< Patch version number
	pub patch: u8,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of WatchInfoVersion"][::core::mem::size_of::<WatchInfoVersion>() - 3usize];
	["Alignment of WatchInfoVersion"][::core::mem::align_of::<WatchInfoVersion>() - 1usize];
	["Offset of field: WatchInfoVersion::major"]
		[::core::mem::offset_of!(WatchInfoVersion, major) - 0usize];
	["Offset of field: WatchInfoVersion::minor"]
		[::core::mem::offset_of!(WatchInfoVersion, minor) - 1usize];
	["Offset of field: WatchInfoVersion::patch"]
		[::core::mem::offset_of!(WatchInfoVersion, patch) - 2usize];
};
/**! Provides the model of the watch.
! @return {@link WatchInfoModel} representing the model of the watch.*/
/** watch_info_get_model
  -> Added in 22
gneiss: Vtable Slot 347, Offset 1388*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.watch_info_get_model")]
pub unsafe extern "C" fn watch_info_get_model() -> WatchInfoModel {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1388", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Provides the version of the firmware running on the watch.
! @return {@link WatchInfoVersion} representing the version of the firmware running on the watch.*/
/** watch_info_get_firmware_version
  -> Added in 22
gneiss: Vtable Slot 346, Offset 1384*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.watch_info_get_firmware_version")]
pub unsafe extern "C" fn watch_info_get_firmware_version() -> WatchInfoVersion {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1384", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/** watch_info_get_color
  -> Added in 22
gneiss: Vtable Slot 345, Offset 1380*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.watch_info_get_color")]
pub unsafe extern "C" fn watch_info_get_color() -> WatchInfoColor {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1380", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Look-up the sine of the given angle from a pre-computed table.
! @param angle The angle for which to compute the cosine.
! The angle value is scaled linearly, such that a value of 0x10000 corresponds to 360 degrees or 2 PI radians.*/
/** sin_lookup
  -> Added in 0
gneiss: Vtable Slot 238, Offset 952*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.sin_lookup")]
pub unsafe extern "C" fn sin_lookup(angle: i32) -> i32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =952", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Look-up the cosine of the given angle from a pre-computed table.
! This is equivalent to calling `sin_lookup(angle + TRIG_MAX_ANGLE / 4)`.
! @param angle The angle for which to compute the cosine.
! The angle value is scaled linearly, such that a value of 0x10000 corresponds to 360 degrees or 2 PI radians.*/
/** cos_lookup
  -> Added in 0
gneiss: Vtable Slot 70, Offset 280*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.cos_lookup")]
pub unsafe extern "C" fn cos_lookup(angle: i32) -> i32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =280", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Look-up the arctangent of a given x, y pair
! The angle value is scaled linearly, such that a value of 0x10000 corresponds to 360 degrees or 2 PI radians.*/
/** atan2_lookup
  -> Added in 0
gneiss: Vtable Slot 50, Offset 200*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.atan2_lookup")]
pub unsafe extern "C" fn atan2_lookup(y: i16, x: i16) -> i32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =200", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///!< Today
pub const WeekDay_TODAY: WeekDay = 0;
///!< Sunday
pub const WeekDay_SUNDAY: WeekDay = 1;
///!< Monday
pub const WeekDay_MONDAY: WeekDay = 2;
///!< Tuesday
pub const WeekDay_TUESDAY: WeekDay = 3;
///!< Wednesday
pub const WeekDay_WEDNESDAY: WeekDay = 4;
///!< Thursday
pub const WeekDay_THURSDAY: WeekDay = 5;
///!< Friday
pub const WeekDay_FRIDAY: WeekDay = 6;
///!< Saturday
pub const WeekDay_SATURDAY: WeekDay = 7;
///! Weekday values
pub type WeekDay = ::core::ffi::c_uchar;
/**! Copies a time string into the buffer, formatted according to the user's time display preferences (such as 12h/24h
! time).
! Example results: "7:30" or "15:00".
! @note AM/PM are also outputted with the time if the user's preference is 12h time.
! @param[out] buffer A pointer to the buffer to copy the time string into
! @param size The maximum size of buffer*/
/** clock_copy_time_string
  -> Added in 0
gneiss: Vtable Slot 68, Offset 272*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.clock_copy_time_string")]
pub unsafe extern "C" fn clock_copy_time_string(buffer: *mut ::core::ffi::c_char, size: u8) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =272", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the user's 12/24h clock style preference.
! @return `true` if the user prefers 24h-style time display or `false` if the
! user prefers 12h-style time display.*/
/** clock_is_24h_style
  -> Added in 0
gneiss: Vtable Slot 69, Offset 276*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.clock_is_24h_style")]
pub unsafe extern "C" fn clock_is_24h_style() -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =276", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Converts a (day, hour, minute) specification to a UTC timestamp occurring in the future
! Always returns a timestamp for the next occurring instance,
! example: specifying TODAY@14:30 when it is 14:40 will return a timestamp for 7 days from
! now at 14:30
! @param day WeekDay day of week including support for specifying TODAY
! @param hour hour specified in 24-hour format [0-23]
! @param minute minute [0-59]*/
/** clock_to_timestamp
  -> Added in 24
gneiss: Vtable Slot 351, Offset 1404*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.clock_to_timestamp")]
pub unsafe extern "C" fn clock_to_timestamp(
	day: WeekDay,
	hour: ::core::ffi::c_int,
	minute: ::core::ffi::c_int,
) -> time_t {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1404", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Checks if timezone is currently set, otherwise gmtime == localtime.
! @return `true` if timezone has been set, false otherwise*/
/** clock_is_timezone_set
  -> Added in 25
gneiss: Vtable Slot 359, Offset 1436*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.clock_is_timezone_set")]
pub unsafe extern "C" fn clock_is_timezone_set() -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1436", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! If timezone is set, copies the current timezone long name (e.g. America/Chicago)
! to user-provided buffer.
! @param timezone A pointer to the buffer to copy the timezone long name into
! @param buffer_size Size of the allocated buffer to copy the timezone long name into
! @note timezone buffer should be at least TIMEZONE_NAME_LENGTH bytes*/
/** clock_get_timezone
  -> Added in 31
gneiss: Vtable Slot 378, Offset 1512*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.clock_get_timezone")]
pub unsafe extern "C" fn clock_get_timezone(
	timezone: *mut ::core::ffi::c_char,
	buffer_size: usize,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1512", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
pub const PlatformType_PlatformTypeAplite: PlatformType = 0;
pub const PlatformType_PlatformTypeBasalt: PlatformType = 1;
pub const PlatformType_PlatformTypeChalk: PlatformType = 2;
pub const PlatformType_PlatformTypeDiorite: PlatformType = 3;
pub const PlatformType_PlatformTypeEmery: PlatformType = 4;
pub const PlatformType_PlatformTypeFlint: PlatformType = 5;
pub const PlatformType_PlatformTypeGabbro: PlatformType = 6;
/**! @addtogroup Platform
! @{*/
pub type PlatformType = ::core::ffi::c_uchar;
/**! @addtogroup ConnectionService
! \brief Determine what the Pebble watch is connected to
!
! The ConnectionService allows your app to learn about the apps the Pebble
! watch is connected to. You can ask the system for this information at a
! given time or you can register to receive events every time connection or
! disconnection events occur.
!
! It allows you to determine whether the watch is connected to the Pebble
! mobile app by subscribing to the pebble_app_connection_handler or by calling
! the connection_service_peek_pebble_app_connection function.  Note that when
! the Pebble app is connected, you can assume PebbleKit JS apps will also be
! running correctly.
!
! The service also allows you to determine if the Pebble watch can establish
! a connection to a PebbleKit companion app by subscribing to the
! pebblekit_connection_handler or by calling the
! connection_service_peek_pebblekit_connection function.  Today, due to
! architectural differences between iOS and Android, this will return true
! for Android anytime a connection with the Pebble mobile app is established
! (since PebbleKit messages are routed through the Android app). For iOS,
! this will return true when any PebbleKit companion app has established a
! connection with the Pebble watch (since companion app messages are routed
! directly to the watch)
!
! @{*/
pub type ConnectionHandler = ::core::option::Option<unsafe extern "C" fn(connected: bool)>;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ConnectionHandlers {
	/**! callback to be executed when the connection state between the watch and
	! the phone app has changed. Note, if the phone App is connected, PebbleKit JS apps
	! will also be working correctly*/
	pub pebble_app_connection_handler: ConnectionHandler,
	///! ID for callback to be executed on PebbleKit connection event
	pub pebblekit_connection_handler:  ConnectionHandler,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of ConnectionHandlers"][::core::mem::size_of::<ConnectionHandlers>() - 8usize];
	["Alignment of ConnectionHandlers"][::core::mem::align_of::<ConnectionHandlers>() - 4usize];
	["Offset of field: ConnectionHandlers::pebble_app_connection_handler"]
		[::core::mem::offset_of!(ConnectionHandlers, pebble_app_connection_handler) - 0usize];
	["Offset of field: ConnectionHandlers::pebblekit_connection_handler"]
		[::core::mem::offset_of!(ConnectionHandlers, pebblekit_connection_handler) - 4usize];
};
/**! Query the bluetooth connection service for the current Pebble app connection status
! @return true if the Pebble app is connected, false otherwise*/
/** connection_service_peek_pebble_app_connection
  -> Added in 66
gneiss: Vtable Slot 566, Offset 2264*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.connection_service_peek_pebble_app_connection")]
pub unsafe extern "C" fn connection_service_peek_pebble_app_connection() -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2264", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Query the bluetooth connection service for the current PebbleKit connection status
! @return true if a PebbleKit companion app is connected, false otherwise*/
/** connection_service_peek_pebblekit_connection
  -> Added in 66
gneiss: Vtable Slot 567, Offset 2268*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.connection_service_peek_pebblekit_connection")]
pub unsafe extern "C" fn connection_service_peek_pebblekit_connection() -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2268", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Subscribe to the connection event service. Once subscribed, the appropriate
! handler gets called based on the type of connection event and user provided
! handlers
! @param conn_handlers A struct populated with the handlers to
! be called when the specified connection event occurs. If a given handler is
! NULL, no function will be called.*/
/** connection_service_subscribe
  -> Added in 66
gneiss: Vtable Slot 568, Offset 2272*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.connection_service_subscribe")]
pub unsafe extern "C" fn connection_service_subscribe(conn_handlers: ConnectionHandlers) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2272", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Unsubscribe from the bluetooth event service. Once unsubscribed, the previously registered
! handler will no longer be called.*/
/** connection_service_unsubscribe
  -> Added in 66
gneiss: Vtable Slot 569, Offset 2276*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.connection_service_unsubscribe")]
pub unsafe extern "C" fn connection_service_unsubscribe() {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2276", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @deprecated Backwards compatibility typedef for ConnectionHandler. New code
! should use ConnectionHandler directly.  This will be removed in a future
! version of the Pebble SDK.*/
pub type BluetoothConnectionHandler = ConnectionHandler;
/**! @deprecated Backward compatibility function for
! connection_service_peek_pebble_app_connection.  New code should use
! connection_service_peek_pebble_app_connection directly. This will be
! removed in a future version of the Pebble SDK*/
/** bluetooth_connection_service_peek
  -> Added in 0
gneiss: Vtable Slot 63, Offset 252*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.bluetooth_connection_service_peek")]
pub unsafe extern "C" fn bluetooth_connection_service_peek() -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =252", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @deprecated Backward compatibility function for
! connection_service_subscribe.  New code should use
! connection_service_subscribe directly. This will be removed in a future
! version of the Pebble SDK*/
/** bluetooth_connection_service_subscribe
  -> Added in 0
gneiss: Vtable Slot 64, Offset 256*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.bluetooth_connection_service_subscribe")]
pub unsafe extern "C" fn bluetooth_connection_service_subscribe(handler: ConnectionHandler) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =256", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @deprecated Backward compatibility function for
! connection_service_unsubscribe.  New code should use
! connection_service_unsubscribe directly. This will be removed in a future
! version of the Pebble SDK*/
/** bluetooth_connection_service_unsubscribe
  -> Added in 0
gneiss: Vtable Slot 65, Offset 260*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.bluetooth_connection_service_unsubscribe")]
pub unsafe extern "C" fn bluetooth_connection_service_unsubscribe() {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =260", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Callback type for focus events
! @param in_focus True if the app is gaining focus, false otherwise.*/
pub type AppFocusHandler = ::core::option::Option<unsafe extern "C" fn(in_focus: bool)>;
/**! There are two different focus events which take place when transitioning to and from an app
! being in focus. Below is an example of when these events will occur:
! 1) The app is launched. Once the system animation to the app has completed and the app is
! completely in focus, the did_focus handler is called with in_focus set to true.
! 2) A notification comes in and the animation to show the notification starts. The will_focus
! handler is called with in_focus set to false.
! 3) The animation completes and the notification is in focus, with the app being completely
! covered. The did_focus hander is called with in_focus set to false.
! 4) The notification is dismissed and the animation to return to the app starts. The will_focus
! handler is called with in_focus set to true.
! 5) The animation completes and the app is in focus. The did_focus handler is called with
! in_focus set to true.*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct AppFocusHandlers {
	/**! Handler which will be called right before an app will lose or gain focus.
	! @note This will be called with in_focus set to true when a window which is covering the app is
	! about to close and return focus to the app.
	! @note This will be called with in_focus set to false when a window which will cover the app is
	! about to open, causing the app to lose focus.*/
	pub will_focus: AppFocusHandler,
	/**! Handler which will be called when an animation finished which has put the app into focus or
	! taken the app out of focus.
	! @note This will be called with in_focus set to true when a window which was covering the app
	! has closed and the app has gained focus.
	! @note This will be called with in_focus set to false when a window has opened which is now
	! covering the app, causing the app to lose focus.*/
	pub did_focus:  AppFocusHandler,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of AppFocusHandlers"][::core::mem::size_of::<AppFocusHandlers>() - 8usize];
	["Alignment of AppFocusHandlers"][::core::mem::align_of::<AppFocusHandlers>() - 4usize];
	["Offset of field: AppFocusHandlers::will_focus"]
		[::core::mem::offset_of!(AppFocusHandlers, will_focus) - 0usize];
	["Offset of field: AppFocusHandlers::did_focus"]
		[::core::mem::offset_of!(AppFocusHandlers, did_focus) - 4usize];
};
/**! Subscribe to the focus event service. Once subscribed, the handlers get called every time the
! app gains or loses focus.
! @param handlers Handlers which will be called on will-focus and did-focus events.
! @see AppFocusHandlers*/
/** app_focus_service_subscribe_handlers
  -> Added in 62
gneiss: Vtable Slot 535, Offset 2140*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_focus_service_subscribe_handlers")]
pub unsafe extern "C" fn app_focus_service_subscribe_handlers(handlers: AppFocusHandlers) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2140", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Subscribe to the focus event service. Once subscribed, the handler
! gets called every time the app focus changes.
! @note Calling this function is equivalent to
! \code{.c}
! app_focus_service_subscribe_handlers((AppFocusHandlers){
!   .will_focus = handler,
! });
! \endcode
! @note Out focus events are triggered when a modal window is about to open and cover the app.
! @note In focus events are triggered when a modal window which is covering the app is about to
! close.
! @param handler A callback to be called on will-focus events.*/
/** app_focus_service_subscribe
  -> Added in 1
gneiss: Vtable Slot 289, Offset 1156*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_focus_service_subscribe")]
pub unsafe extern "C" fn app_focus_service_subscribe(handler: AppFocusHandler) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1156", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Unsubscribe from the focus event service. Once unsubscribed, the previously
! registered handlers will no longer be called.*/
/** app_focus_service_unsubscribe
  -> Added in 1
gneiss: Vtable Slot 290, Offset 1160*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_focus_service_unsubscribe")]
pub unsafe extern "C" fn app_focus_service_unsubscribe() {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1160", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Structure for retrieval of the battery charge state
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct BatteryChargeState {
	///! A percentage (0-100) of how full the battery is
	pub charge_percent: u8,
	///! True if the battery is currently being charged. False if not.
	pub is_charging:    bool,
	///! True if the charger cable is connected. False if not.
	pub is_plugged:     bool,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of BatteryChargeState"][::core::mem::size_of::<BatteryChargeState>() - 3usize];
	["Alignment of BatteryChargeState"][::core::mem::align_of::<BatteryChargeState>() - 1usize];
	["Offset of field: BatteryChargeState::charge_percent"]
		[::core::mem::offset_of!(BatteryChargeState, charge_percent) - 0usize];
	["Offset of field: BatteryChargeState::is_charging"]
		[::core::mem::offset_of!(BatteryChargeState, is_charging) - 1usize];
	["Offset of field: BatteryChargeState::is_plugged"]
		[::core::mem::offset_of!(BatteryChargeState, is_plugged) - 2usize];
};
/**! Callback type for battery state change events
! @param charge the state of the battery \ref BatteryChargeState*/
pub type BatteryStateHandler =
	::core::option::Option<unsafe extern "C" fn(charge: BatteryChargeState)>;
/**! Subscribe to the battery state event service. Once subscribed, the handler gets called
! on every battery state change
! @param handler A callback to be executed on battery state change event*/
/** battery_state_service_subscribe
  -> Added in 0
gneiss: Vtable Slot 54, Offset 216*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.battery_state_service_subscribe")]
pub unsafe extern "C" fn battery_state_service_subscribe(handler: BatteryStateHandler) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =216", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Unsubscribe from the battery state event service. Once unsubscribed, the previously registered
! handler will no longer be called.*/
/** battery_state_service_unsubscribe
  -> Added in 0
gneiss: Vtable Slot 55, Offset 220*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.battery_state_service_unsubscribe")]
pub unsafe extern "C" fn battery_state_service_unsubscribe() {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =220", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Peek at the last known battery state.
! @return a \ref BatteryChargeState containing the last known data*/
/** battery_state_service_peek
  -> Added in 0
gneiss: Vtable Slot 53, Offset 212*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.battery_state_service_peek")]
pub unsafe extern "C" fn battery_state_service_peek() -> BatteryChargeState {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =212", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Callback type for backlight on/off events.
! @param on true when the backlight has just turned on, false when it has
!   just turned fully off.*/
pub type BacklightHandler = ::core::option::Option<unsafe extern "C" fn(on: bool)>;
/**! A single accelerometer sample for all three axes including timestamp and
! vibration rumble status.*/
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct AccelData {
	///! acceleration along the x axis
	pub x:           i16,
	///! acceleration along the y axis
	pub y:           i16,
	///! acceleration along the z axis
	pub z:           i16,
	///! true if the watch vibrated when this sample was collected
	pub did_vibrate: bool,
	///! timestamp, in milliseconds
	pub timestamp:   u64,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of AccelData"][::core::mem::size_of::<AccelData>() - 15usize];
	["Alignment of AccelData"][::core::mem::align_of::<AccelData>() - 1usize];
	["Offset of field: AccelData::x"][::core::mem::offset_of!(AccelData, x) - 0usize];
	["Offset of field: AccelData::y"][::core::mem::offset_of!(AccelData, y) - 2usize];
	["Offset of field: AccelData::z"][::core::mem::offset_of!(AccelData, z) - 4usize];
	["Offset of field: AccelData::did_vibrate"]
		[::core::mem::offset_of!(AccelData, did_vibrate) - 6usize];
	["Offset of field: AccelData::timestamp"]
		[::core::mem::offset_of!(AccelData, timestamp) - 7usize];
};
///! A single accelerometer sample for all three axes
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct AccelRawData {
	///! acceleration along the x axis
	pub x: i16,
	///! acceleration along the y axis
	pub y: i16,
	///! acceleration along the z axis
	pub z: i16,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of AccelRawData"][::core::mem::size_of::<AccelRawData>() - 6usize];
	["Alignment of AccelRawData"][::core::mem::align_of::<AccelRawData>() - 1usize];
	["Offset of field: AccelRawData::x"][::core::mem::offset_of!(AccelRawData, x) - 0usize];
	["Offset of field: AccelRawData::y"][::core::mem::offset_of!(AccelRawData, y) - 2usize];
	["Offset of field: AccelRawData::z"][::core::mem::offset_of!(AccelRawData, z) - 4usize];
};
/**! Accelerometer's X axis. The positive direction along the X axis goes
! toward the right of the watch.*/
pub const AccelAxisType_ACCEL_AXIS_X: AccelAxisType = 0;
/**! Accelerometer's Y axis. The positive direction along the Y axis goes
! toward the top of the watch.*/
pub const AccelAxisType_ACCEL_AXIS_Y: AccelAxisType = 1;
/**! Accelerometer's Z axis. The positive direction along the Z axis goes
! vertically out of the watchface.*/
pub const AccelAxisType_ACCEL_AXIS_Z: AccelAxisType = 2;
///! Enumerated values defining the three accelerometer axes.
pub type AccelAxisType = ::core::ffi::c_uchar;
/**! Callback type for accelerometer data events
! @param data Pointer to the collected accelerometer samples.
! @param num_samples the number of samples stored in data.*/
pub type AccelDataHandler =
	::core::option::Option<unsafe extern "C" fn(data: *mut AccelData, num_samples: u32)>;
/**! Callback type for accelerometer raw data events
! @param data Pointer to the collected accelerometer samples.
! @param num_samples the number of samples stored in data.
! @param timestamp the timestamp, in ms, of the first sample.*/
pub type AccelRawDataHandler = ::core::option::Option<
	unsafe extern "C" fn(data: *mut AccelRawData, num_samples: u32, timestamp: u64),
>;
/**! Callback type for accelerometer tap events
! @param axis the axis on which a tap was registered (x, y, or z)
! @param direction the direction (-1 or +1) of the tap*/
pub type AccelTapHandler =
	::core::option::Option<unsafe extern "C" fn(axis: AccelAxisType, direction: i32)>;
///! 10 HZ sampling rate
pub const AccelSamplingRate_ACCEL_SAMPLING_10HZ: AccelSamplingRate = 10;
///! 25 HZ sampling rate [Default]
pub const AccelSamplingRate_ACCEL_SAMPLING_25HZ: AccelSamplingRate = 25;
///! 50 HZ sampling rate
pub const AccelSamplingRate_ACCEL_SAMPLING_50HZ: AccelSamplingRate = 50;
///! 100 HZ sampling rate
pub const AccelSamplingRate_ACCEL_SAMPLING_100HZ: AccelSamplingRate = 100;
///! Valid accelerometer sampling rates, in Hz
pub type AccelSamplingRate = ::core::ffi::c_uchar;
/**! Peek at the last recorded reading.
! @param[out] data a pointer to a pre-allocated AccelData item
! @note Cannot be used when subscribed to accelerometer data events.
! @return -1 if the accel is not running
! @return -2 if subscribed to accelerometer events.*/
/** accel_service_peek
  -> Added in 0
gneiss: Vtable Slot 2, Offset 8*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.accel_service_peek")]
pub unsafe extern "C" fn accel_service_peek(data: *mut AccelData) -> ::core::ffi::c_int {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =8", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Change the accelerometer sampling rate.
! @param rate The sampling rate in Hz (10Hz, 25Hz, 50Hz, and 100Hz possible)*/
/** accel_service_set_sampling_rate
  -> Added in 0
gneiss: Vtable Slot 4, Offset 16*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.accel_service_set_sampling_rate")]
pub unsafe extern "C" fn accel_service_set_sampling_rate(
	rate: AccelSamplingRate,
) -> ::core::ffi::c_int {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =16", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Change the number of samples buffered between each accelerometer data event
! @param num_samples the number of samples to buffer, between 0 and 25.*/
/** accel_service_set_samples_per_update
  -> Added in 0
gneiss: Vtable Slot 3, Offset 12*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.accel_service_set_samples_per_update")]
pub unsafe extern "C" fn accel_service_set_samples_per_update(
	num_samples: u32,
) -> ::core::ffi::c_int {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =12", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Subscribe to the accelerometer data event service. Once subscribed, the handler
! gets called every time there are new accelerometer samples available.
! @note Cannot use \ref accel_service_peek() when subscribed to accelerometer data events.
! @param handler A callback to be executed on accelerometer data events
! @param samples_per_update the number of samples to buffer, between 0 and 25.*/
/** accel_data_service_subscribe
  -> Added in 9
gneiss: Vtable Slot 317, Offset 1268*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.accel_data_service_subscribe")]
pub unsafe extern "C" fn accel_data_service_subscribe(
	samples_per_update: u32,
	handler: AccelDataHandler,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1268", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Unsubscribe from the accelerometer data event service. Once unsubscribed,
! the previously registered handler will no longer be called.*/
/** accel_data_service_unsubscribe
  -> Added in 0
gneiss: Vtable Slot 1, Offset 4*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.accel_data_service_unsubscribe")]
pub unsafe extern "C" fn accel_data_service_unsubscribe() {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =4", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Subscribe to the accelerometer tap event service. Once subscribed, the handler
! gets called on every tap event emitted by the accelerometer.
! @param handler A callback to be executed on tap event*/
/** accel_tap_service_subscribe
  -> Added in 0
gneiss: Vtable Slot 5, Offset 20*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.accel_tap_service_subscribe")]
pub unsafe extern "C" fn accel_tap_service_subscribe(handler: AccelTapHandler) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =20", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Unsubscribe from the accelerometer tap event service. Once unsubscribed,
! the previously registered handler will no longer be called.*/
/** accel_tap_service_unsubscribe
  -> Added in 0
gneiss: Vtable Slot 6, Offset 24*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.accel_tap_service_unsubscribe")]
pub unsafe extern "C" fn accel_tap_service_unsubscribe() {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =24", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Subscribe to the accelerometer raw data event service. Once subscribed, the handler
! gets called every time there are new accelerometer samples available.
! @note Cannot use \ref accel_service_peek() when subscribed to accelerometer data events.
! @param handler A callback to be executed on accelerometer data events
! @param samples_per_update the number of samples to buffer, between 0 and 25.*/
/** accel_raw_data_service_subscribe
  -> Added in 15
gneiss: Vtable Slot 326, Offset 1304*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.accel_raw_data_service_subscribe")]
pub unsafe extern "C" fn accel_raw_data_service_subscribe(
	samples_per_update: u32,
	handler: AccelRawDataHandler,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1304", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! The Compass Service is unavailable.
pub const CompassStatus_CompassStatusUnavailable: CompassStatus = -1;
/**! Compass is calibrating: data is invalid and should not be used
! Data will become valid once calibration is complete*/
pub const CompassStatus_CompassStatusDataInvalid: CompassStatus = 0;
///! Compass is calibrating: the data is valid but the calibration is still being refined
pub const CompassStatus_CompassStatusCalibrating: CompassStatus = 1;
///! Compass data is valid and the calibration has completed
pub const CompassStatus_CompassStatusCalibrated: CompassStatus = 2;
///! Enum describing the current state of the Compass Service
pub type CompassStatus = ::core::ffi::c_schar;
/**! Represents an angle relative to get to a reference direction, e.g. (magnetic) north.
! The angle value is scaled linearly, such that a value of TRIG_MAX_ANGLE
! corresponds to 360 degrees or 2 PI radians.
! Thus, if heading towards north, north is 0, west is TRIG_MAX_ANGLE/4,
! south is TRIG_MAX_ANGLE/2, and so on.*/
pub type CompassHeading = i32;
///! Structure containing a single heading towards magnetic and true north.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CompassHeadingData {
	/**! Measured angle that increases counter-clockwise from magnetic north
	! (use `int clockwise_heading = TRIG_MAX_ANGLE - heading_data.magnetic_heading;`
	! for example to find your heading clockwise from magnetic north).*/
	pub magnetic_heading:     CompassHeading,
	///! Currently same value as magnetic_heading (reserved for future implementation).
	pub true_heading:         CompassHeading,
	///! Indicates the current state of the Compass Service calibration.
	pub compass_status:       CompassStatus,
	///! Currently always false (reserved for future implementation).
	pub is_declination_valid: bool,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of CompassHeadingData"][::core::mem::size_of::<CompassHeadingData>() - 12usize];
	["Alignment of CompassHeadingData"][::core::mem::align_of::<CompassHeadingData>() - 4usize];
	["Offset of field: CompassHeadingData::magnetic_heading"]
		[::core::mem::offset_of!(CompassHeadingData, magnetic_heading) - 0usize];
	["Offset of field: CompassHeadingData::true_heading"]
		[::core::mem::offset_of!(CompassHeadingData, true_heading) - 4usize];
	["Offset of field: CompassHeadingData::compass_status"]
		[::core::mem::offset_of!(CompassHeadingData, compass_status) - 8usize];
	["Offset of field: CompassHeadingData::is_declination_valid"]
		[::core::mem::offset_of!(CompassHeadingData, is_declination_valid) - 9usize];
};
/**! Callback type for compass heading events
! @param heading copy of last recorded heading*/
pub type CompassHeadingHandler =
	::core::option::Option<unsafe extern "C" fn(heading: CompassHeadingData)>;
/**! Set the minimum angular change required to generate new compass heading events.
! The angular distance is measured relative to the last delivered heading event.
! Use 0 to be notified of all movements.
! Negative values and values > TRIG_MAX_ANGLE / 2 are not valid.
! The default value of this property is TRIG_MAX_ANGLE / 360.
! @return 0, success.
! @return Non-Zero, if filter is invalid.
! @see compass_service_subscribe*/
/** compass_service_set_heading_filter
  -> Added in 18
gneiss: Vtable Slot 338, Offset 1352*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.compass_service_set_heading_filter")]
pub unsafe extern "C" fn compass_service_set_heading_filter(
	filter: CompassHeading,
) -> ::core::ffi::c_int {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1352", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Subscribe to the compass heading event service. Once subscribed, the handler
! gets called every time the angular distance relative to the previous value
! exceeds the configured filter.
! @param handler A callback to be executed on heading events
! @see compass_service_set_heading_filter
! @see compass_service_unsubscribe*/
/** compass_service_subscribe
  -> Added in 18
gneiss: Vtable Slot 339, Offset 1356*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.compass_service_subscribe")]
pub unsafe extern "C" fn compass_service_subscribe(handler: CompassHeadingHandler) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1356", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Unsubscribe from the compass heading event service. Once unsubscribed,
! the previously registered handler will no longer be called.
! @see compass_service_subscribe*/
/** compass_service_unsubscribe
  -> Added in 18
gneiss: Vtable Slot 340, Offset 1360*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.compass_service_unsubscribe")]
pub unsafe extern "C" fn compass_service_unsubscribe() {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1360", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Peek at the last recorded reading.
! @param[out] data a pointer to a pre-allocated CompassHeadingData
! @return Always returns 0 to indicate success.*/
/** compass_service_peek
  -> Added in 18
gneiss: Vtable Slot 337, Offset 1348*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.compass_service_peek")]
pub unsafe extern "C" fn compass_service_peek(data: *mut CompassHeadingData) -> ::core::ffi::c_int {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1348", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Flag to represent the "seconds" time unit
pub const TimeUnits_SECOND_UNIT: TimeUnits = 1;
///! Flag to represent the "minutes" time unit
pub const TimeUnits_MINUTE_UNIT: TimeUnits = 2;
///! Flag to represent the "hours" time unit
pub const TimeUnits_HOUR_UNIT: TimeUnits = 4;
///! Flag to represent the "days" time unit
pub const TimeUnits_DAY_UNIT: TimeUnits = 8;
///! Flag to represent the "months" time unit
pub const TimeUnits_MONTH_UNIT: TimeUnits = 16;
///! Flag to represent the "years" time unit
pub const TimeUnits_YEAR_UNIT: TimeUnits = 32;
/**! Time unit flags that can be used to create a bitmask for use in \ref tick_timer_service_subscribe().
! This will also be passed to \ref TickHandler.*/
pub type TimeUnits = ::core::ffi::c_uchar;
/**! Callback type for tick timer events
! @param tick_time the time at which the tick event was triggered
! @param units_changed which unit change triggered this tick event*/
pub type TickHandler =
	::core::option::Option<unsafe extern "C" fn(tick_time: *mut tm, units_changed: TimeUnits)>;
/**! Subscribe to the tick timer event service. Once subscribed, the handler gets called
! on every requested unit change.
! Calling this function multiple times will override the units and handler (i.e., only
! the last tick_units and handler passed will be used).
! @param handler The callback to be executed on tick events
! @param tick_units a bitmask of all the units that have changed*/
/** tick_timer_service_subscribe
  -> Added in 0
gneiss: Vtable Slot 262, Offset 1048*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.tick_timer_service_subscribe")]
pub unsafe extern "C" fn tick_timer_service_subscribe(tick_units: TimeUnits, handler: TickHandler) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1048", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Unsubscribe from the tick timer event service. Once unsubscribed, the previously registered
! handler will no longer be called.*/
/** tick_timer_service_unsubscribe
  -> Added in 0
gneiss: Vtable Slot 263, Offset 1052*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.tick_timer_service_unsubscribe")]
pub unsafe extern "C" fn tick_timer_service_unsubscribe() {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1052", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
pub const TouchEventType_TouchEvent_Touchdown: TouchEventType = 0;
pub const TouchEventType_TouchEvent_Liftoff: TouchEventType = 1;
pub const TouchEventType_TouchEvent_PositionUpdate: TouchEventType = 2;
///! Touch event type
pub type TouchEventType = ::core::ffi::c_uchar;
///! Touch event data, carried directly in PebbleTouchEvent
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TouchEvent {
	pub _bitfield_1:      __BindgenBitfieldUnit<[u8; 1usize]>,
	/**! true when the touch must not drive navigation: the interaction session
	! was inactive at Touchdown (unarmed contact on the idle watchface).
	! Latched on Touchdown and carried across the whole gesture.*/
	pub non_navigational: bool,
	pub x:                i16,
	pub y:                i16,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of TouchEvent"][::core::mem::size_of::<TouchEvent>() - 6usize];
	["Alignment of TouchEvent"][::core::mem::align_of::<TouchEvent>() - 2usize];
	["Offset of field: TouchEvent::non_navigational"]
		[::core::mem::offset_of!(TouchEvent, non_navigational) - 1usize];
	["Offset of field: TouchEvent::x"][::core::mem::offset_of!(TouchEvent, x) - 2usize];
	["Offset of field: TouchEvent::y"][::core::mem::offset_of!(TouchEvent, y) - 4usize];
};
impl TouchEvent {
	#[inline]
	#[allow(unnecessary_transmutes)]
	pub fn type_(&self) -> TouchEventType {
		unsafe { ::core::mem::transmute(self._bitfield_1.get_const::<0usize, 8u8>() as u8) }
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub fn set_type(&mut self, val: TouchEventType) {
		unsafe {
			let val: u8 = ::core::mem::transmute(val);
			self._bitfield_1.set_const::<0usize, 8u8>(val as u64)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub unsafe fn type__raw(this: *const Self) -> TouchEventType {
		unsafe {
			::core::mem::transmute(
				<__BindgenBitfieldUnit<[u8; 1usize]>>::raw_get_const::<0usize, 8u8>(
					::core::ptr::addr_of!((*this)._bitfield_1),
				) as u8,
			)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub unsafe fn set_type_raw(this: *mut Self, val: TouchEventType) {
		unsafe {
			let val: u8 = ::core::mem::transmute(val);
			<__BindgenBitfieldUnit<[u8; 1usize]>>::raw_set_const::<0usize, 8u8>(
				::core::ptr::addr_of_mut!((*this)._bitfield_1),
				val as u64,
			)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub fn new_bitfield_1(type_: TouchEventType) -> __BindgenBitfieldUnit<[u8; 1usize]> {
		let mut __bindgen_bitfield_unit: __BindgenBitfieldUnit<[u8; 1usize]> = Default::default();
		__bindgen_bitfield_unit.set_const::<0usize, 8u8>({
			let type_: u8 = unsafe { ::core::mem::transmute(type_) };
			type_ as u64
		});
		__bindgen_bitfield_unit
	}
}
/**! Callback for touch events.
! @param event The touch event data
! @param context User-provided context*/
pub type TouchServiceHandler = ::core::option::Option<
	unsafe extern "C" fn(event: *const TouchEvent, context: *mut ::core::ffi::c_void),
>;
///! The number of steps counted.
pub const HealthMetric_HealthMetricStepCount: HealthMetric = 0;
///! The number of seconds spent active (i.e. not resting).
pub const HealthMetric_HealthMetricActiveSeconds: HealthMetric = 1;
///! The distance walked, in meters.
pub const HealthMetric_HealthMetricWalkedDistanceMeters: HealthMetric = 2;
///! The number of seconds spent sleeping.
pub const HealthMetric_HealthMetricSleepSeconds: HealthMetric = 3;
///! The number of sleep seconds in the 'restful' or deep sleep state.
pub const HealthMetric_HealthMetricSleepRestfulSeconds: HealthMetric = 4;
///! The number of kcal (Calories) burned while resting due to resting metabolism.
pub const HealthMetric_HealthMetricRestingKCalories: HealthMetric = 5;
///! The number of kcal (Calories) burned while active.
pub const HealthMetric_HealthMetricActiveKCalories: HealthMetric = 6;
///! The heart rate, in beats per minute. This is a filtered value that is at most 15 minutes old.
pub const HealthMetric_HealthMetricHeartRateBPM: HealthMetric = 7;
///! The raw heart rate value of the most recent sample, in beats per minute.
pub const HealthMetric_HealthMetricHeartRateRawBPM: HealthMetric = 8;
/**! Health metric values used to retrieve health data.
! For example, using \ref health_service_sum().*/
pub type HealthMetric = ::core::ffi::c_uchar;
///! Type used to represent HealthMetric values
pub type HealthValue = i32;
/**! Type used as a handle to a registered metric alert (returned by
! \ref health_service_register_metric_alert)*/
#[repr(C)]
#[derive(Debug)]
pub struct HealthMetricAlert {
	_unused: [u8; 0],
}
/**! Return the sum of a \ref HealthMetric's values over a time range.
! The `time_start` and `time_end` parameters define the range of time you want the sum for.
! @note The value returned will be based on daily totals, weighted for the length of the
! specified time range. This may change in the future.
! @param metric The metric to query for data.
! @param time_start UTC time of the earliest data item to incorporate into the sum.
! @param time_end UTC time of the most recent data item to incorporate into the sum.
! @return The sum of that metric over the given time range, if available.*/
/** health_service_sum
  -> Added in 77
gneiss: Vtable Slot 606, Offset 2424*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.health_service_sum")]
pub unsafe extern "C" fn health_service_sum(
	metric: HealthMetric,
	time_start: time_t,
	time_end: time_t,
) -> HealthValue {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2424", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Convenience wrapper for \ref health_service_sum() that returns the sum for today.
! @param metric The metric to query.
! @return The sum of that metric's data for today, if available.*/
/** health_service_sum_today
  -> Added in 77
gneiss: Vtable Slot 607, Offset 2428*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.health_service_sum_today")]
pub unsafe extern "C" fn health_service_sum_today(metric: HealthMetric) -> HealthValue {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2428", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Convenience function for peeking at the current value of a metric. This is useful for metrics
! like \ref HealthMetricHeartRateBPM that represent instantaneous values. It is NOT applicable for
! metrics like \ref HealthMetricStepCount that must be accumulated over time (it will return 0 if
! passed that type of metric). This call is equivalent to calling
! `health_service_aggregate_averaged(metric, time(NULL), time(NULL), HealthAggregationAvg,
! HealthServiceTimeScopeOnce)`
! @param metric The metric to query.
! @return The current value of that metric, if available.*/
/** health_service_peek_current_value
  -> Added in 83
gneiss: Vtable Slot 620, Offset 2480*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.health_service_peek_current_value")]
pub unsafe extern "C" fn health_service_peek_current_value(metric: HealthMetric) -> HealthValue {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2480", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! No average computed. The result is the same as calling \ref health_service_sum().
pub const HealthServiceTimeScope_HealthServiceTimeScopeOnce: HealthServiceTimeScope = 0;
/**! Compute average using the same day from each week. For example, every Monday if the passed
! in time range falls on a Monday.*/
pub const HealthServiceTimeScope_HealthServiceTimeScopeWeekly: HealthServiceTimeScope = 1;
/**! Compute average using either weekdays (Monday to Friday) or weekends (Saturday and Sunday),
! depending on which day the passed in time range falls.*/
pub const HealthServiceTimeScope_HealthServiceTimeScopeDailyWeekdayOrWeekend:
	HealthServiceTimeScope = 2;
///! Compute average across all days of the week.
pub const HealthServiceTimeScope_HealthServiceTimeScopeDaily: HealthServiceTimeScope = 3;
///! Used by \ref health_service_sum_averaged() to specify how the average is computed.
pub type HealthServiceTimeScope = ::core::ffi::c_uchar;
/**! Return the value of a metric's sum over a given time range between `time_start`
! and `time_end`. Using this call you can specify the time range that you are interested in
! getting the average for, as well as a `scope` specifier on how to compute an average of the sum.
! For example, if you want to get the average number of steps taken from 12 AM (midnight) to 9 AM
! across all days you would specify:
! \code{.c}
! time_t time_start = time_start_of_today();
! time_t time_end = time_start + (9 * SECONDS_PER_HOUR);
! HealthValue value = health_service_sum_averaged(HealthMetricStepCount, time_start,
!    time_end, HealthServiceTimeScopeDaily);
! \endcode
! If you want the average number of steps taken on a weekday (Monday to Friday) and today is a
! Monday (in the local timezone) you would specify:
! \code{.c}
! time_start = time_start_of_today();
! time_end = time_start + SECONDS_PER_DAY;
! HealthValue value = health_service_sum_averaged(HealthMetricStepCount, time_start,
!    time_end, HealthServiceTimeScopeDailyWeekdayOrWeekend);
! \endcode
!
! Note that this call is the same as calling `health_service_aggregate_averaged(metric,
! time_start, time_end, HealthAggregationSum, scope)`
!
! @param metric Which \ref HealthMetric to query.
! @param time_start UTC time of the start of the query interval.
! @param time_end UTC time of the end of the query interval.
! @param scope \ref HealthServiceTimeScope value describing how the average should be computed.
! @return The average of the sum of the given metric over the given time range, if available.*/
/** health_service_sum_averaged
  -> Added in 79
gneiss: Vtable Slot 610, Offset 2440*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.health_service_sum_averaged")]
pub unsafe extern "C" fn health_service_sum_averaged(
	metric: HealthMetric,
	time_start: time_t,
	time_end: time_t,
	scope: HealthServiceTimeScope,
) -> HealthValue {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2440", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sum the metric. The result is the same as calling \ref health_service_sum_averaged(). This
! operation is only applicable for metrics that accumulate, like HealthMetricStepCount,
! HealthMetricActiveSeconds, etc.*/
pub const HealthAggregation_HealthAggregationSum: HealthAggregation = 0;
/**! Use the average of the metric. This is only applicable for metrics that measure instantaneous
! values, like HealthMetricHeartRateBPM*/
pub const HealthAggregation_HealthAggregationAvg: HealthAggregation = 1;
/**! Use the minimum value of the metric. This is only applicable for metrics that measure
! instantaneous values, like HealthMetricHeartRateBPM*/
pub const HealthAggregation_HealthAggregationMin: HealthAggregation = 2;
/**! Use the maximum value of the metric. This is only applicable for metrics that measure
! instantaneous values, like HealthMetricHeartRateBPM*/
pub const HealthAggregation_HealthAggregationMax: HealthAggregation = 3;
/**! Used by \ref health_service_aggregate_averaged() to specify what type of aggregation to perform.
! This aggregation is applied to the metric before the average is computed.*/
pub type HealthAggregation = ::core::ffi::c_uchar;
/**! Return the value of an aggregated metric over a given time range. This call is more
! flexible than health_service_sum_averaged because it lets you specify which aggregation function
! to perform.
!
! The aggregation function `aggregation` is applied to the metric `metric` over the given time
! range `time_start` to `time_end` first, and then an average is computed based on the passed in
! `scope`.
!
! For example, if you want to get the average number of steps taken from 12 AM (midnight) to 9 AM
! across all days you would specify:
! \code{.c}
! time_t time_start = time_start_of_today();
! time_t time_end = time_start + (9 * SECONDS_PER_HOUR);
! HealthValue value = health_service_aggregate_averaged(HealthMetricStepCount, time_start,
!    time_end, HealthAggregationSum, HealthServiceTimeScopeDaily);
! \endcode
!
! If you want to compute the average heart rate on Mondays and today is a Monday, you would
! specify:
! \code{.c}
! time_t time_start = time_start_of_today(),
! time_t time_end = time_start + SECONDS_PER_DAY,
! HealthValue value = health_service_aggregate_averaged(HealthMetricHeartRateBPM, time_start,
!    time_end, HealthAggregationAvg, HealthServiceTimeScopeWeekly);
! \endcode
!
! To get the average of the minimum heart rate seen on Mondays for example, you would instead
! pass in `HealthAggregationMin`
!
! Certain HealthAggregation operations are only applicable to certain types of metrics. See the
! notes above on \ref HealthAggregation for details. Use
! \ref health_service_metric_aggregate_averaged_accessible to check for applicability at run
! time.
!
! @param metric Which \ref HealthMetric to query.
! @param time_start UTC time of the start of the query interval.
! @param time_end UTC time of the end of the query interval.
! @param aggregation the aggregation function to perform on the metric. This operation is
!   performed across the passed in time range `time_start` to `time_end`.
! @param scope \ref HealthServiceTimeScope value describing how the average should be computed.
!  Use `HealthServiceTimeScopeOnce` to not compute an average.
! @return The average of the aggregation performed on the given metric over the given time range,
!  if available.*/
/** health_service_aggregate_averaged
  -> Added in 83
gneiss: Vtable Slot 617, Offset 2468*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.health_service_aggregate_averaged")]
pub unsafe extern "C" fn health_service_aggregate_averaged(
	metric: HealthMetric,
	time_start: time_t,
	time_end: time_t,
	aggregation: HealthAggregation,
	scope: HealthServiceTimeScope,
) -> HealthValue {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2468", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Expresses a set of \ref HealthActivity values as a bitmask.
pub type HealthActivityMask = u32;
///! No special activity.
pub const HealthActivity_HealthActivityNone: HealthActivity = 0;
///! The 'sleeping' activity.
pub const HealthActivity_HealthActivitySleep: HealthActivity = 1;
///! The 'restful sleeping' activity.
pub const HealthActivity_HealthActivityRestfulSleep: HealthActivity = 2;
///! The 'walk' activity.
pub const HealthActivity_HealthActivityWalk: HealthActivity = 4;
///! The 'run' activity.
pub const HealthActivity_HealthActivityRun: HealthActivity = 8;
///! The 'generic' activity.
pub const HealthActivity_HealthActivityOpenWorkout: HealthActivity = 16;
///! Health-related activities that can be accessed using
pub type HealthActivity = ::core::ffi::c_uchar;
/**! Return a \ref HealthActivityMask containing a set of bits, one set for each
! activity that is currently active.
! @return A bitmask with zero or more \ref HealthActivityMask bits set as appropriate.*/
/** health_service_peek_current_activities
  -> Added in 77
gneiss: Vtable Slot 605, Offset 2420*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.health_service_peek_current_activities")]
pub unsafe extern "C" fn health_service_peek_current_activities() -> HealthActivityMask {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2420", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Callback used by \ref health_service_activities_iterate().
! @param activity Which activity the caller is being informed about.
! @param time_start Start UTC time of the activity.
! @param time_end End UTC time of the activity.
! @param context The `context` parameter initially passed
!     to \ref health_service_activities_iterate().
! @return `true` if you are interested in more activities, or `false` to stop iterating.*/
pub type HealthActivityIteratorCB = ::core::option::Option<
	unsafe extern "C" fn(
		activity: HealthActivity,
		time_start: time_t,
		time_end: time_t,
		context: *mut ::core::ffi::c_void,
	) -> bool,
>;
///! Iterate into the past.
pub const HealthIterationDirection_HealthIterationDirectionPast: HealthIterationDirection = 0;
///! Iterate into the future.
pub const HealthIterationDirection_HealthIterationDirectionFuture: HealthIterationDirection = 1;
/**! Iteration direction, passed to \ref health_service_activities_iterate().
! When iterating backwards (`HealthIterationDirectionPast`), activities that have a greater value
! for `time_end` come first.
! When iterating forward (`HealthIterationDirectionFuture`), activities that have a smaller value
! for `time_start` come first.*/
pub type HealthIterationDirection = ::core::ffi::c_uchar;
/**! Iterates backwards or forward within a given time span to list all recorded activities.
! For example, this can be used to find the last recorded sleep phase or all deep sleep phases in
! a given time range. Any activity that overlaps with `time_start` and `time_end` will be
! included, even if the start time starts before `time_start` or end time ends after `time_end`.
! @param activity_mask A bitmask containing set of activities you are interested in.
! @param time_start UTC time of the earliest time you are interested in.
! @param time_end UTC time of the latest time you are interested in.
! @param direction The direction in which to iterate.
! @param callback Developer-supplied callback that is called for each activity iterated over.
! @param context Developer-supplied context pointer that is passed to the callback.*/
/** health_service_activities_iterate
  -> Added in 77
gneiss: Vtable Slot 599, Offset 2396*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.health_service_activities_iterate")]
pub unsafe extern "C" fn health_service_activities_iterate(
	activity_mask: HealthActivityMask,
	time_start: time_t,
	time_end: time_t,
	direction: HealthIterationDirection,
	callback: HealthActivityIteratorCB,
	context: *mut ::core::ffi::c_void,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2396", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Return values are available and represent the collected health information.
pub const HealthServiceAccessibilityMask_HealthServiceAccessibilityMaskAvailable:
	HealthServiceAccessibilityMask = 1;
///! The user hasn't granted permission.
pub const HealthServiceAccessibilityMask_HealthServiceAccessibilityMaskNoPermission:
	HealthServiceAccessibilityMask = 2;
/**! The queried combination of time span and \ref HealthMetric or \ref HealthActivityMask
! is currently unsupported.*/
pub const HealthServiceAccessibilityMask_HealthServiceAccessibilityMaskNotSupported:
	HealthServiceAccessibilityMask = 4;
///! No samples were recorded for the given time span.
pub const HealthServiceAccessibilityMask_HealthServiceAccessibilityMaskNotAvailable:
	HealthServiceAccessibilityMask = 8;
/**! Possible values returned by \ref health_service_metric_accessible().
! The values are used in combination as a bitmask.
! For example, to check if any data is available for a given request use:
! bool any_data_available = value & HealthServiceAccessibilityMaskAvailable;*/
pub type HealthServiceAccessibilityMask = ::core::ffi::c_uchar;
/**! Check if a certain combination of metric and time span is accessible using
! \ref health_service_sum by returning a value of \ref HealthServiceAccessibilityMask. Developers
! should check if the return value is \ref HealthServiceAccessibilityMaskAvailable before calling
! \ref health_service_sum.
!
! Note that this call is the same as calling `health_service_metric_averaged_accessible(metric,
! time_start, time_end, HealthServiceTimeScopeOnce)`
!
! @param metric The metric to query for data.
! @param time_start Earliest UTC time you are interested in.
! @param time_end Latest UTC time you are interested in.
! @return A \ref HealthServiceAccessibilityMask representing the accessible metrics
! in this time range.*/
/** health_service_metric_accessible
  -> Added in 77
gneiss: Vtable Slot 604, Offset 2416*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.health_service_metric_accessible")]
pub unsafe extern "C" fn health_service_metric_accessible(
	metric: HealthMetric,
	time_start: time_t,
	time_end: time_t,
) -> HealthServiceAccessibilityMask {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2416", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Check if a certain combination of metric, time span, and scope is accessible for calculating
! summed, averaged data by returning a value of \ref HealthServiceAccessibilityMask. Developers
! should check if the return value is \ref HealthServiceAccessibilityMaskAvailable before calling
! \ref health_service_sum_averaged.
!
! Note that this call is the same as calling
! `health_service_metric_aggregate_averaged_accessible(metric, time_start, time_end,
!  HealthAggregationSum, HealthServiceTimeScopeOnce)`
!
! @param metric The metric to query for averaged data.
! @param time_start Earliest UTC time you are interested in.
! @param time_end Latest UTC time you are interested in.
! @param scope \ref HealthServiceTimeScope value describing how the average should be computed.
! @return A \ref HealthServiceAccessibilityMask value decribing whether averaged data is available.*/
/** health_service_metric_averaged_accessible
  -> Added in 79
gneiss: Vtable Slot 609, Offset 2436*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.health_service_metric_averaged_accessible")]
pub unsafe extern "C" fn health_service_metric_averaged_accessible(
	metric: HealthMetric,
	time_start: time_t,
	time_end: time_t,
	scope: HealthServiceTimeScope,
) -> HealthServiceAccessibilityMask {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2436", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Check if a certain combination of metric, time span, aggregation operation, and scope is
! accessible for calculating aggregated, averaged data by returning a value of
! \ref HealthServiceAccessibilityMask. Developers should check if the return value is
! \ref HealthServiceAccessibilityMaskAvailable before calling
! \ref health_service_aggregate_averaged.
! @param metric The metric to query for averaged data.
! @param time_start Earliest UTC time you are interested in.
! @param time_end Latest UTC time you are interested in.
! @param aggregation The aggregation to perform
! @param scope \ref HealthServiceTimeScope value describing how the average should be computed.
! @return A \ref HealthServiceAccessibilityMask value decribing whether averaged data is available.*/
/** health_service_metric_aggregate_averaged_accessible
  -> Added in 83
gneiss: Vtable Slot 619, Offset 2476*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.health_service_metric_aggregate_averaged_accessible")]
pub unsafe extern "C" fn health_service_metric_aggregate_averaged_accessible(
	metric: HealthMetric,
	time_start: time_t,
	time_end: time_t,
	aggregation: HealthAggregation,
	scope: HealthServiceTimeScope,
) -> HealthServiceAccessibilityMask {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2476", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Check if a certain combination of metric, \ref HealthActivityMask and time span is
! accessible. Developers should check if the return value is
! \ref HealthServiceAccessibilityMaskAvailable before calling any other HealthService APIs that
! involve the given activities.
! @param activity_mask A bitmask of activities you are interested in.
! @param time_start Earliest UTC time you are interested in.
! @param time_end Latest UTC time you are interested in.
! @return A \ref HealthServiceAccessibilityMask representing which of the
! passed \ref HealthActivityMask values are available under the given constraints.*/
/** health_service_any_activity_accessible
  -> Added in 77
gneiss: Vtable Slot 600, Offset 2400*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.health_service_any_activity_accessible")]
pub unsafe extern "C" fn health_service_any_activity_accessible(
	activity_mask: HealthActivityMask,
	time_start: time_t,
	time_end: time_t,
) -> HealthServiceAccessibilityMask {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2400", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! All data is considered as outdated and apps should re-read all health data.
! This happens after an app is subscribed via \ref health_service_events_subscribe(),
! on a change of the day, or in other cases that significantly change the underlying data.*/
pub const HealthEventType_HealthEventSignificantUpdate: HealthEventType = 0;
/**! Recent values around \ref HealthMetricStepCount, \ref HealthMetricActiveSeconds,
! or \ref HealthMetricWalkedDistanceMeters have changed.*/
pub const HealthEventType_HealthEventMovementUpdate: HealthEventType = 1;
/**! Recent values around \ref HealthMetricSleepSeconds, \ref HealthMetricSleepRestfulSeconds,
! \ref HealthActivitySleep, and \ref HealthActivityRestfulSleep changed.*/
pub const HealthEventType_HealthEventSleepUpdate: HealthEventType = 2;
///! A metric has crossed the threshold set by \ref health_service_register_metric_alert.
pub const HealthEventType_HealthEventMetricAlert: HealthEventType = 3;
///! Value of \ref HealthMetricHeartRateBPM or \ref HealthMetricHeartRateRawBPM has changed.
pub const HealthEventType_HealthEventHeartRateUpdate: HealthEventType = 4;
///! A new HRV peak-to-peak interval reading is available.
pub const HealthEventType_HealthEventHRVUpdate: HealthEventType = 5;
///! Health event enum. Passed into the \ref HealthEventHandler.
pub type HealthEventType = ::core::ffi::c_uchar;
/**! Developer-supplied event handler, called when a health-related event occurs after subscribing
! via \ref health_service_events_subscribe();
! @param event The type of health-related event that occured.
! @param context The developer-supplied context pointer.*/
pub type HealthEventHandler = ::core::option::Option<
	unsafe extern "C" fn(event: HealthEventType, context: *mut ::core::ffi::c_void),
>;
/**! Subscribe to HealthService events. This allocates a cache on the application's heap of up
! to 2048 bytes that will be de-allocated if you call \ref health_service_events_unsubscribe().
! If there's not enough heap available, this function will return `false` and will not
! subscribe to any events.
! @param handler Developer-supplied event handler function.
! @param context Developer-supplied context pointer.
! @return `true` on success, `false` on failure.*/
/** health_service_events_subscribe
  -> Added in 77
gneiss: Vtable Slot 601, Offset 2404*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.health_service_events_subscribe")]
pub unsafe extern "C" fn health_service_events_subscribe(
	handler: HealthEventHandler,
	context: *mut ::core::ffi::c_void,
) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2404", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Unsubscribe from HealthService events.
! @return `true` on success, `false` on failure.*/
/** health_service_events_unsubscribe
  -> Added in 77
gneiss: Vtable Slot 602, Offset 2408*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.health_service_events_unsubscribe")]
pub unsafe extern "C" fn health_service_events_unsubscribe() -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2408", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set the desired sampling period for heart rate readings. Normally, the system will sample the
! heart rate using a sampling period that is automatically chosen to provide useful information
! without undue battery drain (it automatically samples more often during periods of intense
! activity, and less often when the user is idle). If desired though, an application can request a
! specific sampling period using this call. The system will use this as a suggestion, but does not
! guarantee that the requested period will be used. The actual sampling period may be greater or
! less due to system needs or heart rate sensor reading quality issues.
!
! Each time a new heart rate reading becomes available, a `HealthEventHeartRateUpdate` event will
! be sent to the application's `HealthEventHandler`. The sample period request will remain in
! effect the entire time the app is running unless it is explicitly cancelled (by calling this
! method again with 0 as the desired interval). If the app exits without first cancelling the
! request, it will remain in effect even for a limited time afterwards. To determine how long it
! will remain active after the app exits, use
! `health_service_get_heart_rate_sample_period_expiration_sec`.
!
! Unless the app explicitly needs to access to historical high-resolution heart rate data, it is
! best practice to always cancel the sample period request before exiting in order to maximize
! battery life. Historical heart rate data can be accessed using the
! `health_service_get_minute_history` call.
! @param interval_sec desired interval between heart rate reading updates. Pass 0 to
!   go back to automatically chosen intervals.
! @return `true` on success, `false` on failure*/
/** health_service_set_heart_rate_sample_period
  -> Added in 86
gneiss: Vtable Slot 629, Offset 2516*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.health_service_set_heart_rate_sample_period")]
pub unsafe extern "C" fn health_service_set_heart_rate_sample_period(interval_sec: u16) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2516", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Return how long a heart rate sample period request (sent via
! `health_service_set_heart_rate_sample_period`) will remain active after the app exits. If
! there is no current request by this app, this call will return 0.
! @return The number of seconds the heart rate sample period request will remain active after
! the app exits, or 0 if there is no active request by this app.*/
/** health_service_get_heart_rate_sample_period_expiration_sec
  -> Added in 86
gneiss: Vtable Slot 628, Offset 2512*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.health_service_get_heart_rate_sample_period_expiration_sec")]
pub unsafe extern "C" fn health_service_get_heart_rate_sample_period_expiration_sec() -> u16 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2512", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Register for an alert when a metric crosses the given threshold. When the metric crosses this
! threshold (either goes above or below it), a \ref HealthEventMetricAlert event will be
! generated. To cancel this registration, pass the returned \ref HealthMetricAlert value to
! \ref health_service_cancel_metric_alert. The only metric currently supported by this call is
! \ref HealthMetricHeartRateBPM, but future versions may support additional metrics. To see if a
! specific metric is supported by this call, use:
! \code{.c}
! time_t now = time(NULL);
! HealthServiceAccessibilityMask accessible =
!     health_service_metric_aggregate_averaged_accessible(metric, now, now, HealthAggregationAvg,
!     HealthServiceTimeScopeOnce);
! bool alert_supported = (accessible & HealthServiceAccessibilityMaskAvailable);
! \endcode
!
! In the current implementation, only one alert per metric can be registered at a time. Future
! implementations may support two or more simulataneous alert registrations per metric. To change
! the alert threshold in the current implementation, cancel the original registration
! using `health_service_cancel_metric_alert` before registering the new threshold.
! @param metric Which \ref HealthMetric to query.
! @param threshold The threshold value.
! @return handle to the alert registration on success, NULL on failure*/
/** health_service_register_metric_alert
  -> Added in 83
gneiss: Vtable Slot 621, Offset 2484*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.health_service_register_metric_alert")]
pub unsafe extern "C" fn health_service_register_metric_alert(
	metric: HealthMetric,
	threshold: HealthValue,
) -> *mut HealthMetricAlert {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2484", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Cancel an metric alert previously created with \ref health_service_register_metric_alert.
! @param alert the \ref HealthMetricAlert previously returned by
!  \ref health_service_register_metric_alert
! @return `true` on success, `false` on failure*/
/** health_service_cancel_metric_alert
  -> Added in 83
gneiss: Vtable Slot 618, Offset 2472*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.health_service_cancel_metric_alert")]
pub unsafe extern "C" fn health_service_cancel_metric_alert(alert: *mut HealthMetricAlert) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2472", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
pub const AmbientLightLevel_AmbientLightLevelUnknown: AmbientLightLevel = 0;
pub const AmbientLightLevel_AmbientLightLevelVeryDark: AmbientLightLevel = 1;
pub const AmbientLightLevel_AmbientLightLevelDark: AmbientLightLevel = 2;
pub const AmbientLightLevel_AmbientLightLevelLight: AmbientLightLevel = 3;
pub const AmbientLightLevel_AmbientLightLevelVeryLight: AmbientLightLevel = 4;
///! Light level enum
pub type AmbientLightLevel = ::core::ffi::c_uchar;
/**! Structure representing a single minute data record returned
! by \ref health_service_get_minute_history().
! The `orientation` field encodes the angle of the watch in the x-y plane (the "yaw") in the
! lower 4 bits (360 degrees linearly mapped to 1 of 16 different values) and the angle to the
! z axis (the "pitch") in the upper 4 bits.
! The `vmc` value is a measure of the total amount of movement seen by the watch. More vigorous
! movement yields higher VMC values.*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct HealthMinuteData {
	///!< Number of steps taken in this minute.
	pub steps:          u8,
	///!< Quantized average orientation.
	pub orientation:    u8,
	///!< Vector Magnitude Counts (vmc).
	pub vmc:            u16,
	pub _bitfield_1:    __BindgenBitfieldUnit<[u8; 1usize]>,
	///!< heart rate in beats per minute
	pub heart_rate_bpm: u8,
	///!< Reserved for future use.
	pub reserved:       [u8; 6usize],
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of HealthMinuteData"][::core::mem::size_of::<HealthMinuteData>() - 12usize];
	["Alignment of HealthMinuteData"][::core::mem::align_of::<HealthMinuteData>() - 2usize];
	["Offset of field: HealthMinuteData::steps"]
		[::core::mem::offset_of!(HealthMinuteData, steps) - 0usize];
	["Offset of field: HealthMinuteData::orientation"]
		[::core::mem::offset_of!(HealthMinuteData, orientation) - 1usize];
	["Offset of field: HealthMinuteData::vmc"]
		[::core::mem::offset_of!(HealthMinuteData, vmc) - 2usize];
	["Offset of field: HealthMinuteData::heart_rate_bpm"]
		[::core::mem::offset_of!(HealthMinuteData, heart_rate_bpm) - 5usize];
	["Offset of field: HealthMinuteData::reserved"]
		[::core::mem::offset_of!(HealthMinuteData, reserved) - 6usize];
};
impl HealthMinuteData {
	#[inline]
	#[allow(unnecessary_transmutes)]
	pub fn is_invalid(&self) -> bool {
		unsafe { ::core::mem::transmute(self._bitfield_1.get_const::<0usize, 1u8>() as u8) }
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub fn set_is_invalid(&mut self, val: bool) {
		unsafe {
			let val: u8 = ::core::mem::transmute(val);
			self._bitfield_1.set_const::<0usize, 1u8>(val as u64)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub unsafe fn is_invalid_raw(this: *const Self) -> bool {
		unsafe {
			::core::mem::transmute(
				<__BindgenBitfieldUnit<[u8; 1usize]>>::raw_get_const::<0usize, 1u8>(
					::core::ptr::addr_of!((*this)._bitfield_1),
				) as u8,
			)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub unsafe fn set_is_invalid_raw(this: *mut Self, val: bool) {
		unsafe {
			let val: u8 = ::core::mem::transmute(val);
			<__BindgenBitfieldUnit<[u8; 1usize]>>::raw_set_const::<0usize, 1u8>(
				::core::ptr::addr_of_mut!((*this)._bitfield_1),
				val as u64,
			)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub fn light(&self) -> AmbientLightLevel {
		unsafe { ::core::mem::transmute(self._bitfield_1.get_const::<1usize, 3u8>() as u8) }
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub fn set_light(&mut self, val: AmbientLightLevel) {
		unsafe {
			let val: u8 = ::core::mem::transmute(val);
			self._bitfield_1.set_const::<1usize, 3u8>(val as u64)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub unsafe fn light_raw(this: *const Self) -> AmbientLightLevel {
		unsafe {
			::core::mem::transmute(
				<__BindgenBitfieldUnit<[u8; 1usize]>>::raw_get_const::<1usize, 3u8>(
					::core::ptr::addr_of!((*this)._bitfield_1),
				) as u8,
			)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub unsafe fn set_light_raw(this: *mut Self, val: AmbientLightLevel) {
		unsafe {
			let val: u8 = ::core::mem::transmute(val);
			<__BindgenBitfieldUnit<[u8; 1usize]>>::raw_set_const::<1usize, 3u8>(
				::core::ptr::addr_of_mut!((*this)._bitfield_1),
				val as u64,
			)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub fn padding(&self) -> u8 {
		unsafe { ::core::mem::transmute(self._bitfield_1.get_const::<4usize, 4u8>() as u8) }
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub fn set_padding(&mut self, val: u8) {
		unsafe {
			let val: u8 = ::core::mem::transmute(val);
			self._bitfield_1.set_const::<4usize, 4u8>(val as u64)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub unsafe fn padding_raw(this: *const Self) -> u8 {
		unsafe {
			::core::mem::transmute(
				<__BindgenBitfieldUnit<[u8; 1usize]>>::raw_get_const::<4usize, 4u8>(
					::core::ptr::addr_of!((*this)._bitfield_1),
				) as u8,
			)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub unsafe fn set_padding_raw(this: *mut Self, val: u8) {
		unsafe {
			let val: u8 = ::core::mem::transmute(val);
			<__BindgenBitfieldUnit<[u8; 1usize]>>::raw_set_const::<4usize, 4u8>(
				::core::ptr::addr_of_mut!((*this)._bitfield_1),
				val as u64,
			)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub fn new_bitfield_1(
		is_invalid: bool,
		light: AmbientLightLevel,
		padding: u8,
	) -> __BindgenBitfieldUnit<[u8; 1usize]> {
		let mut __bindgen_bitfield_unit: __BindgenBitfieldUnit<[u8; 1usize]> = Default::default();
		__bindgen_bitfield_unit.set_const::<0usize, 1u8>({
			let is_invalid: u8 = unsafe { ::core::mem::transmute(is_invalid) };
			is_invalid as u64
		});
		__bindgen_bitfield_unit.set_const::<1usize, 3u8>({
			let light: u8 = unsafe { ::core::mem::transmute(light) };
			light as u64
		});
		__bindgen_bitfield_unit.set_const::<4usize, 4u8>({
			let padding: u8 = unsafe { ::core::mem::transmute(padding) };
			padding as u64
		});
		__bindgen_bitfield_unit
	}
}
/**! Return historical minute data records. This fills in the `minute_data` array parameter with
! minute by minute statistics of the user's steps, average watch orientation, etc. The data is
! returned in time order, with the oldest minute data returned at `minute_data[0]`.
! @param minute_data Pointer to an array of \ref HealthMinuteData records that will be filled
!      in with the historical minute data.
! @param max_records The maximum number of records the `minute_data` array can hold.
! @param[in,out] time_start On entry, the UTC time of the first requested record. On exit,
!      the UTC time of the first second of the first record actually returned.
!      If `time_start` on entry is somewhere in the middle of a minute interval, this function
!      behaves as if the caller passed in the start of that minute.
! @param[in,out] time_end On entry, the UTC time of the end of the requested range of records. On
!      exit, the UTC time of the end of the last record actually returned (i.e. start time of last
!      record + 60). If `time_end` on entry is somewhere in the middle of a minute interval, this
!      function behaves as if the caller passed in the end of that minute.
! @return Actual number of records returned. May be less then the maximum requested.
! @note If the return value is zero, `time_start` and `time_end` are meaningless.
!      It's not guaranteed that all records contain valid data, even if the return value is
!      greater than zero. Check `HealthMinuteData.is_invalid` to see if a given record contains
!      valid data.*/
/** health_service_get_minute_history
  -> Added in 77
gneiss: Vtable Slot 603, Offset 2412*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.health_service_get_minute_history")]
pub unsafe extern "C" fn health_service_get_minute_history(
	minute_data: *mut HealthMinuteData,
	max_records: u32,
	time_start: *mut time_t,
	time_end: *mut time_t,
) -> u32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2412", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! The measurement system is unknown, or does not apply to the chosen metric.
pub const MeasurementSystem_MeasurementSystemUnknown: MeasurementSystem = 0;
///! The metric measurement system.
pub const MeasurementSystem_MeasurementSystemMetric: MeasurementSystem = 1;
///! The imperial measurement system.
pub const MeasurementSystem_MeasurementSystemImperial: MeasurementSystem = 2;
///! Types of measurement system a \ref HealthMetric may be measured in.
pub type MeasurementSystem = ::core::ffi::c_uchar;
/**! Get the preferred measurement system for a given \ref HealthMetric, if the user has chosen
! a preferred system and it is applicable to that metric.
! @param metric A metric value chosen from \ref HealthMetric.
! @return A value from \ref MeasurementSystem if applicable, else \ref MeasurementSystemUnknown.*/
/** health_service_get_measurement_system_for_display
  -> Added in 80
gneiss: Vtable Slot 611, Offset 2444*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.health_service_get_measurement_system_for_display")]
pub unsafe extern "C" fn health_service_get_measurement_system_for_display(
	metric: HealthMetric,
) -> MeasurementSystem {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2444", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @addtogroup DataLogging
! \brief Enables logging data asynchronously to a mobile app
!
! In Pebble OS, data logging is a data storage and transfer subsystem that allows watchapps to
! save data on non-volatile storage devices when the phone is not available to process it. The
! API provides your watchapp with a mechanism for short-term data buffering for asynchronous data
! transmission to a mobile app.
!
! Using this API, your Pebble watchapp can create an arbitrary number of logs, but you’re
! limited in the amount of storage space you can use. Note that approximately 640K is available
! for data logging, which is shared among all watchapps that use it. This value is subject to
! change in the future. When the data spool is full, an app will start overwriting its own data.
! An app cannot overwrite another apps's data. However, the other app might have 0 bytes for data
! logging.
!
! Your app can log data to a session. Every new block of data is appended to the session.
! The data is then sent to the associated phone application at the earliest convenience.
! If a phone is available, the data is sent directly to the phone. Otherwise, it is saved to the
! watch storage until the watch is connected to a phone.
!
!
! For example:
!
! To create a data logging session for 4-byte unsigned integers with a tag of 0x1234, you would
! do this: \code{.c}
!
! DataLoggingSessionRef logging_session = data_logging_create(0x1234, DATA_LOGGING_UINT, 4,
!                                                             false);
!
! // Fake creating some data and logging it to the session.
! uint32_t data[] = { 1, 2, 3};
! data_logging_log(logging_session, &data, 3);
!
! // Fake creating more data and logging that as well.
! uint32_t data2[] = { 1, 2 };
! data_logging_log(logging_session, &data, 2);
!
! // When we don't need to log anything else, we can close off the session.
! data_logging_finish(logging_session);
! \endcode
!
! @{*/
pub type DataLoggingSessionRef = *mut ::core::ffi::c_void;
/**! Array of bytes. Remember that this is the type of a single item in the logging session, so
! using this type means you'll be logging multiple byte arrays (each a fixed length described
! by item_length) for the duration of the session.*/
pub const DataLoggingItemType_DATA_LOGGING_BYTE_ARRAY: DataLoggingItemType = 0;
///! Unsigned integer. This may be a 1, 2, or 4 byte integer depending on the item_length parameter
pub const DataLoggingItemType_DATA_LOGGING_UINT: DataLoggingItemType = 2;
///! Signed integer. This may be a 1, 2, or 4 byte integer depending on the item_length parameter
pub const DataLoggingItemType_DATA_LOGGING_INT: DataLoggingItemType = 3;
/**! The different types of session data that Pebble supports. This type describes the type of a
! singular item in the data session. Every item in a given session is the same type and size.*/
pub type DataLoggingItemType = ::core::ffi::c_uchar;
///!< Successful operation
pub const DataLoggingResult_DATA_LOGGING_SUCCESS: DataLoggingResult = 0;
///!< Someone else is writing to this logging session
pub const DataLoggingResult_DATA_LOGGING_BUSY: DataLoggingResult = 1;
///!< No more space to save data
pub const DataLoggingResult_DATA_LOGGING_FULL: DataLoggingResult = 2;
///!< The logging session does not exist
pub const DataLoggingResult_DATA_LOGGING_NOT_FOUND: DataLoggingResult = 3;
///!< The logging session was made inactive
pub const DataLoggingResult_DATA_LOGGING_CLOSED: DataLoggingResult = 4;
///!< An invalid parameter was passed to one of the functions
pub const DataLoggingResult_DATA_LOGGING_INVALID_PARAMS: DataLoggingResult = 5;
///!< An internal error occurred
pub const DataLoggingResult_DATA_LOGGING_INTERNAL_ERR: DataLoggingResult = 6;
///! Enumerated values describing the possible outcomes of data logging operations
pub type DataLoggingResult = ::core::ffi::c_uchar;
/**! Create a new data logging session.
!
! @param tag A tag associated with the logging session.
! @param item_type The type of data stored in this logging session
! @param item_length The size of a single data item in bytes
! @param resume True if we want to look for a logging session of the same tag and
!   resume logging to it. If this is false and a session with the specified tag exists, that
!   session will be closed and a new session will be opened.
! @return An opaque reference to the data logging session*/
/** data_logging_create
  -> Added in 0
gneiss: Vtable Slot 71, Offset 284*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.data_logging_create")]
pub unsafe extern "C" fn data_logging_create(
	tag: u32,
	item_type: DataLoggingItemType,
	item_length: u16,
	resume: bool,
) -> DataLoggingSessionRef {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =284", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Finish up a data logging_session. Logging data is kept until it has successfully been
! transferred over to the phone, but no data may be added to the session after this function is
! called.
!
! @param logging_session a reference to the data logging session previously allocated using
!   data_logging_create*/
/** data_logging_finish
  -> Added in 0
gneiss: Vtable Slot 72, Offset 288*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.data_logging_finish")]
pub unsafe extern "C" fn data_logging_finish(logging_session: DataLoggingSessionRef) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =288", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Add data to the data logging session. If a phone is available, the data is sent directly
! to the phone. Otherwise, it is saved to the watch storage until the watch is connected to a
! phone.
!
! @param logging_session a reference to the data logging session you want to add the data to
! @param data a pointer to the data buffer that contains multiple items
! @param num_items the number of items to log. This means data must be at least
!    (num_items * item_length) long in bytes
! @return
! DATA_LOGGING_SUCCESS on success
!
! @return
! DATA_LOGGING_NOT_FOUND if the logging session is invalid
!
! @return
! DATA_LOGGING_CLOSED if the sesion is not active
!
! @return
! DATA_LOGGING_BUSY if the sesion is not available for writing
!
! @return
! DATA_LOGGING_INVALID_PARAMS if num_items is 0 or data is NULL*/
/** data_logging_log
  -> Added in 0
gneiss: Vtable Slot 73, Offset 292*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.data_logging_log")]
pub unsafe extern "C" fn data_logging_log(
	logging_session: DataLoggingSessionRef,
	data: *const ::core::ffi::c_void,
	num_items: u32,
) -> DataLoggingResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =292", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @addtogroup UUID
! @{*/
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct Uuid {
	pub byte0:  u8,
	pub byte1:  u8,
	pub byte2:  u8,
	pub byte3:  u8,
	pub byte4:  u8,
	pub byte5:  u8,
	pub byte6:  u8,
	pub byte7:  u8,
	pub byte8:  u8,
	pub byte9:  u8,
	pub byte10: u8,
	pub byte11: u8,
	pub byte12: u8,
	pub byte13: u8,
	pub byte14: u8,
	pub byte15: u8,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of Uuid"][::core::mem::size_of::<Uuid>() - 16usize];
	["Alignment of Uuid"][::core::mem::align_of::<Uuid>() - 1usize];
	["Offset of field: Uuid::byte0"][::core::mem::offset_of!(Uuid, byte0) - 0usize];
	["Offset of field: Uuid::byte1"][::core::mem::offset_of!(Uuid, byte1) - 1usize];
	["Offset of field: Uuid::byte2"][::core::mem::offset_of!(Uuid, byte2) - 2usize];
	["Offset of field: Uuid::byte3"][::core::mem::offset_of!(Uuid, byte3) - 3usize];
	["Offset of field: Uuid::byte4"][::core::mem::offset_of!(Uuid, byte4) - 4usize];
	["Offset of field: Uuid::byte5"][::core::mem::offset_of!(Uuid, byte5) - 5usize];
	["Offset of field: Uuid::byte6"][::core::mem::offset_of!(Uuid, byte6) - 6usize];
	["Offset of field: Uuid::byte7"][::core::mem::offset_of!(Uuid, byte7) - 7usize];
	["Offset of field: Uuid::byte8"][::core::mem::offset_of!(Uuid, byte8) - 8usize];
	["Offset of field: Uuid::byte9"][::core::mem::offset_of!(Uuid, byte9) - 9usize];
	["Offset of field: Uuid::byte10"][::core::mem::offset_of!(Uuid, byte10) - 10usize];
	["Offset of field: Uuid::byte11"][::core::mem::offset_of!(Uuid, byte11) - 11usize];
	["Offset of field: Uuid::byte12"][::core::mem::offset_of!(Uuid, byte12) - 12usize];
	["Offset of field: Uuid::byte13"][::core::mem::offset_of!(Uuid, byte13) - 13usize];
	["Offset of field: Uuid::byte14"][::core::mem::offset_of!(Uuid, byte14) - 14usize];
	["Offset of field: Uuid::byte15"][::core::mem::offset_of!(Uuid, byte15) - 15usize];
};
/**! Compares two UUIDs.
! @return True if the two UUIDs are equal, false if they are not.*/
/** uuid_equal
  -> Added in 19
gneiss: Vtable Slot 341, Offset 1364*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.uuid_equal")]
pub unsafe extern "C" fn uuid_equal(uu1: *const Uuid, uu2: *const Uuid) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1364", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Writes UUID in a string form into buffer that looks like the following...
! {12345678-1234-5678-1234-567812345678} or {NULL UUID} if NULL was passed.
! @param uuid The Uuid to write into the buffer as human-readable string
! @param buffer Memory to write the string to. Must be at least \ref UUID_STRING_BUFFER_LENGTH bytes long.*/
/** uuid_to_string
  -> Added in 19
gneiss: Vtable Slot 342, Offset 1368*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.uuid_to_string")]
pub unsafe extern "C" fn uuid_to_string(uuid: *const Uuid, buffer: *mut ::core::ffi::c_char) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1368", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @addtogroup Logging Logging
!   \brief Functions related to logging from apps.
!
! This module contains the functions necessary to log messages through
! Bluetooth.
! @note It is no longer necessary to enable app logging output from the "settings->about" menu on the Pebble for
! them to be transmitted!  Instead use the "pebble logs" command included with the SDK to activate logs.  The logs
! will appear right in your console. Logging
! over Bluetooth is a fairly power hungry operation that non-developers will
! not need when your apps are distributed.
! @{*/
/** app_log
  -> Added in 0
gneiss: Vtable Slot 34, Offset 136*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_log")]
pub unsafe extern "C" fn app_log(
	log_level: u8,
	src_filename: *const ::core::ffi::c_char,
	src_line_number: ::core::ffi::c_int,
	fmt: *const ::core::ffi::c_char,
	args: ...
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =136", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Error level log message
pub const AppLogLevel_APP_LOG_LEVEL_ERROR: AppLogLevel = 1;
///! Warning level log message
pub const AppLogLevel_APP_LOG_LEVEL_WARNING: AppLogLevel = 50;
///! Info level log message
pub const AppLogLevel_APP_LOG_LEVEL_INFO: AppLogLevel = 100;
///! Debug level log message
pub const AppLogLevel_APP_LOG_LEVEL_DEBUG: AppLogLevel = 200;
///! Verbose Debug level log message
pub const AppLogLevel_APP_LOG_LEVEL_DEBUG_VERBOSE: AppLogLevel = 255;
///! Suggested log level values
pub type AppLogLevel = ::core::ffi::c_uchar;
///! The operation returned successfully
pub const DictionaryResult_DICT_OK: DictionaryResult = 0;
///! There was not enough backing storage to complete the operation
pub const DictionaryResult_DICT_NOT_ENOUGH_STORAGE: DictionaryResult = 2;
///! One or more arguments were invalid or uninitialized
pub const DictionaryResult_DICT_INVALID_ARGS: DictionaryResult = 4;
///! The lengths and/or count of the dictionary its tuples are inconsistent
pub const DictionaryResult_DICT_INTERNAL_INCONSISTENCY: DictionaryResult = 8;
/**! A requested operation required additional memory to be allocated, but
! the allocation failed, likely due to insufficient remaining heap memory.*/
pub const DictionaryResult_DICT_MALLOC_FAILED: DictionaryResult = 16;
///! Return values for dictionary write/conversion functions.
pub type DictionaryResult = ::core::ffi::c_uchar;
///! The value is an array of bytes
pub const TupleType_TUPLE_BYTE_ARRAY: TupleType = 0;
///! The value is a zero-terminated, UTF-8 C-string
pub const TupleType_TUPLE_CSTRING: TupleType = 1;
/**! The value is an unsigned integer. The tuple's `.length` field is used to
! determine the size of the integer (1, 2, or 4 bytes).*/
pub const TupleType_TUPLE_UINT: TupleType = 2;
/**! The value is a signed integer. The tuple's `.length` field is used to
! determine the size of the integer (1, 2, or 4 bytes).*/
pub const TupleType_TUPLE_INT: TupleType = 3;
///! Values representing the type of data that the `value` field of a Tuple contains
pub type TupleType = ::core::ffi::c_uchar;
/**! Data structure for one serialized key/value tuple
! @note The structure is variable length! The length depends on the value data that the tuple
! contains.*/
#[repr(C, packed)]
pub struct Tuple {
	///! The key
	pub key:         u32,
	pub _bitfield_1: __BindgenBitfieldUnit<[u8; 1usize]>,
	///! The length of `.value` in bytes
	pub length:      u16,
	pub value:       __IncompleteArrayField<Tuple__bindgen_ty_1>,
}
/**! @brief The value itself.
!
! The different union fields are provided for convenience, avoiding the need for manual casts.
! @note The array length is of incomplete length on purpose, to facilitate
! variable length data and because a data length of zero is valid.
! @note __Important: The integers are little endian!__*/
#[repr(C)]
pub struct Tuple__bindgen_ty_1 {
	///! The byte array value. Valid when `.type` is \ref TUPLE_BYTE_ARRAY.
	pub data:                __BindgenUnionField<[u8; 0usize]>,
	///! The C-string value. Valid when `.type` is \ref TUPLE_CSTRING.
	pub cstring:             __BindgenUnionField<[::core::ffi::c_char; 0usize]>,
	/**! The 8-bit unsigned integer value. Valid when `.type` is \ref TUPLE_UINT
	! and `.length` is 1 byte.*/
	pub uint8:               __BindgenUnionField<u8>,
	/**! The 16-bit unsigned integer value. Valid when `.type` is \ref TUPLE_UINT
	! and `.length` is 2 byte.*/
	pub uint16:              __BindgenUnionField<u16>,
	/**! The 32-bit unsigned integer value. Valid when `.type` is \ref TUPLE_UINT
	! and `.length` is 4 byte.*/
	pub uint32:              __BindgenUnionField<u32>,
	/**! The 8-bit signed integer value. Valid when `.type` is \ref TUPLE_INT
	! and `.length` is 1 byte.*/
	pub int8:                __BindgenUnionField<i8>,
	/**! The 16-bit signed integer value. Valid when `.type` is \ref TUPLE_INT
	! and `.length` is 2 byte.*/
	pub int16:               __BindgenUnionField<i16>,
	/**! The 32-bit signed integer value. Valid when `.type` is \ref TUPLE_INT
	! and `.length` is 4 byte.*/
	pub int32:               __BindgenUnionField<i32>,
	pub bindgen_union_field: u32,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of Tuple__bindgen_ty_1"][::core::mem::size_of::<Tuple__bindgen_ty_1>() - 4usize];
	["Alignment of Tuple__bindgen_ty_1"][::core::mem::align_of::<Tuple__bindgen_ty_1>() - 4usize];
	["Offset of field: Tuple__bindgen_ty_1::data"]
		[::core::mem::offset_of!(Tuple__bindgen_ty_1, data) - 0usize];
	["Offset of field: Tuple__bindgen_ty_1::cstring"]
		[::core::mem::offset_of!(Tuple__bindgen_ty_1, cstring) - 0usize];
	["Offset of field: Tuple__bindgen_ty_1::uint8"]
		[::core::mem::offset_of!(Tuple__bindgen_ty_1, uint8) - 0usize];
	["Offset of field: Tuple__bindgen_ty_1::uint16"]
		[::core::mem::offset_of!(Tuple__bindgen_ty_1, uint16) - 0usize];
	["Offset of field: Tuple__bindgen_ty_1::uint32"]
		[::core::mem::offset_of!(Tuple__bindgen_ty_1, uint32) - 0usize];
	["Offset of field: Tuple__bindgen_ty_1::int8"]
		[::core::mem::offset_of!(Tuple__bindgen_ty_1, int8) - 0usize];
	["Offset of field: Tuple__bindgen_ty_1::int16"]
		[::core::mem::offset_of!(Tuple__bindgen_ty_1, int16) - 0usize];
	["Offset of field: Tuple__bindgen_ty_1::int32"]
		[::core::mem::offset_of!(Tuple__bindgen_ty_1, int32) - 0usize];
};
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of Tuple"][::core::mem::size_of::<Tuple>() - 7usize];
	["Alignment of Tuple"][::core::mem::align_of::<Tuple>() - 1usize];
	["Offset of field: Tuple::key"][::core::mem::offset_of!(Tuple, key) - 0usize];
	["Offset of field: Tuple::length"][::core::mem::offset_of!(Tuple, length) - 5usize];
	["Offset of field: Tuple::value"][::core::mem::offset_of!(Tuple, value) - 7usize];
};
impl Tuple {
	#[inline]
	#[allow(unnecessary_transmutes)]
	pub fn type_(&self) -> TupleType {
		unsafe { ::core::mem::transmute(self._bitfield_1.get_const::<0usize, 8u8>() as u8) }
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub fn set_type(&mut self, val: TupleType) {
		unsafe {
			let val: u8 = ::core::mem::transmute(val);
			self._bitfield_1.set_const::<0usize, 8u8>(val as u64)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub unsafe fn type__raw(this: *const Self) -> TupleType {
		unsafe {
			::core::mem::transmute(
				<__BindgenBitfieldUnit<[u8; 1usize]>>::raw_get_const::<0usize, 8u8>(
					::core::ptr::addr_of!((*this)._bitfield_1),
				) as u8,
			)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub unsafe fn set_type_raw(this: *mut Self, val: TupleType) {
		unsafe {
			let val: u8 = ::core::mem::transmute(val);
			<__BindgenBitfieldUnit<[u8; 1usize]>>::raw_set_const::<0usize, 8u8>(
				::core::ptr::addr_of_mut!((*this)._bitfield_1),
				val as u64,
			)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub fn new_bitfield_1(type_: TupleType) -> __BindgenBitfieldUnit<[u8; 1usize]> {
		let mut __bindgen_bitfield_unit: __BindgenBitfieldUnit<[u8; 1usize]> = Default::default();
		__bindgen_bitfield_unit.set_const::<0usize, 8u8>({
			let type_: u8 = unsafe { ::core::mem::transmute(type_) };
			type_ as u64
		});
		__bindgen_bitfield_unit
	}
}
#[repr(C)]
#[derive(Debug)]
pub struct Dictionary {
	_unused: [u8; 0],
}
/**! An iterator can be used to iterate over the key/value
! tuples in an existing dictionary, using \ref dict_read_begin_from_buffer(),
! \ref dict_read_first() and \ref dict_read_next().
! An iterator can also be used to append key/value tuples to a dictionary,
! for example using \ref dict_write_data() or \ref dict_write_cstring().*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DictionaryIterator {
	///!< The dictionary being iterated
	pub dictionary: *mut Dictionary,
	///!< Points to the first memory address after the last byte of the dictionary
	pub end:        *const ::core::ffi::c_void,
	/**! Points to the next Tuple in the dictionary. Given the end of the
	! Dictionary has not yet been reached: when writing, the next key/value
	! pair will be written at the cursor. When reading, the next call
	! to \ref dict_read_next() will return the cursor.*/
	pub cursor:     *mut Tuple,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of DictionaryIterator"][::core::mem::size_of::<DictionaryIterator>() - 12usize];
	["Alignment of DictionaryIterator"][::core::mem::align_of::<DictionaryIterator>() - 4usize];
	["Offset of field: DictionaryIterator::dictionary"]
		[::core::mem::offset_of!(DictionaryIterator, dictionary) - 0usize];
	["Offset of field: DictionaryIterator::end"]
		[::core::mem::offset_of!(DictionaryIterator, end) - 4usize];
	["Offset of field: DictionaryIterator::cursor"]
		[::core::mem::offset_of!(DictionaryIterator, cursor) - 8usize];
};
/**! Calculates the number of bytes that a dictionary will occupy, given
! one or more value lengths that need to be stored in the dictionary.
! @note The formula to calculate the size of a Dictionary in bytes is:
! <pre>1 + (n * 7) + D1 + ... + Dn</pre>
! Where `n` is the number of Tuples in the Dictionary and `Dx` are the sizes
! of the values in the Tuples. The size of the Dictionary header is 1 byte.
! The size of the header for each Tuple is 7 bytes.
! @param tuple_count The total number of key/value pairs in the dictionary.
! @param ... The sizes of each of the values that need to be
! stored in the dictionary.
! @return The total number of bytes of storage needed.*/
/** dict_calc_buffer_size
  -> Added in 0
gneiss: Vtable Slot 74, Offset 296*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_calc_buffer_size")]
pub unsafe extern "C" fn dict_calc_buffer_size(tuple_count: u8, args: ...) -> u32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =296", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Calculates the size of data that has been written to the dictionary.
! AKA, the "dictionary size". Note that this is most likely different
! than the size of the backing storage/backing buffer.
! @param iter The dictionary iterator
! @return The total number of bytes which have been written to the dictionary.*/
/** dict_size
  -> Added in 6
gneiss: Vtable Slot 314, Offset 1256*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_size")]
pub unsafe extern "C" fn dict_size(iter: *mut DictionaryIterator) -> u32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1256", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Initializes the dictionary iterator with a given buffer and size,
! resets and empties it, in preparation of writing key/value tuples.
! @param iter The dictionary iterator
! @param buffer The storage of the dictionary
! @param size The storage size of the dictionary
! @return \ref DICT_OK, \ref DICT_NOT_ENOUGH_STORAGE or \ref DICT_INVALID_ARGS
! @see dict_calc_buffer_size
! @see dict_write_end*/
/** dict_write_begin
  -> Added in 0
gneiss: Vtable Slot 84, Offset 336*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_write_begin")]
pub unsafe extern "C" fn dict_write_begin(
	iter: *mut DictionaryIterator,
	buffer: *mut u8,
	size: u16,
) -> DictionaryResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =336", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Adds a key with a byte array value pair to the dictionary.
! @param iter The dictionary iterator
! @param key The key
! @param data Pointer to the byte array
! @param size Length of the byte array
! @return \ref DICT_OK, \ref DICT_NOT_ENOUGH_STORAGE or \ref DICT_INVALID_ARGS
! @note The data will be copied into the backing storage of the dictionary.
! @note There is _no_ checking for duplicate keys.*/
/** dict_write_data
  -> Added in 0
gneiss: Vtable Slot 86, Offset 344*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_write_data")]
pub unsafe extern "C" fn dict_write_data(
	iter: *mut DictionaryIterator,
	key: u32,
	data: *const u8,
	size: u16,
) -> DictionaryResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =344", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Adds a key with a C string value pair to the dictionary.
! @param iter The dictionary iterator
! @param key The key
! @param cstring Pointer to the zero-terminated C string
! @return \ref DICT_OK, \ref DICT_NOT_ENOUGH_STORAGE or \ref DICT_INVALID_ARGS
! @note The string will be copied into the backing storage of the dictionary.
! @note There is _no_ checking for duplicate keys.*/
/** dict_write_cstring
  -> Added in 0
gneiss: Vtable Slot 85, Offset 340*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_write_cstring")]
pub unsafe extern "C" fn dict_write_cstring(
	iter: *mut DictionaryIterator,
	key: u32,
	cstring: *const ::core::ffi::c_char,
) -> DictionaryResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =340", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Adds a key with an integer value pair to the dictionary.
! @param iter The dictionary iterator
! @param key The key
! @param integer Pointer to the integer value
! @param width_bytes The width of the integer value
! @param is_signed Whether the integer's type is signed or not
! @return \ref DICT_OK, \ref DICT_NOT_ENOUGH_STORAGE or \ref DICT_INVALID_ARGS
! @note There is _no_ checking for duplicate keys. dict_write_int() is only for serializing a single
! integer. width_bytes can only be 1, 2, or 4.*/
/** dict_write_int
  -> Added in 0
gneiss: Vtable Slot 88, Offset 352*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_write_int")]
pub unsafe extern "C" fn dict_write_int(
	iter: *mut DictionaryIterator,
	key: u32,
	integer: *const ::core::ffi::c_void,
	width_bytes: u8,
	is_signed: bool,
) -> DictionaryResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =352", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Adds a key with an unsigned, 8-bit integer value pair to the dictionary.
! @param iter The dictionary iterator
! @param key The key
! @param value The unsigned, 8-bit integer value
! @return \ref DICT_OK, \ref DICT_NOT_ENOUGH_STORAGE or \ref DICT_INVALID_ARGS
! @note There is _no_ checking for duplicate keys.
! @note There are counterpart functions for different signedness and widths,
! `dict_write_uint16()`, `dict_write_uint32()`, `dict_write_int8()`,
! `dict_write_int16()` and `dict_write_int32()`. The documentation is not
! repeated for brevity's sake.*/
/** dict_write_uint8
  -> Added in 0
gneiss: Vtable Slot 95, Offset 380*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_write_uint8")]
pub unsafe extern "C" fn dict_write_uint8(
	iter: *mut DictionaryIterator,
	key: u32,
	value: u8,
) -> DictionaryResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =380", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/** dict_write_uint16
  -> Added in 0
gneiss: Vtable Slot 93, Offset 372*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_write_uint16")]
pub unsafe extern "C" fn dict_write_uint16(
	iter: *mut DictionaryIterator,
	key: u32,
	value: u16,
) -> DictionaryResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =372", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/** dict_write_uint32
  -> Added in 0
gneiss: Vtable Slot 94, Offset 376*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_write_uint32")]
pub unsafe extern "C" fn dict_write_uint32(
	iter: *mut DictionaryIterator,
	key: u32,
	value: u32,
) -> DictionaryResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =376", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/** dict_write_int8
  -> Added in 0
gneiss: Vtable Slot 91, Offset 364*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_write_int8")]
pub unsafe extern "C" fn dict_write_int8(
	iter: *mut DictionaryIterator,
	key: u32,
	value: i8,
) -> DictionaryResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =364", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/** dict_write_int16
  -> Added in 0
gneiss: Vtable Slot 89, Offset 356*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_write_int16")]
pub unsafe extern "C" fn dict_write_int16(
	iter: *mut DictionaryIterator,
	key: u32,
	value: i16,
) -> DictionaryResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =356", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/** dict_write_int32
  -> Added in 0
gneiss: Vtable Slot 90, Offset 360*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_write_int32")]
pub unsafe extern "C" fn dict_write_int32(
	iter: *mut DictionaryIterator,
	key: u32,
	value: i32,
) -> DictionaryResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =360", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! End a series of writing operations to a dictionary.
! This must be called before reading back from the dictionary.
! @param iter The dictionary iterator
! @return The size in bytes of the finalized dictionary, or 0 if the parameters were invalid.*/
/** dict_write_end
  -> Added in 0
gneiss: Vtable Slot 87, Offset 348*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_write_end")]
pub unsafe extern "C" fn dict_write_end(iter: *mut DictionaryIterator) -> u32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =348", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Initializes the dictionary iterator with a given buffer and size,
! in preparation of reading key/value tuples.
! @param iter The dictionary iterator
! @param buffer The storage of the dictionary
! @param size The storage size of the dictionary
! @return The first tuple in the dictionary, or NULL in case the dictionary was empty or if there was a parsing error.*/
/** dict_read_begin_from_buffer
  -> Added in 0
gneiss: Vtable Slot 78, Offset 312*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_read_begin_from_buffer")]
pub unsafe extern "C" fn dict_read_begin_from_buffer(
	iter: *mut DictionaryIterator,
	buffer: *const u8,
	size: u16,
) -> *mut Tuple {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =312", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Progresses the iterator to the next key/value pair.
! @param iter The dictionary iterator
! @return The next tuple in the dictionary, or NULL in case the end has been reached or if there was a parsing error.*/
/** dict_read_next
  -> Added in 0
gneiss: Vtable Slot 80, Offset 320*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_read_next")]
pub unsafe extern "C" fn dict_read_next(iter: *mut DictionaryIterator) -> *mut Tuple {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =320", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Resets the iterator back to the same state as a call to \ref dict_read_begin_from_buffer() would do.
! @param iter The dictionary iterator
! @return The first tuple in the dictionary, or NULL in case the dictionary was empty or if there was a parsing error.*/
/** dict_read_first
  -> Added in 0
gneiss: Vtable Slot 79, Offset 316*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_read_first")]
pub unsafe extern "C" fn dict_read_first(iter: *mut DictionaryIterator) -> *mut Tuple {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =316", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Non-serialized, template data structure for a key/value pair.
! For strings and byte arrays, it only has a pointer to the actual data.
! For integers, it provides storage for integers up to 32-bits wide.
! The Tuplet data structure is useful when creating dictionaries from values
! that are already stored in arbitrary buffers.
! See also \ref Tuple, with is the header of a serialized key/value pair.*/
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Tuplet {
	/**! The type of the Tuplet. This determines which of the struct fields in the
	! anonymomous union are valid.*/
	pub type_:            TupleType,
	///! The key.
	pub key:              u32,
	pub __bindgen_anon_1: Tuplet__bindgen_ty_1,
}
/**! Anonymous union containing the reference to the Tuplet's value, being
! either a byte array, c-string or integer. See documentation of `.bytes`,
! `.cstring` and `.integer` fields.*/
#[repr(C)]
#[derive(Copy, Clone)]
pub union Tuplet__bindgen_ty_1 {
	pub bytes:   Tuplet__bindgen_ty_1__bindgen_ty_1,
	pub cstring: Tuplet__bindgen_ty_1__bindgen_ty_2,
	pub integer: Tuplet__bindgen_ty_1__bindgen_ty_3,
}
///! Valid when `.type.` is \ref TUPLE_BYTE_ARRAY
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Tuplet__bindgen_ty_1__bindgen_ty_1 {
	///! Pointer to the data
	pub data:   *const u8,
	///! Length of the data
	pub length: u16,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of Tuplet__bindgen_ty_1__bindgen_ty_1"]
		[::core::mem::size_of::<Tuplet__bindgen_ty_1__bindgen_ty_1>() - 8usize];
	["Alignment of Tuplet__bindgen_ty_1__bindgen_ty_1"]
		[::core::mem::align_of::<Tuplet__bindgen_ty_1__bindgen_ty_1>() - 4usize];
	["Offset of field: Tuplet__bindgen_ty_1__bindgen_ty_1::data"]
		[::core::mem::offset_of!(Tuplet__bindgen_ty_1__bindgen_ty_1, data) - 0usize];
	["Offset of field: Tuplet__bindgen_ty_1__bindgen_ty_1::length"]
		[::core::mem::offset_of!(Tuplet__bindgen_ty_1__bindgen_ty_1, length) - 4usize];
};
///! Valid when `.type.` is \ref TUPLE_CSTRING
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Tuplet__bindgen_ty_1__bindgen_ty_2 {
	///! Pointer to the c-string data
	pub data:   *const ::core::ffi::c_char,
	///! Length of the c-string, including terminating zero.
	pub length: u16,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of Tuplet__bindgen_ty_1__bindgen_ty_2"]
		[::core::mem::size_of::<Tuplet__bindgen_ty_1__bindgen_ty_2>() - 8usize];
	["Alignment of Tuplet__bindgen_ty_1__bindgen_ty_2"]
		[::core::mem::align_of::<Tuplet__bindgen_ty_1__bindgen_ty_2>() - 4usize];
	["Offset of field: Tuplet__bindgen_ty_1__bindgen_ty_2::data"]
		[::core::mem::offset_of!(Tuplet__bindgen_ty_1__bindgen_ty_2, data) - 0usize];
	["Offset of field: Tuplet__bindgen_ty_1__bindgen_ty_2::length"]
		[::core::mem::offset_of!(Tuplet__bindgen_ty_1__bindgen_ty_2, length) - 4usize];
};
///! Valid when `.type.` is \ref TUPLE_INT or \ref TUPLE_UINT
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Tuplet__bindgen_ty_1__bindgen_ty_3 {
	/**! Actual storage of the integer.
	! The signedness can be derived from the `.type` value.*/
	pub storage: u32,
	///! Width of the integer.
	pub width:   u16,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of Tuplet__bindgen_ty_1__bindgen_ty_3"]
		[::core::mem::size_of::<Tuplet__bindgen_ty_1__bindgen_ty_3>() - 8usize];
	["Alignment of Tuplet__bindgen_ty_1__bindgen_ty_3"]
		[::core::mem::align_of::<Tuplet__bindgen_ty_1__bindgen_ty_3>() - 4usize];
	["Offset of field: Tuplet__bindgen_ty_1__bindgen_ty_3::storage"]
		[::core::mem::offset_of!(Tuplet__bindgen_ty_1__bindgen_ty_3, storage) - 0usize];
	["Offset of field: Tuplet__bindgen_ty_1__bindgen_ty_3::width"]
		[::core::mem::offset_of!(Tuplet__bindgen_ty_1__bindgen_ty_3, width) - 4usize];
};
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of Tuplet__bindgen_ty_1"][::core::mem::size_of::<Tuplet__bindgen_ty_1>() - 8usize];
	["Alignment of Tuplet__bindgen_ty_1"][::core::mem::align_of::<Tuplet__bindgen_ty_1>() - 4usize];
	["Offset of field: Tuplet__bindgen_ty_1::bytes"]
		[::core::mem::offset_of!(Tuplet__bindgen_ty_1, bytes) - 0usize];
	["Offset of field: Tuplet__bindgen_ty_1::cstring"]
		[::core::mem::offset_of!(Tuplet__bindgen_ty_1, cstring) - 0usize];
	["Offset of field: Tuplet__bindgen_ty_1::integer"]
		[::core::mem::offset_of!(Tuplet__bindgen_ty_1, integer) - 0usize];
};
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of Tuplet"][::core::mem::size_of::<Tuplet>() - 16usize];
	["Alignment of Tuplet"][::core::mem::align_of::<Tuplet>() - 4usize];
	["Offset of field: Tuplet::type_"][::core::mem::offset_of!(Tuplet, type_) - 0usize];
	["Offset of field: Tuplet::key"][::core::mem::offset_of!(Tuplet, key) - 4usize];
};
/**! Callback for \ref dict_serialize_tuplets() utility.
! @param data The data of the serialized dictionary
! @param size The size of data
! @param context The context pointer as passed in to \ref dict_serialize_tuplets()
! @see dict_serialize_tuplets*/
pub type DictionarySerializeCallback = ::core::option::Option<
	unsafe extern "C" fn(data: *const u8, size: u16, context: *mut ::core::ffi::c_void),
>;
/**! Utility function that takes a list of Tuplets from which a dictionary
! will be serialized, ready to transmit or store.
! @note The callback will be called before the function returns, so the data that
! that `context` points to, can be stack allocated.
! @param callback The callback that will be called with the serialized data of the generated dictionary.
! @param context Pointer to any application specific data that gets passed into the callback.
! @param tuplets An array of Tuplets that need to be serialized into the dictionary.
! @param tuplets_count The number of tuplets that follow.
! @return \ref DICT_OK, \ref DICT_NOT_ENOUGH_STORAGE or \ref DICT_INVALID_ARGS*/
/** dict_serialize_tuplets
  -> Added in 0
gneiss: Vtable Slot 81, Offset 324*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_serialize_tuplets")]
pub unsafe extern "C" fn dict_serialize_tuplets(
	callback: DictionarySerializeCallback,
	context: *mut ::core::ffi::c_void,
	tuplets: *const Tuplet,
	tuplets_count: u8,
) -> DictionaryResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =324", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Utility function that takes an array of Tuplets and serializes them into
! a dictionary with a given buffer and size.
! @param tuplets The array of tuplets
! @param tuplets_count The number of tuplets in the array
! @param buffer The buffer in which to write the serialized dictionary
! @param [in,out] size_in_out In: the available buffer size in bytes. Out: the number of bytes written.
! @return \ref DICT_OK, \ref DICT_NOT_ENOUGH_STORAGE or \ref DICT_INVALID_ARGS*/
/** dict_serialize_tuplets_to_buffer
  -> Added in 5
gneiss: Vtable Slot 310, Offset 1240*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_serialize_tuplets_to_buffer")]
pub unsafe extern "C" fn dict_serialize_tuplets_to_buffer(
	tuplets: *const Tuplet,
	tuplets_count: u8,
	buffer: *mut u8,
	size_in_out: *mut u32,
) -> DictionaryResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1240", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Serializes an array of Tuplets into a dictionary with a given buffer and size.
! @param iter The dictionary iterator
! @param tuplets The array of tuplets
! @param tuplets_count The number of tuplets in the array
! @param buffer The buffer in which to write the serialized dictionary
! @param [in,out] size_in_out In: the available buffer size in bytes. Out: the number of bytes written.
! @return \ref DICT_OK, \ref DICT_NOT_ENOUGH_STORAGE or \ref DICT_INVALID_ARGS*/
/** dict_serialize_tuplets_to_buffer_with_iter
  -> Added in 0
gneiss: Vtable Slot 83, Offset 332*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_serialize_tuplets_to_buffer_with_iter")]
pub unsafe extern "C" fn dict_serialize_tuplets_to_buffer_with_iter(
	iter: *mut DictionaryIterator,
	tuplets: *const Tuplet,
	tuplets_count: u8,
	buffer: *mut u8,
	size_in_out: *mut u32,
) -> DictionaryResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =332", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Serializes a Tuplet and writes the resulting Tuple into a dictionary.
! @param iter The dictionary iterator
! @param tuplet The Tuplet describing the key/value pair to write
! @return \ref DICT_OK, \ref DICT_NOT_ENOUGH_STORAGE or \ref DICT_INVALID_ARGS*/
/** dict_write_tuplet
  -> Added in 0
gneiss: Vtable Slot 92, Offset 368*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_write_tuplet")]
pub unsafe extern "C" fn dict_write_tuplet(
	iter: *mut DictionaryIterator,
	tuplet: *const Tuplet,
) -> DictionaryResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =368", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Calculates the number of bytes that a dictionary will occupy, given
! one or more Tuplets that need to be stored in the dictionary.
! @note See \ref dict_calc_buffer_size() for the formula for the calculation.
! @param tuplets An array of Tuplets that need to be stored in the dictionary.
! @param tuplets_count The total number of Tuplets that follow.
! @return The total number of bytes of storage needed.
! @see Tuplet*/
/** dict_calc_buffer_size_from_tuplets
  -> Added in 0
gneiss: Vtable Slot 75, Offset 300*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_calc_buffer_size_from_tuplets")]
pub unsafe extern "C" fn dict_calc_buffer_size_from_tuplets(
	tuplets: *const Tuplet,
	tuplets_count: u8,
) -> u32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =300", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Type of the callback used in \ref dict_merge()
! @param key The key that is being updated.
! @param new_tuple The new tuple. The tuple points to the actual, updated destination dictionary or NULL_TUPLE
! in case there was an error (e.g. backing buffer was too small).
! Therefore the Tuple can be used after the callback returns, until the destination dictionary
! storage is free'd (by the application itself).
! @param old_tuple The values that will be replaced with `new_tuple`. The key, value and type will be
! equal to the previous tuple in the old destination dictionary, however the `old_tuple` points
! to a stack-allocated copy of the old data.
! @param context Pointer to application specific data
! The storage backing `old_tuple` can only be used during the callback and
! will no longer be valid after the callback returns.
! @see dict_merge*/
pub type DictionaryKeyUpdatedCallback = ::core::option::Option<
	unsafe extern "C" fn(
		key: u32,
		new_tuple: *const Tuple,
		old_tuple: *const Tuple,
		context: *mut ::core::ffi::c_void,
	),
>;
/**! Merges entries from another "source" dictionary into a "destination" dictionary.
! All Tuples from the source are written into the destination dictionary, while
! updating the exsting Tuples with matching keys.
! @param dest The destination dictionary to update
! @param [in,out] dest_max_size_in_out In: the maximum size of buffer backing `dest`. Out: the final size of the updated dictionary.
! @param source The source dictionary of which its Tuples will be used to update dest.
! @param update_existing_keys_only Specify True if only the existing keys in `dest` should be updated.
! @param key_callback The callback that will be called for each Tuple in the merged destination dictionary.
! @param context Pointer to app specific data that will get passed in when `update_key_callback` is called.
! @return \ref DICT_OK, \ref DICT_INVALID_ARGS, \ref DICT_NOT_ENOUGH_STORAGE*/
/** dict_merge
  -> Added in 0
gneiss: Vtable Slot 77, Offset 308*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_merge")]
pub unsafe extern "C" fn dict_merge(
	dest: *mut DictionaryIterator,
	dest_max_size_in_out: *mut u32,
	source: *mut DictionaryIterator,
	update_existing_keys_only: bool,
	key_callback: DictionaryKeyUpdatedCallback,
	context: *mut ::core::ffi::c_void,
) -> DictionaryResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =308", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Tries to find a Tuple with specified key in a dictionary
! @param iter Iterator to the dictionary to search in.
! @param key The key for which to find a Tuple
! @return Pointer to a found Tuple, or NULL if there was no Tuple with the specified key.*/
/** dict_find
  -> Added in 0
gneiss: Vtable Slot 76, Offset 304*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dict_find")]
pub unsafe extern "C" fn dict_find(iter: *const DictionaryIterator, key: u32) -> *mut Tuple {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =304", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @addtogroup Dictation
! @{*/
#[repr(C)]
#[derive(Debug)]
pub struct DictationSession {
	_unused: [u8; 0],
}
///! Transcription successful, with a valid result
pub const DictationSessionStatus_DictationSessionStatusSuccess: DictationSessionStatus = 0;
///! User rejected transcription and exited UI
pub const DictationSessionStatus_DictationSessionStatusFailureTranscriptionRejected:
	DictationSessionStatus = 1;
///! User exited UI after transcription error
pub const DictationSessionStatus_DictationSessionStatusFailureTranscriptionRejectedWithError:
	DictationSessionStatus = 2;
///! Too many errors occurred during transcription and the UI exited
pub const DictationSessionStatus_DictationSessionStatusFailureSystemAborted:
	DictationSessionStatus = 3;
///! No speech was detected and UI exited
pub const DictationSessionStatus_DictationSessionStatusFailureNoSpeechDetected:
	DictationSessionStatus = 4;
///! No BT or internet connection
pub const DictationSessionStatus_DictationSessionStatusFailureConnectivityError:
	DictationSessionStatus = 5;
///! Voice transcription disabled for this user
pub const DictationSessionStatus_DictationSessionStatusFailureDisabled: DictationSessionStatus = 6;
///! Voice transcription failed due to internal error
pub const DictationSessionStatus_DictationSessionStatusFailureInternalError:
	DictationSessionStatus = 7;
///! Cloud recognizer failed to transcribe speech (only possible if error dialogs disabled)
pub const DictationSessionStatus_DictationSessionStatusFailureRecognizerError:
	DictationSessionStatus = 8;
pub type DictationSessionStatus = ::core::ffi::c_uchar;
/**! Dictation status callback. Indicates success or failure of the dictation session and, if
! successful, passes the transcribed string to the user of the dictation session. The transcribed
! string will be freed after this call returns, so the string should be copied if it needs to be
! retained afterwards.
! @param session        dictation session from which the status was received
! @param status         dictation status
! @param transcription  transcribed string
! @param context        callback context specified when starting the session*/
pub type DictationSessionStatusCallback = ::core::option::Option<
	unsafe extern "C" fn(
		session: *mut DictationSession,
		status: DictationSessionStatus,
		transcription: *mut ::core::ffi::c_char,
		context: *mut ::core::ffi::c_void,
	),
>;
/**! Create a dictation session. The session object can be used more than once to get a
! transcription. When a transcription is received a buffer will be allocated to store the text in
! with a maximum size specified by @c buffer_size. When a transcription is accepted by the user
! or a failure of some sort occurs, the callback specified will be called with the status and the
! transcription if one was accepted.
! @param buffer_size       size of buffer to allocate for the transcription text; text will be
!                          truncated if it is longer than the maximum size specified; a size of 0
!                          will allow the session to allocate as much as it needs and text will
!                          not be truncated
! @param callback          dictation session status handler (must be valid)
! @param callback_context  context pointer for status handler
! @return handle to the dictation session or NULL if the phone app is not connected or does not
! support voice dictation, if this is called on a platform that doesn't support voice dictation,
! or if an internal error occurs.*/
/** dictation_session_create
  -> Added in 63
gneiss: Vtable Slot 550, Offset 2200*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dictation_session_create")]
pub unsafe extern "C" fn dictation_session_create(
	buffer_size: u32,
	callback: DictationSessionStatusCallback,
	callback_context: *mut ::core::ffi::c_void,
) -> *mut DictationSession {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2200", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Destroy the dictation session and free its memory. Will terminate a session in progress.
! @param session  dictation session to be destroyed*/
/** dictation_session_destroy
  -> Added in 63
gneiss: Vtable Slot 551, Offset 2204*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dictation_session_destroy")]
pub unsafe extern "C" fn dictation_session_destroy(session: *mut DictationSession) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2204", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Start the dictation session. The dictation UI will be shown. When the user accepts a
! transcription or exits the UI, or, when the confirmation dialog is disabled and a status is
! received, the status callback will be called. Can only be called when no session is in progress.
! The session can be restarted multiple times after the UI is exited or the session is stopped.
! @param session  dictation session to start or restart
! @return true if session was started, false if session is already in progress or is invalid.*/
/** dictation_session_start
  -> Added in 63
gneiss: Vtable Slot 553, Offset 2212*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dictation_session_start")]
pub unsafe extern "C" fn dictation_session_start(
	session: *mut DictationSession,
) -> DictationSessionStatus {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2212", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Stop the current dictation session. The UI will be hidden and no status callbacks will be
! received after the session is stopped.
! @param session  dictation session to stop
! @return true if session was stopped, false if session was not started or is invalid*/
/** dictation_session_stop
  -> Added in 63
gneiss: Vtable Slot 554, Offset 2216*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dictation_session_stop")]
pub unsafe extern "C" fn dictation_session_stop(
	session: *mut DictationSession,
) -> DictationSessionStatus {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2216", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Enable or disable user confirmation of transcribed text, which allows the user to accept or
! reject (and restart) the transcription. Must be called before the session is started.
! @param session      dictation session to modify
! @param is_enabled   set to true to enable user confirmation of transcriptions (default), false
! to disable*/
/** dictation_session_enable_confirmation
  -> Added in 63
gneiss: Vtable Slot 552, Offset 2208*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dictation_session_enable_confirmation")]
pub unsafe extern "C" fn dictation_session_enable_confirmation(
	session: *mut DictationSession,
	is_enabled: bool,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2208", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Enable or disable error dialogs when transcription fails. Must be called before the session
! is started. Disabling error dialogs will also disable automatic retries if transcription fails.
! @param session      dictation session to modify
! @param is_enabled   set to true to enable error dialogs (default), false to disable*/
/** dictation_session_enable_error_dialogs
  -> Added in 69
gneiss: Vtable Slot 570, Offset 2280*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.dictation_session_enable_error_dialogs")]
pub unsafe extern "C" fn dictation_session_enable_error_dialogs(
	session: *mut DictationSession,
	is_enabled: bool,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2280", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! (0) All good, operation was successful.
pub const AppMessageResult_APP_MSG_OK: AppMessageResult = 0;
///! (2) The other end did not confirm receiving the sent data with an (n)ack in time.
pub const AppMessageResult_APP_MSG_SEND_TIMEOUT: AppMessageResult = 2;
///! (4) The other end rejected the sent data, with a "nack" reply.
pub const AppMessageResult_APP_MSG_SEND_REJECTED: AppMessageResult = 4;
///! (8) The other end was not connected.
pub const AppMessageResult_APP_MSG_NOT_CONNECTED: AppMessageResult = 8;
///! (16) The local application was not running.
pub const AppMessageResult_APP_MSG_APP_NOT_RUNNING: AppMessageResult = 16;
///! (32) The function was called with invalid arguments.
pub const AppMessageResult_APP_MSG_INVALID_ARGS: AppMessageResult = 32;
/**! (64) There are pending (in or outbound) messages that need to be processed first before
! new ones can be received or sent.*/
pub const AppMessageResult_APP_MSG_BUSY: AppMessageResult = 64;
///! (128) The buffer was too small to contain the incoming message.
pub const AppMessageResult_APP_MSG_BUFFER_OVERFLOW: AppMessageResult = 128;
///! (512) The resource had already been released.
pub const AppMessageResult_APP_MSG_ALREADY_RELEASED: AppMessageResult = 512;
///! (1024) The callback was already registered.
pub const AppMessageResult_APP_MSG_CALLBACK_ALREADY_REGISTERED: AppMessageResult = 1024;
///! (2048) The callback could not be deregistered, because it had not been registered before.
pub const AppMessageResult_APP_MSG_CALLBACK_NOT_REGISTERED: AppMessageResult = 2048;
/**! (4096) The system did not have sufficient application memory to
! perform the requested operation.*/
pub const AppMessageResult_APP_MSG_OUT_OF_MEMORY: AppMessageResult = 4096;
///! (8192) App message was closed.
pub const AppMessageResult_APP_MSG_CLOSED: AppMessageResult = 8192;
///! (16384) An internal OS error prevented AppMessage from completing an operation.
pub const AppMessageResult_APP_MSG_INTERNAL_ERROR: AppMessageResult = 16384;
///! (32768) The function was called while App Message was not in the appropriate state.
pub const AppMessageResult_APP_MSG_INVALID_STATE: AppMessageResult = 32768;
///! AppMessage result codes.
pub type AppMessageResult = ::core::ffi::c_ushort;
/**! Open AppMessage to transfers.
!
! Use \ref dict_calc_buffer_size_from_tuplets() or \ref dict_calc_buffer_size() to estimate the size you need.
!
! \param[in] size_inbound The required size for the Inbox buffer
! \param[in] size_outbound The required size for the Outbox buffer
!
! \return A result code such as \ref APP_MSG_OK or \ref APP_MSG_OUT_OF_MEMORY.
!
! \note It is recommended that if the Inbox will be used, that at least the Inbox callbacks should be registered
!   before this call.  Otherwise it is possible for an Inbox message to be NACK'ed without being seen by the
!   application.
!*/
/** app_message_open
  -> Added in 0
gneiss: Vtable Slot 36, Offset 144*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_message_open")]
pub unsafe extern "C" fn app_message_open(
	size_inbound: u32,
	size_outbound: u32,
) -> AppMessageResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =144", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Deregisters all callbacks and their context.
!*/
/** app_message_deregister_callbacks
  -> Added in 0
gneiss: Vtable Slot 35, Offset 140*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_message_deregister_callbacks")]
pub unsafe extern "C" fn app_message_deregister_callbacks() {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =140", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Called after an incoming message is received.
!
! \param[in] iterator
!   The dictionary iterator to the received message.  Never NULL.  Note that the iterator cannot be modified or
!   saved off.  The library may need to re-use the buffered space where this message is supplied.  Returning from
!   the callback indicates to the library that the received message contents are no longer needed or have already
!   been externalized outside its buffering space and iterator.
!
! \param[in] context
!   Pointer to application data as specified when registering the callback.
!*/
pub type AppMessageInboxReceived = ::core::option::Option<
	unsafe extern "C" fn(iterator: *mut DictionaryIterator, context: *mut ::core::ffi::c_void),
>;
/**! Called after an incoming message is dropped.
!
! \param[in] reason
!   The reason why the message was dropped.  Some possibilities include \ref APP_MSG_BUSY and
!   \ref APP_MSG_BUFFER_OVERFLOW.
!
! \param[in] context
!   Pointer to application data as specified when registering the callback.
!
! Note that you can call app_message_outbox_begin() from this handler to prepare a new message.
! This will invalidate the previous dictionary iterator; do not use it after calling app_message_outbox_begin().
!*/
pub type AppMessageInboxDropped = ::core::option::Option<
	unsafe extern "C" fn(reason: AppMessageResult, context: *mut ::core::ffi::c_void),
>;
/**! Called after an outbound message has been sent and the reply has been received.
!
! \param[in] iterator
!   The dictionary iterator to the sent message.  The iterator will be in the final state that was sent.  Note that
!   the iterator cannot be modified or saved off as the library will re-open the dictionary with dict_begin() after
!   this callback returns.
!
! \param[in] context
!   Pointer to application data as specified when registering the callback.
!*/
pub type AppMessageOutboxSent = ::core::option::Option<
	unsafe extern "C" fn(iterator: *mut DictionaryIterator, context: *mut ::core::ffi::c_void),
>;
/**! Called after an outbound message has not been sent successfully.
!
! \param[in] iterator
!   The dictionary iterator to the sent message.  The iterator will be in the final state that was sent.  Note that
!   the iterator cannot be modified or saved off as the library will re-open the dictionary with dict_begin() after
!   this callback returns.
!
! \param[in] reason
!   The result of the operation.  Some possibilities for the value include \ref APP_MSG_SEND_TIMEOUT,
!   \ref APP_MSG_SEND_REJECTED, \ref APP_MSG_NOT_CONNECTED, \ref APP_MSG_APP_NOT_RUNNING, and the combination
!   `(APP_MSG_NOT_CONNECTED | APP_MSG_APP_NOT_RUNNING)`.
!
! \param context
!   Pointer to application data as specified when registering the callback.
!
! Note that you can call app_message_outbox_begin() from this handler to prepare a new message.
! This will invalidate the previous dictionary iterator; do not use it after calling app_message_outbox_begin().
!*/
pub type AppMessageOutboxFailed = ::core::option::Option<
	unsafe extern "C" fn(
		iterator: *mut DictionaryIterator,
		reason: AppMessageResult,
		context: *mut ::core::ffi::c_void,
	),
>;
/**! Gets the context that will be passed to all AppMessage callbacks.
!
! \return The current context on record.
!*/
/** app_message_get_context
  -> Added in 3
gneiss: Vtable Slot 293, Offset 1172*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_message_get_context")]
pub unsafe extern "C" fn app_message_get_context() -> *mut ::core::ffi::c_void {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1172", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the context that will be passed to all AppMessage callbacks.
!
! \param[in] context The context that will be passed to all AppMessage callbacks.
!
! \return The previous context that was on record.
!*/
/** app_message_set_context
  -> Added in 3
gneiss: Vtable Slot 302, Offset 1208*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_message_set_context")]
pub unsafe extern "C" fn app_message_set_context(
	context: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_void {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1208", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Registers a function that will be called after any Inbox message is received successfully.
!
! Only one callback may be registered at a time.  Each subsequent call to this function will replace the previous
! callback.  The callback is optional; setting it to NULL will deregister the current callback and no function will
! be called anymore.
!
! \param[in] received_callback The callback that will be called going forward; NULL to not have a callback.
!
! \return The previous callback (or NULL) that was on record.
!*/
/** app_message_register_inbox_received
  -> Added in 3
gneiss: Vtable Slot 299, Offset 1196*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_message_register_inbox_received")]
pub unsafe extern "C" fn app_message_register_inbox_received(
	received_callback: AppMessageInboxReceived,
) -> AppMessageInboxReceived {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1196", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Registers a function that will be called after any Inbox message is received but dropped by the system.
!
! Only one callback may be registered at a time.  Each subsequent call to this function will replace the previous
! callback.  The callback is optional; setting it to NULL will deregister the current callback and no function will
! be called anymore.
!
! \param[in] dropped_callback The callback that will be called going forward; NULL to not have a callback.
!
! \return The previous callback (or NULL) that was on record.
!*/
/** app_message_register_inbox_dropped
  -> Added in 3
gneiss: Vtable Slot 298, Offset 1192*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_message_register_inbox_dropped")]
pub unsafe extern "C" fn app_message_register_inbox_dropped(
	dropped_callback: AppMessageInboxDropped,
) -> AppMessageInboxDropped {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1192", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Registers a function that will be called after any Outbox message is sent and an ACK reply occurs in a timely
! fashion.
!
! Only one callback may be registered at a time.  Each subsequent call to this function will replace the previous
! callback.  The callback is optional; setting it to NULL will deregister the current callback and no function will
! be called anymore.
!
! \param[in] sent_callback The callback that will be called going forward; NULL to not have a callback.
!
! \return The previous callback (or NULL) that was on record.
!*/
/** app_message_register_outbox_sent
  -> Added in 3
gneiss: Vtable Slot 301, Offset 1204*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_message_register_outbox_sent")]
pub unsafe extern "C" fn app_message_register_outbox_sent(
	sent_callback: AppMessageOutboxSent,
) -> AppMessageOutboxSent {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1204", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Registers a function that will be called after any Outbox message is not sent with a timely ACK reply.
! The call to \ref app_message_outbox_send() must have succeeded.
!
! Only one callback may be registered at a time.  Each subsequent call to this function will replace the previous
! callback.  The callback is optional; setting it to NULL will deregister the current callback and no function will
! be called anymore.
!
! \param[in] failed_callback The callback that will be called going forward; NULL to not have a callback.
!
! \return The previous callback (or NULL) that was on record.
!*/
/** app_message_register_outbox_failed
  -> Added in 3
gneiss: Vtable Slot 300, Offset 1200*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_message_register_outbox_failed")]
pub unsafe extern "C" fn app_message_register_outbox_failed(
	failed_callback: AppMessageOutboxFailed,
) -> AppMessageOutboxFailed {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1200", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Programatically determine the inbox size maximum in the current configuration.
!
! \return The inbox size maximum on this firmware.
!
! \sa APP_MESSAGE_INBOX_SIZE_MINIMUM
! \sa app_message_outbox_size_maximum()
!*/
/** app_message_inbox_size_maximum
  -> Added in 3
gneiss: Vtable Slot 294, Offset 1176*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_message_inbox_size_maximum")]
pub unsafe extern "C" fn app_message_inbox_size_maximum() -> u32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1176", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Programatically determine the outbox size maximum in the current configuration.
!
! \return The outbox size maximum on this firmware.
!
! \sa APP_MESSAGE_OUTBOX_SIZE_MINIMUM
! \sa app_message_inbox_size_maximum()
!*/
/** app_message_outbox_size_maximum
  -> Added in 3
gneiss: Vtable Slot 297, Offset 1188*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_message_outbox_size_maximum")]
pub unsafe extern "C" fn app_message_outbox_size_maximum() -> u32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1188", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Begin writing to the Outbox's Dictionary buffer.
!
! \param[out] iterator Location to write the DictionaryIterator pointer.  This will be NULL on failure.
!
! \return A result code, including but not limited to \ref APP_MSG_OK, \ref APP_MSG_INVALID_ARGS or
!   \ref APP_MSG_BUSY.
!
! \note After a successful call, one can add values to the dictionary using functions like \ref dict_write_data()
!   and friends.
!
! \sa Dictionary
!*/
/** app_message_outbox_begin
  -> Added in 3
gneiss: Vtable Slot 295, Offset 1180*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_message_outbox_begin")]
pub unsafe extern "C" fn app_message_outbox_begin(
	iterator: *mut *mut DictionaryIterator,
) -> AppMessageResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1180", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sends the outbound dictionary.
!
! \return A result code, including but not limited to \ref APP_MSG_OK or \ref APP_MSG_BUSY.  The APP_MSG_OK code does
!         not mean that the message was sent successfully, but only that the start of processing was successful.
!         Since this call is asynchronous, callbacks provide the final result instead.
!
! \sa AppMessageOutboxSent
! \sa AppMessageOutboxFailed
!*/
/** app_message_outbox_send
  -> Added in 3
gneiss: Vtable Slot 296, Offset 1184*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_message_outbox_send")]
pub unsafe extern "C" fn app_message_outbox_send() -> AppMessageResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1184", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Called whenever a Tuple changes. This does not necessarily mean the value in
! the Tuple has changed. When the internal "current" dictionary gets updated,
! existing Tuples might get shuffled around in the backing buffer, even though
! the values stay the same. In this callback, the client code gets the chance
! to remove the old reference and start using the new one.
! In this callback, your application MUST clean up any references to the
! `old_tuple` of a PREVIOUS call to this callback (and replace it with the
! `new_tuple` that is passed in with the current call).
! @param key The key for which the Tuple was changed.
! @param new_tuple The new tuple. The tuple points to the actual, updated
! "current" dictionary, as backed by the buffer internal to the AppSync
! struct. Therefore the Tuple can be used after the callback returns, until
! the AppSync is deinited. In case there was an error (e.g. storage shortage),
! this `new_tuple` can be `NULL_TUPLE`.
! @param old_tuple The values that will be replaced with `new_tuple`. The key,
! value and type will be equal to the previous tuple in the old destination
! dictionary; however, the `old_tuple` points to a stack-allocated copy of the
! old data. This value will be `NULL_TUPLE` when the initial values are
! being set.
! @param context Pointer to application specific data, as set using
! \ref app_sync_init()
! @see \ref app_sync_init()*/
pub type AppSyncTupleChangedCallback = ::core::option::Option<
	unsafe extern "C" fn(
		key: u32,
		new_tuple: *const Tuple,
		old_tuple: *const Tuple,
		context: *mut ::core::ffi::c_void,
	),
>;
/**! Called whenever there was an error.
! @param dict_error The dictionary result error code, if the error was
! dictionary related.
! @param app_message_error The app_message result error code, if the error
! was app_message related.
! @param context Pointer to application specific data, as set using
! \ref app_sync_init()
! @see \ref app_sync_init()*/
pub type AppSyncErrorCallback = ::core::option::Option<
	unsafe extern "C" fn(
		dict_error: DictionaryResult,
		app_message_error: AppMessageResult,
		context: *mut ::core::ffi::c_void,
	),
>;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct AppSync {
	pub current_iter:     DictionaryIterator,
	pub __bindgen_anon_1: AppSync__bindgen_ty_1,
	pub buffer_size:      u16,
	pub callback:         AppSync__bindgen_ty_2,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union AppSync__bindgen_ty_1 {
	pub current: *mut Dictionary,
	pub buffer:  *mut u8,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of AppSync__bindgen_ty_1"][::core::mem::size_of::<AppSync__bindgen_ty_1>() - 4usize];
	["Alignment of AppSync__bindgen_ty_1"]
		[::core::mem::align_of::<AppSync__bindgen_ty_1>() - 4usize];
	["Offset of field: AppSync__bindgen_ty_1::current"]
		[::core::mem::offset_of!(AppSync__bindgen_ty_1, current) - 0usize];
	["Offset of field: AppSync__bindgen_ty_1::buffer"]
		[::core::mem::offset_of!(AppSync__bindgen_ty_1, buffer) - 0usize];
};
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct AppSync__bindgen_ty_2 {
	pub value_changed: AppSyncTupleChangedCallback,
	pub error:         AppSyncErrorCallback,
	pub context:       *mut ::core::ffi::c_void,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of AppSync__bindgen_ty_2"][::core::mem::size_of::<AppSync__bindgen_ty_2>() - 12usize];
	["Alignment of AppSync__bindgen_ty_2"]
		[::core::mem::align_of::<AppSync__bindgen_ty_2>() - 4usize];
	["Offset of field: AppSync__bindgen_ty_2::value_changed"]
		[::core::mem::offset_of!(AppSync__bindgen_ty_2, value_changed) - 0usize];
	["Offset of field: AppSync__bindgen_ty_2::error"]
		[::core::mem::offset_of!(AppSync__bindgen_ty_2, error) - 4usize];
	["Offset of field: AppSync__bindgen_ty_2::context"]
		[::core::mem::offset_of!(AppSync__bindgen_ty_2, context) - 8usize];
};
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of AppSync"][::core::mem::size_of::<AppSync>() - 32usize];
	["Alignment of AppSync"][::core::mem::align_of::<AppSync>() - 4usize];
	["Offset of field: AppSync::current_iter"]
		[::core::mem::offset_of!(AppSync, current_iter) - 0usize];
	["Offset of field: AppSync::buffer_size"]
		[::core::mem::offset_of!(AppSync, buffer_size) - 16usize];
	["Offset of field: AppSync::callback"][::core::mem::offset_of!(AppSync, callback) - 20usize];
};
/**! Initialized an AppSync system with specific buffer size and initial keys and
! values. The `callback.value_changed` callback will be called
! __asynchronously__ with the initial keys and values, as to avoid duplicating
! code to update your app's UI.
! @param s The AppSync context to initialize
! @param buffer The buffer that AppSync should use
! @param buffer_size The size of the backing storage of the "current"
! dictionary. Use \ref dict_calc_buffer_size_from_tuplets() to estimate the
! size you need.
! @param keys_and_initial_values An array of Tuplets with the initial keys and
! values.
! @param count The number of Tuplets in the `keys_and_initial_values` array.
! @param tuple_changed_callback The callback that will handle changed
! key/value pairs
! @param error_callback The callback that will handle errors
! @param context Pointer to app specific data that will get passed into calls
! to the callbacks
! @note Only updates for the keys specified in this initial array will be
! accepted by AppSync, updates for other keys that might come in will just be
! ignored.*/
/** app_sync_init
  -> Added in 0
gneiss: Vtable Slot 45, Offset 180*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_sync_init")]
pub unsafe extern "C" fn app_sync_init(
	s: *mut AppSync,
	buffer: *mut u8,
	buffer_size: u16,
	keys_and_initial_values: *const Tuplet,
	count: u8,
	tuple_changed_callback: AppSyncTupleChangedCallback,
	error_callback: AppSyncErrorCallback,
	context: *mut ::core::ffi::c_void,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =180", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Cleans up an AppSync system.
! It frees the buffer allocated by an \ref app_sync_init() call and
! deregisters itself from the \ref AppMessage subsystem.
! @param s The AppSync context to deinit.*/
/** app_sync_deinit
  -> Added in 0
gneiss: Vtable Slot 43, Offset 172*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_sync_deinit")]
pub unsafe extern "C" fn app_sync_deinit(s: *mut AppSync) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =172", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Updates key/value pairs using an array of Tuplets.
! @note The call will attempt to send the updated keys and values to the
! application on the other end.
! Only after the other end has acknowledged the update, the `.value_changed`
! callback will be called to confirm the update has completed and your
! application code can update its user interface.
! @param s The AppSync context
! @param keys_and_values_to_update An array of Tuplets with the keys and
! values to update. The data in the Tuplets are copied during the call, so the
! array can be stack-allocated.
! @param count The number of Tuplets in the `keys_and_values_to_update` array.
! @return The result code from the \ref AppMessage subsystem.
! Can be \ref APP_MSG_OK, \ref APP_MSG_BUSY or \ref APP_MSG_INVALID_ARGS*/
/** app_sync_set
  -> Added in 0
gneiss: Vtable Slot 46, Offset 184*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_sync_set")]
pub unsafe extern "C" fn app_sync_set(
	s: *mut AppSync,
	keys_and_values_to_update: *const Tuplet,
	count: u8,
) -> AppMessageResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =184", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Finds and gets a tuple in the "current" dictionary.
! @param s The AppSync context
! @param key The key for which to find a Tuple
! @return Pointer to a found Tuple, or NULL if there was no Tuple with the
! specified key.*/
/** app_sync_get
  -> Added in 0
gneiss: Vtable Slot 44, Offset 176*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_sync_get")]
pub unsafe extern "C" fn app_sync_get(s: *const AppSync, key: u32) -> *const Tuple {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =176", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Opaque reference to a resource.
! @see @c resource_get_handle()*/
pub type ResHandle = *mut ::core::ffi::c_void;
/**! Gets the resource handle for a file identifier.
! @param resource_id The resource ID
!
! The resource IDs are auto-generated by the Pebble build process, based
! on the `appinfo.json`. The "name" field of each resource is prefixed
! by `RESOURCE_ID_` and made visible to the application (through the
! `build/src/resource_ids.auto.h` header which is automatically included).
!
! For example, given the following fragment of `appinfo.json`:
! \code{.json}
!   ...
!   "resources" : {
!     "media": [
!        {
!           "name": "MY_ICON",
!           "file": "img/icon.png",
!           "type": "png",
!        },
!    ...
! \endcode
! The generated file identifier for this resource is `RESOURCE_ID_MY_ICON`.
! To get a resource handle for that resource write:
! \code{.c}
! ResHandle rh = resource_get_handle(RESOURCE_ID_MY_ICON);
! \endcode*/
/** resource_get_handle
  -> Added in 0
gneiss: Vtable Slot 206, Offset 824*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.resource_get_handle")]
pub unsafe extern "C" fn resource_get_handle(resource_id: u32) -> ResHandle {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =824", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the size of the resource given a resource handle.
! @param h The handle to the resource
! @return The size of the resource in bytes*/
/** resource_size
  -> Added in 0
gneiss: Vtable Slot 209, Offset 836*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.resource_size")]
pub unsafe extern "C" fn resource_size(h: ResHandle) -> usize {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =836", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Copies the bytes for the resource with a given handle from flash storage into a given buffer.
! @param h The handle to the resource
! @param buffer The buffer to load the resource data into
! @param max_length The maximum number of bytes to copy
! @return The number of bytes actually copied*/
/** resource_load
  -> Added in 0
gneiss: Vtable Slot 207, Offset 828*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.resource_load")]
pub unsafe extern "C" fn resource_load(h: ResHandle, buffer: *mut u8, max_length: usize) -> usize {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =828", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Copies a range of bytes from a resource with a given handle into a given buffer.
! @param h The handle to the resource
! @param start_offset The offset in bytes at which to start reading from the resource
! @param buffer The buffer to load the resource data into
! @param num_bytes The maximum number of bytes to copy
! @return The number of bytes actually copied*/
/** resource_load_byte_range
  -> Added in 0
gneiss: Vtable Slot 208, Offset 832*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.resource_load_byte_range")]
pub unsafe extern "C" fn resource_load_byte_range(
	h: ResHandle,
	start_offset: u32,
	buffer: *mut u8,
	num_bytes: usize,
) -> usize {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =832", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! The event loop for C apps, to be used in app's main().
! Will block until the app is ready to exit.*/
/** app_event_loop
  -> Added in 0
gneiss: Vtable Slot 31, Offset 124*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_event_loop")]
pub unsafe extern "C" fn app_event_loop() {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =124", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Configuration record for creating a Moddable XS virtual machine.
! Used with moddable_createMachine() to customize the JS runtime.
! Set recordSize to sizeof(ModdableCreationRecord) for version compatibility.*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ModdableCreationRecord {
	///!< Size of this struct in bytes (for versioning)
	pub recordSize: u32,
	///!< Stack size in bytes (0 for default)
	pub stack:      u32,
	///!< Slot heap size in bytes (0 for default)
	pub slot:       u32,
	///!< Chunk heap size in bytes (0 for default)
	pub chunk:      u32,
	///!< Combination of kModdableCreationFlag* values
	pub flags:      u32,
	///!< Optional pointer to an fxBuildFFI function for custom native bindings
	pub fxBuildFFI: *mut ::core::ffi::c_void,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of ModdableCreationRecord"][::core::mem::size_of::<ModdableCreationRecord>() - 24usize];
	["Alignment of ModdableCreationRecord"]
		[::core::mem::align_of::<ModdableCreationRecord>() - 4usize];
	["Offset of field: ModdableCreationRecord::recordSize"]
		[::core::mem::offset_of!(ModdableCreationRecord, recordSize) - 0usize];
	["Offset of field: ModdableCreationRecord::stack"]
		[::core::mem::offset_of!(ModdableCreationRecord, stack) - 4usize];
	["Offset of field: ModdableCreationRecord::slot"]
		[::core::mem::offset_of!(ModdableCreationRecord, slot) - 8usize];
	["Offset of field: ModdableCreationRecord::chunk"]
		[::core::mem::offset_of!(ModdableCreationRecord, chunk) - 12usize];
	["Offset of field: ModdableCreationRecord::flags"]
		[::core::mem::offset_of!(ModdableCreationRecord, flags) - 16usize];
	["Offset of field: ModdableCreationRecord::fxBuildFFI"]
		[::core::mem::offset_of!(ModdableCreationRecord, fxBuildFFI) - 20usize];
};
///! Success
pub const AppWorkerResult_APP_WORKER_RESULT_SUCCESS: AppWorkerResult = 0;
///! No worker found for the current app
pub const AppWorkerResult_APP_WORKER_RESULT_NO_WORKER: AppWorkerResult = 1;
///! A worker for a different app is already running
pub const AppWorkerResult_APP_WORKER_RESULT_DIFFERENT_APP: AppWorkerResult = 2;
///! The worker is not running
pub const AppWorkerResult_APP_WORKER_RESULT_NOT_RUNNING: AppWorkerResult = 3;
///! The worker is already running
pub const AppWorkerResult_APP_WORKER_RESULT_ALREADY_RUNNING: AppWorkerResult = 4;
///! The user will be asked for confirmation
pub const AppWorkerResult_APP_WORKER_RESULT_ASKING_CONFIRMATION: AppWorkerResult = 5;
///! Possible error codes from app_worker_launch, app_worker_kill
pub type AppWorkerResult = ::core::ffi::c_uchar;
///! Generic structure of a worker message that can be sent between an app and its worker
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct AppWorkerMessage {
	pub data0: u16,
	pub data1: u16,
	pub data2: u16,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of AppWorkerMessage"][::core::mem::size_of::<AppWorkerMessage>() - 6usize];
	["Alignment of AppWorkerMessage"][::core::mem::align_of::<AppWorkerMessage>() - 2usize];
	["Offset of field: AppWorkerMessage::data0"]
		[::core::mem::offset_of!(AppWorkerMessage, data0) - 0usize];
	["Offset of field: AppWorkerMessage::data1"]
		[::core::mem::offset_of!(AppWorkerMessage, data1) - 2usize];
	["Offset of field: AppWorkerMessage::data2"]
		[::core::mem::offset_of!(AppWorkerMessage, data2) - 4usize];
};
/**! Determine if the worker for the current app is running
! @return true if running*/
/** app_worker_is_running
  -> Added in 16
gneiss: Vtable Slot 327, Offset 1308*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_worker_is_running")]
pub unsafe extern "C" fn app_worker_is_running() -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1308", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Launch the worker for the current app. Note that this is an asynchronous operation, a result code
! of APP_WORKER_RESULT_SUCCESS merely means that the request was successfully queued up.
! @return result code*/
/** app_worker_launch
  -> Added in 16
gneiss: Vtable Slot 329, Offset 1316*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_worker_launch")]
pub unsafe extern "C" fn app_worker_launch() -> AppWorkerResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1316", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Kill the worker for the current app. Note that this is an asynchronous operation, a result code
! of APP_WORKER_RESULT_SUCCESS merely means that the request was successfully queued up.
! @return result code*/
/** app_worker_kill
  -> Added in 16
gneiss: Vtable Slot 328, Offset 1312*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_worker_kill")]
pub unsafe extern "C" fn app_worker_kill() -> AppWorkerResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1312", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Callback type for worker messages. Messages can be sent from worker to app or vice versa.
! @param type An application defined message type
! @param data pointer to message data. The receiver must know the structure of the data provided by the sender.*/
pub type AppWorkerMessageHandler =
	::core::option::Option<unsafe extern "C" fn(type_: u16, data: *mut AppWorkerMessage)>;
/**! Subscribe to worker messages. Once subscribed, the handler gets called on every message emitted by the other task
! (either worker or app).
! @param handler A callback to be executed when the event is received
! @return true on success*/
/** app_worker_message_subscribe
  -> Added in 16
gneiss: Vtable Slot 330, Offset 1320*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_worker_message_subscribe")]
pub unsafe extern "C" fn app_worker_message_subscribe(handler: AppWorkerMessageHandler) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1320", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Unsubscribe from worker messages. Once unsubscribed, the previously registered handler will no longer be called.
! @return true on success*/
/** app_worker_message_unsubscribe
  -> Added in 16
gneiss: Vtable Slot 331, Offset 1324*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_worker_message_unsubscribe")]
pub unsafe extern "C" fn app_worker_message_unsubscribe() -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1324", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Send a message to the other task (either worker or app).
! @param type An application defined message type
! @param data the message data structure*/
/** app_worker_send_message
  -> Added in 16
gneiss: Vtable Slot 332, Offset 1328*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_worker_send_message")]
pub unsafe extern "C" fn app_worker_send_message(type_: u8, data: *mut AppWorkerMessage) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1328", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Set the sniff interval to normal (power-saving) mode
pub const SniffInterval_SNIFF_INTERVAL_NORMAL: SniffInterval = 0;
/**! Reduce the sniff interval to increase the responsiveness of the radio at
! the expense of increasing Bluetooth energy consumption by a multiple of 2-5
! (very significant)*/
pub const SniffInterval_SNIFF_INTERVAL_REDUCED: SniffInterval = 1;
/**! Intervals during which the Bluetooth module may enter a low power mode.
! The sniff interval defines the period during which the Bluetooth module may
! not exchange (ACL) packets. The longer the sniff interval, the more time the
! Bluetooth module may spend in a low power mode.
! It may be necessary to reduce the sniff interval if an app requires reduced
! latency when sending messages.
! @note These settings have a dramatic effect on the Pebble's energy
! consumption. Use the normal sniff interval whenever possible.
! Note, however, that switching between modes increases power consumption
! during the process. Frequent switching between modes is thus
! discouraged. Ensure you do not drop to normal frequently. The Bluetooth module
! is a major consumer of the Pebble's energy.*/
pub type SniffInterval = ::core::ffi::c_uchar;
/**! Set the Bluetooth module's sniff interval.
! The sniff interval will be restored to normal by the OS after the app's
! de-init handler is called. Set the sniff interval to normal whenever
! possible.*/
/** app_comm_set_sniff_interval
  -> Added in 0
gneiss: Vtable Slot 30, Offset 120*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_comm_set_sniff_interval")]
pub unsafe extern "C" fn app_comm_set_sniff_interval(interval: SniffInterval) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =120", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the Bluetooth module's sniff interval
! @return The SniffInterval value corresponding to the current interval*/
/** app_comm_get_sniff_interval
  -> Added in 0
gneiss: Vtable Slot 29, Offset 116*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_comm_get_sniff_interval")]
pub unsafe extern "C" fn app_comm_get_sniff_interval() -> SniffInterval {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =116", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Waits for a certain amount of milliseconds
! @param millis The number of milliseconds to wait for*/
/** psleep
  -> Added in 0
gneiss: Vtable Slot 204, Offset 816*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.psleep")]
pub unsafe extern "C" fn psleep(millis: ::core::ffi::c_int) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =816", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
#[repr(C)]
#[derive(Debug)]
pub struct AppTimer {
	_unused: [u8; 0],
}
/**! The type of function which can be called when a timer fires.  The argument will be the @p callback_data passed to
! @ref app_timer_register().*/
pub type AppTimerCallback =
	::core::option::Option<unsafe extern "C" fn(data: *mut ::core::ffi::c_void)>;
/**! Registers a timer that ends up in callback being called some specified time in the future.
! @param timeout_ms The expiry time in milliseconds from the current time
! @param callback The callback that gets called at expiry time
! @param callback_data The data that will be passed to callback
! @return A pointer to an `AppTimer` that can be used to later reschedule or cancel this timer*/
/** app_timer_register
  -> Added in 0
gneiss: Vtable Slot 48, Offset 192*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_timer_register")]
pub unsafe extern "C" fn app_timer_register(
	timeout_ms: u32,
	callback: AppTimerCallback,
	callback_data: *mut ::core::ffi::c_void,
) -> *mut AppTimer {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =192", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Reschedules an already running timer for some point in the future.
! Elapsed timers cannot be rescheduled.
! @param timer_handle The timer to reschedule
! @param new_timeout_ms The new expiry time in milliseconds from the current time
! @return true if the timer was rescheduled, false if the timer has already elapsed*/
/** app_timer_reschedule
  -> Added in 0
gneiss: Vtable Slot 49, Offset 196*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_timer_reschedule")]
pub unsafe extern "C" fn app_timer_reschedule(
	timer_handle: *mut AppTimer,
	new_timeout_ms: u32,
) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =196", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Cancels an already registered timer.
! Once cancelled the handle may no longer be used for any purpose.
! Elapsed timers do not need to be cancelled.*/
/** app_timer_cancel
  -> Added in 0
gneiss: Vtable Slot 47, Offset 188*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_timer_cancel")]
pub unsafe extern "C" fn app_timer_cancel(timer_handle: *mut AppTimer) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =188", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Calculates the number of bytes of heap memory \a not currently being used by the application.
! @return The number of bytes on the heap not currently being used.*/
/** heap_bytes_free
  -> Added in 17
gneiss: Vtable Slot 335, Offset 1340*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.heap_bytes_free")]
pub unsafe extern "C" fn heap_bytes_free() -> usize {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1340", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Calculates the number of bytes of heap memory currently being used by the application.
! @return The number of bytes on the heap currently being used.*/
/** heap_bytes_used
  -> Added in 17
gneiss: Vtable Slot 336, Offset 1344*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.heap_bytes_used")]
pub unsafe extern "C" fn heap_bytes_used() -> usize {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1344", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Flushes the data cache and invalidates the instruction cache for the given region of memory,
! if necessary. This is only required when your app is loading or modifying code in memory and
! intends to execute it. On some platforms, code executed may be cached internally to improve
! performance. After writing to memory, but before executing, this function must be called in
! order to avoid undefined behavior. On platforms without caching, this performs no operation.
! @param start The beginning of the buffer to flush
! @param size How many bytes to flush*/
/** memory_cache_flush
  -> Added in 84
gneiss: Vtable Slot 626, Offset 2504*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.memory_cache_flush")]
pub unsafe extern "C" fn memory_cache_flush(start: *mut ::core::ffi::c_void, size: usize) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2504", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Operation completed successfully.
pub const StatusCode_S_SUCCESS: StatusCode = 0;
///! An error occurred (no description).
pub const StatusCode_E_ERROR: StatusCode = -1;
///! No idea what went wrong.
pub const StatusCode_E_UNKNOWN: StatusCode = -2;
///! There was a generic internal logic error.
pub const StatusCode_E_INTERNAL: StatusCode = -3;
///! The function was not called correctly.
pub const StatusCode_E_INVALID_ARGUMENT: StatusCode = -4;
///! Insufficient allocatable memory available.
pub const StatusCode_E_OUT_OF_MEMORY: StatusCode = -5;
///! Insufficient long-term storage available.
pub const StatusCode_E_OUT_OF_STORAGE: StatusCode = -6;
///! Insufficient resources available.
pub const StatusCode_E_OUT_OF_RESOURCES: StatusCode = -7;
///! Argument out of range (may be dynamic).
pub const StatusCode_E_RANGE: StatusCode = -8;
///! Target of operation does not exist.
pub const StatusCode_E_DOES_NOT_EXIST: StatusCode = -9;
///! Operation not allowed (may depend on state).
pub const StatusCode_E_INVALID_OPERATION: StatusCode = -10;
///! Another operation prevented this one.
pub const StatusCode_E_BUSY: StatusCode = -11;
///! Operation not completed; try again.
pub const StatusCode_E_AGAIN: StatusCode = -12;
///! Equivalent of boolean true.
pub const StatusCode_S_TRUE: StatusCode = 1;
///! Equivalent of boolean false.
pub const StatusCode_S_FALSE: StatusCode = 0;
///! For list-style requests.  At end of list.
pub const StatusCode_S_NO_MORE_ITEMS: StatusCode = 2;
///! No action was taken as none was required.
pub const StatusCode_S_NO_ACTION_REQUIRED: StatusCode = 3;
///! Status codes. See \ref status_t
pub type StatusCode = ::core::ffi::c_schar;
///! Return value for system operations. See \ref StatusCode for possible values.
pub type status_t = i32;
/**! Checks whether a value has been set for a given key in persistent storage.
! @param key The key of the field to check.
! @return true if a value exists, otherwise false.*/
/** persist_exists
  -> Added in 0
gneiss: Vtable Slot 188, Offset 752*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.persist_exists")]
pub unsafe extern "C" fn persist_exists(key: u32) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =752", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the size of a value for a given key in persistent storage.
! @param key The key of the field to lookup the data size.
! @return The size of the value in bytes or \ref E_DOES_NOT_EXIST
! if there is no field matching the given key.*/
/** persist_get_size
  -> Added in 0
gneiss: Vtable Slot 189, Offset 756*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.persist_get_size")]
pub unsafe extern "C" fn persist_get_size(key: u32) -> ::core::ffi::c_int {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =756", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Reads a bool value for a given key from persistent storage.
! If the value has not yet been set, this will return false.
! @param key The key of the field to read from.
! @return The bool value of the key to read from.*/
/** persist_read_bool
  -> Added in 0
gneiss: Vtable Slot 190, Offset 760*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.persist_read_bool")]
pub unsafe extern "C" fn persist_read_bool(key: u32) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =760", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Reads an int value for a given key from persistent storage.
! @note The int is a signed 32-bit integer.
! If the value has not yet been set, this will return 0.
! @param key The key of the field to read from.
! @return The int value of the key to read from.*/
/** persist_read_int
  -> Added in 0
gneiss: Vtable Slot 192, Offset 768*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.persist_read_int")]
pub unsafe extern "C" fn persist_read_int(key: u32) -> i32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =768", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Reads a blob of data for a given key from persistent storage into a given buffer.
! If the value has not yet been set, the given buffer is left unchanged.
! @param key The key of the field to read from.
! @param buffer The pointer to a buffer to be written to.
! @param buffer_size The maximum size of the given buffer.
! @return The number of bytes written into the buffer or \ref E_DOES_NOT_EXIST
! if there is no field matching the given key.*/
/** persist_read_data
  -> Added in 5
gneiss: Vtable Slot 311, Offset 1244*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.persist_read_data")]
pub unsafe extern "C" fn persist_read_data(
	key: u32,
	buffer: *mut ::core::ffi::c_void,
	buffer_size: usize,
) -> ::core::ffi::c_int {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1244", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Reads a string for a given key from persistent storage into a given buffer.
! The string will be null terminated.
! If the value has not yet been set, the given buffer is left unchanged.
! @param key The key of the field to read from.
! @param buffer The pointer to a buffer to be written to.
! @param buffer_size The maximum size of the given buffer. This includes the null character.
! @return The number of bytes written into the buffer or \ref E_DOES_NOT_EXIST
! if there is no field matching the given key.*/
/** persist_read_string
  -> Added in 5
gneiss: Vtable Slot 312, Offset 1248*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.persist_read_string")]
pub unsafe extern "C" fn persist_read_string(
	key: u32,
	buffer: *mut ::core::ffi::c_char,
	buffer_size: usize,
) -> ::core::ffi::c_int {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1248", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Writes a bool value flag for a given key into persistent storage.
! @param key The key of the field to write to.
! @param value The boolean value to write.
! @return The number of bytes written if successful, a value from \ref StatusCode otherwise.*/
/** persist_write_bool
  -> Added in 0
gneiss: Vtable Slot 194, Offset 776*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.persist_write_bool")]
pub unsafe extern "C" fn persist_write_bool(key: u32, value: bool) -> status_t {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =776", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Writes an int value for a given key into persistent storage.
! @note The int is a signed 32-bit integer.
! @param key The key of the field to write to.
! @param value The int value to write.
! @return The number of bytes written if successful, a value from \ref StatusCode otherwise.*/
/** persist_write_int
  -> Added in 0
gneiss: Vtable Slot 196, Offset 784*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.persist_write_int")]
pub unsafe extern "C" fn persist_write_int(key: u32, value: i32) -> status_t {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =784", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Writes a blob of data of a specified size in bytes for a given key into persistent storage.
! The maximum size is \ref PERSIST_DATA_MAX_LENGTH
! @param key The key of the field to write to.
! @param data The pointer to the blob of data.
! @param size The size in bytes.
! @return The number of bytes written if successful, a value from \ref StatusCode otherwise.*/
/** persist_write_data
  -> Added in 5
gneiss: Vtable Slot 313, Offset 1252*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.persist_write_data")]
pub unsafe extern "C" fn persist_write_data(
	key: u32,
	data: *const ::core::ffi::c_void,
	size: usize,
) -> ::core::ffi::c_int {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1252", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Writes a string a given key into persistent storage.
! The maximum size is \ref PERSIST_STRING_MAX_LENGTH including the null terminator.
! @param key The key of the field to write to.
! @param cstring The pointer to null terminated string.
! @return The number of bytes written if successful, a value from \ref StatusCode otherwise.*/
/** persist_write_string
  -> Added in 0
gneiss: Vtable Slot 197, Offset 788*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.persist_write_string")]
pub unsafe extern "C" fn persist_write_string(
	key: u32,
	cstring: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =788", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Deletes the value of a key from persistent storage.
! @param key The key of the field to delete from.
! @return S_TRUE if successful, E_DOES_NOT_EXIST if a value was not set, or another error value from \ref StatusCode.*/
/** persist_delete
  -> Added in 0
gneiss: Vtable Slot 187, Offset 748*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.persist_delete")]
pub unsafe extern "C" fn persist_delete(key: u32) -> status_t {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =748", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! WakeupId is an identifier for a wakeup event
pub type WakeupId = i32;
/**! The type of function which can be called when a wakeup event occurs.
! The arguments will be the id of the wakeup event that occurred,
! as well as the scheduled cookie provided to @c wakeup_schedule.*/
pub type WakeupHandler =
	::core::option::Option<unsafe extern "C" fn(wakeup_id: WakeupId, cookie: i32)>;
/**! Registers a WakeupHandler to be called when wakeup events occur.
! @note The handler is only called for wakeup events which occur while the app is already running;
!       use @c launch_reason() === \ref APP_LAUNCH_WAKEUP to detect when the app was launched by a wakeup event.
! @param handler The callback that gets called when the wakeup event occurs*/
/** wakeup_service_subscribe
  -> Added in 24
gneiss: Vtable Slot 358, Offset 1432*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.wakeup_service_subscribe")]
pub unsafe extern "C" fn wakeup_service_subscribe(handler: WakeupHandler) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1432", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Registers a wakeup event that triggers a callback at the specified time.
! Applications may only schedule up to 8 wakeup events.
! Wakeup events are given a 1 minute duration window, in that no application may schedule a
! wakeup event with 1 minute of a currently scheduled wakeup event.
! @param timestamp The requested time (UTC) for the wakeup event to occur
! @param cookie The application specific reason for the wakeup event
! @param notify_if_missed On powering on Pebble, will alert user when
! notifications were missed due to Pebble being off.
! @return negative values indicate errors (StatusCode)
! E_RANGE if the event cannot be scheduled due to another event in that period.
! E_INVALID_ARGUMENT if the time requested is in the past.
! E_OUT_OF_RESOURCES if the application has already scheduled all 8 wakeup events.
! E_INTERNAL if a system error occurred during scheduling.*/
/** wakeup_schedule
  -> Added in 24
gneiss: Vtable Slot 357, Offset 1428*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.wakeup_schedule")]
pub unsafe extern "C" fn wakeup_schedule(
	timestamp: time_t,
	cookie: i32,
	notify_if_missed: bool,
) -> WakeupId {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1428", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Cancels a wakeup event.
! @param wakeup_id Wakeup event to cancel*/
/** wakeup_cancel
  -> Added in 24
gneiss: Vtable Slot 353, Offset 1412*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.wakeup_cancel")]
pub unsafe extern "C" fn wakeup_cancel(wakeup_id: WakeupId) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1412", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Cancels all wakeup event for the app.
/** wakeup_cancel_all
  -> Added in 24
gneiss: Vtable Slot 354, Offset 1416*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.wakeup_cancel_all")]
pub unsafe extern "C" fn wakeup_cancel_all() {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1416", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Retrieves the wakeup event info for an app that was launched
! by a wakeup_event (ie. @c launch_reason() === APP_LAUNCH_WAKEUP)
! so that an app may display information regarding the wakeup event
! @param wakeup_id WakeupId for the wakeup event that caused the app to wakeup
! @param cookie App provided reason for the wakeup event
! @return True if app was launched due to a wakeup event, false otherwise*/
/** wakeup_get_launch_event
  -> Added in 24
gneiss: Vtable Slot 355, Offset 1420*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.wakeup_get_launch_event")]
pub unsafe extern "C" fn wakeup_get_launch_event(
	wakeup_id: *mut WakeupId,
	cookie: *mut i32,
) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1420", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Checks if the current WakeupId is still scheduled and therefore valid
! @param wakeup_id Wakeup event to query for validity and scheduled time
! @param timestamp Optionally points to an address of a time_t variable to
! store the time that the wakeup event is scheduled to occur.
! (The time is in UTC, but local time when \ref clock_is_timezone_set
! returns false).
! You may pass NULL instead if you do not need it.
! @return True if WakeupId is still scheduled, false if it doesn't exist or has
! already occurred*/
/** wakeup_query
  -> Added in 24
gneiss: Vtable Slot 356, Offset 1424*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.wakeup_query")]
pub unsafe extern "C" fn wakeup_query(wakeup_id: WakeupId, timestamp: *mut time_t) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1424", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///!< App launched by the system
pub const AppLaunchReason_APP_LAUNCH_SYSTEM: AppLaunchReason = 0;
///!< App launched by user selection in launcher menu
pub const AppLaunchReason_APP_LAUNCH_USER: AppLaunchReason = 1;
///!< App launched by mobile or companion app
pub const AppLaunchReason_APP_LAUNCH_PHONE: AppLaunchReason = 2;
///!< App launched by wakeup event
pub const AppLaunchReason_APP_LAUNCH_WAKEUP: AppLaunchReason = 3;
///!< App launched by worker calling worker_launch_app()
pub const AppLaunchReason_APP_LAUNCH_WORKER: AppLaunchReason = 4;
///!< App launched by user using quick launch
pub const AppLaunchReason_APP_LAUNCH_QUICK_LAUNCH: AppLaunchReason = 5;
///!< App launched by user opening it from a pin
pub const AppLaunchReason_APP_LAUNCH_TIMELINE_ACTION: AppLaunchReason = 6;
///!< App launched by a smartstrap
pub const AppLaunchReason_APP_LAUNCH_SMARTSTRAP: AppLaunchReason = 7;
/**! AppLaunchReason is used to inform the application about how it was launched
! @note New launch reasons may be added in the future. As a best practice, it
! is recommended to only handle the cases that the app needs to know about,
! rather than trying to handle all possible launch reasons.*/
pub type AppLaunchReason = ::core::ffi::c_uchar;
///!< App was not launched via Quick Launch
pub const AppQuickLaunchAction_APP_QUICK_LAUNCH_ACTION_NONE: AppQuickLaunchAction = 0;
///!< User held a single button
pub const AppQuickLaunchAction_APP_QUICK_LAUNCH_ACTION_HOLD: AppQuickLaunchAction = 1;
///!< User tapped a button (single click)
pub const AppQuickLaunchAction_APP_QUICK_LAUNCH_ACTION_TAP: AppQuickLaunchAction = 2;
///!< User held a button combination
pub const AppQuickLaunchAction_APP_QUICK_LAUNCH_ACTION_COMBO: AppQuickLaunchAction = 3;
/**! Details about how an app was quick launched.
! Returned by \ref app_launch_get_quick_launch_action.*/
pub type AppQuickLaunchAction = ::core::ffi::c_uchar;
/**! Provides the method used to launch the current application.
! @return The method or reason the current application was launched*/
/** launch_reason
  -> Added in 24
gneiss: Vtable Slot 352, Offset 1408*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.launch_reason")]
pub unsafe extern "C" fn launch_reason() -> AppLaunchReason {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1408", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the argument passed to the app when it was launched.
! @note Currently the only way to pass arguments to apps is by using an openWatchApp action
! on a pin.
! @return The argument passed to the app, or 0 if the app wasn't launched from a Launch App action*/
/** launch_get_args
  -> Added in 38
gneiss: Vtable Slot 438, Offset 1752*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.launch_get_args")]
pub unsafe extern "C" fn launch_get_args() -> u32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1752", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///!< Exit reason not specified
pub const AppExitReason_APP_EXIT_NOT_SPECIFIED: AppExitReason = 0;
///!< Application performed an action when it exited
pub const AppExitReason_APP_EXIT_ACTION_PERFORMED_SUCCESSFULLY: AppExitReason = 1;
///!< Number of AppExitReason options
pub const AppExitReason_NUM_EXIT_REASONS: AppExitReason = 2;
/**! AppExitReason is used to notify the system of the reason of an application exiting, which may
! affect the part of the system UI that is presented after the application terminates.*/
pub type AppExitReason = ::core::ffi::c_uchar;
/**! Set the app exit reason to a new reason.
! @param exit_reason The new app exit reason*/
/** exit_reason_set
  -> Added in 83
gneiss: Vtable Slot 616, Offset 2464*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.exit_reason_set")]
pub unsafe extern "C" fn exit_reason_set(exit_reason: AppExitReason) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2464", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! The ID of a published app resource defined within the publishedMedia section of package.json.
pub type PublishedId = u32;
///! Describes how the slice should be visualized in the app's glance in the launcher.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct AppGlanceSliceLayout {
	/**! The published resource ID of the bitmap icon to display in the app's glance. Use \ref
	! APP_GLANCE_SLICE_DEFAULT_ICON to use the app's default bitmap icon.*/
	pub icon: PublishedId,
	/**! A template string to visualize in the app's glance. The string will be copied, so it is safe
	! to destroy after adding the slice to the glance. Use NULL if no string should be displayed.*/
	pub subtitle_template_string: *const ::core::ffi::c_char,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of AppGlanceSliceLayout"][::core::mem::size_of::<AppGlanceSliceLayout>() - 8usize];
	["Alignment of AppGlanceSliceLayout"][::core::mem::align_of::<AppGlanceSliceLayout>() - 4usize];
	["Offset of field: AppGlanceSliceLayout::icon"]
		[::core::mem::offset_of!(AppGlanceSliceLayout, icon) - 0usize];
	["Offset of field: AppGlanceSliceLayout::subtitle_template_string"]
		[::core::mem::offset_of!(AppGlanceSliceLayout, subtitle_template_string) - 4usize];
};
/**! An app's glance can change over time as defined by zero or more app glance slices that each
! describe the state of the app glance at a particular point in time. Slices are displayed in the
! order they are added, and they are removed at the specified expiration time.*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct AppGlanceSlice {
	///! Describes how the slice should be visualized in the app's glance in the launcher.
	pub layout:          AppGlanceSliceLayout,
	/**! The UTC time after which this slice should no longer be shown in the app's glance. Use \ref
	! APP_GLANCE_SLICE_NO_EXPIRATION if the slice should never expire.*/
	pub expiration_time: time_t,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of AppGlanceSlice"][::core::mem::size_of::<AppGlanceSlice>() - 12usize];
	["Alignment of AppGlanceSlice"][::core::mem::align_of::<AppGlanceSlice>() - 4usize];
	["Offset of field: AppGlanceSlice::layout"]
		[::core::mem::offset_of!(AppGlanceSlice, layout) - 0usize];
	["Offset of field: AppGlanceSlice::expiration_time"]
		[::core::mem::offset_of!(AppGlanceSlice, expiration_time) - 8usize];
};
///! The slice was successfully added to the app's glance.
pub const AppGlanceResult_APP_GLANCE_RESULT_SUCCESS: AppGlanceResult = 0;
///! The subtitle_template_string provided in the slice was invalid.
pub const AppGlanceResult_APP_GLANCE_RESULT_INVALID_TEMPLATE_STRING: AppGlanceResult = 1;
///! The subtitle_template_string provided in the slice was longer than 150 bytes.
pub const AppGlanceResult_APP_GLANCE_RESULT_TEMPLATE_STRING_TOO_LONG: AppGlanceResult = 2;
///! The icon provided in the slice was invalid.
pub const AppGlanceResult_APP_GLANCE_RESULT_INVALID_ICON: AppGlanceResult = 4;
///! The provided slice would exceed the app glance's slice capacity.
pub const AppGlanceResult_APP_GLANCE_RESULT_SLICE_CAPACITY_EXCEEDED: AppGlanceResult = 8;
///! The expiration_time provided in the slice expires in the past.
pub const AppGlanceResult_APP_GLANCE_RESULT_EXPIRES_IN_THE_PAST: AppGlanceResult = 16;
///! The \ref AppGlanceReloadSession provided was invalid.
pub const AppGlanceResult_APP_GLANCE_RESULT_INVALID_SESSION: AppGlanceResult = 32;
///! Bitfield enum describing the result of trying to add an AppGlanceSlice to an app's glance.
pub type AppGlanceResult = ::core::ffi::c_uchar;
#[repr(C)]
#[derive(Debug)]
pub struct AppGlanceReloadSession {
	_unused: [u8; 0],
}
/**! Add a slice to the app's glance. This function will only succeed if called with a valid
! \ref AppGlanceReloadSession that is provided in an \ref AppGlanceReloadCallback.
! @param session The session variable provided in an \ref AppGlanceReloadCallback
! @param slice The slice to add to the app's glance
! @return The result of trying to add the slice to the app's glance*/
/** app_glance_add_slice
  -> Added in 83
gneiss: Vtable Slot 614, Offset 2456*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_glance_add_slice")]
pub unsafe extern "C" fn app_glance_add_slice(
	session: *mut AppGlanceReloadSession,
	slice: AppGlanceSlice,
) -> AppGlanceResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2456", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! User-provided callback for reloading the slices in the app's glance.
! @param session A session variable that must be passed to \ref app_glance_add_slice when adding
! slices to the app's glance; it becomes invalid when the \ref AppGlanceReloadCallback returns
! @param limit The number of entries that can be added to the app's glance
! @param context User-provided context provided when calling \ref app_glance_reload()*/
pub type AppGlanceReloadCallback = ::core::option::Option<
	unsafe extern "C" fn(
		session: *mut AppGlanceReloadSession,
		limit: usize,
		context: *mut ::core::ffi::c_void,
	),
>;
/**! Clear any existing slices in the app's glance and trigger a reload via the provided callback.
! @param callback A function that will be called to add new slices to the app's glance; even if
! the provided callback is NULL, any existing slices will still be cleared from the app's glance
! @param context User-provided context that will be passed to the callback*/
/** app_glance_reload
  -> Added in 83
gneiss: Vtable Slot 615, Offset 2460*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.app_glance_reload")]
pub unsafe extern "C" fn app_glance_reload(
	callback: AppGlanceReloadCallback,
	context: *mut ::core::ffi::c_void,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2460", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @addtogroup GraphicsTypes Graphics Types
! \brief Basic graphics types (point, rect, size, color, bitmaps, etc.) and utility functions.
!
! @{*/
#[repr(C)]
#[derive(Copy, Clone)]
pub union GColor8 {
	pub argb:             u8,
	pub __bindgen_anon_1: GColor8__bindgen_ty_1,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GColor8__bindgen_ty_1 {
	pub _bitfield_1: __BindgenBitfieldUnit<[u8; 1usize]>,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of GColor8__bindgen_ty_1"][::core::mem::size_of::<GColor8__bindgen_ty_1>() - 1usize];
	["Alignment of GColor8__bindgen_ty_1"]
		[::core::mem::align_of::<GColor8__bindgen_ty_1>() - 1usize];
};
impl GColor8__bindgen_ty_1 {
	#[inline]
	#[allow(unnecessary_transmutes)]
	pub fn b(&self) -> u8 {
		unsafe { ::core::mem::transmute(self._bitfield_1.get_const::<0usize, 2u8>() as u8) }
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub fn set_b(&mut self, val: u8) {
		unsafe {
			let val: u8 = ::core::mem::transmute(val);
			self._bitfield_1.set_const::<0usize, 2u8>(val as u64)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub unsafe fn b_raw(this: *const Self) -> u8 {
		unsafe {
			::core::mem::transmute(
				<__BindgenBitfieldUnit<[u8; 1usize]>>::raw_get_const::<0usize, 2u8>(
					::core::ptr::addr_of!((*this)._bitfield_1),
				) as u8,
			)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub unsafe fn set_b_raw(this: *mut Self, val: u8) {
		unsafe {
			let val: u8 = ::core::mem::transmute(val);
			<__BindgenBitfieldUnit<[u8; 1usize]>>::raw_set_const::<0usize, 2u8>(
				::core::ptr::addr_of_mut!((*this)._bitfield_1),
				val as u64,
			)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub fn g(&self) -> u8 {
		unsafe { ::core::mem::transmute(self._bitfield_1.get_const::<2usize, 2u8>() as u8) }
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub fn set_g(&mut self, val: u8) {
		unsafe {
			let val: u8 = ::core::mem::transmute(val);
			self._bitfield_1.set_const::<2usize, 2u8>(val as u64)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub unsafe fn g_raw(this: *const Self) -> u8 {
		unsafe {
			::core::mem::transmute(
				<__BindgenBitfieldUnit<[u8; 1usize]>>::raw_get_const::<2usize, 2u8>(
					::core::ptr::addr_of!((*this)._bitfield_1),
				) as u8,
			)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub unsafe fn set_g_raw(this: *mut Self, val: u8) {
		unsafe {
			let val: u8 = ::core::mem::transmute(val);
			<__BindgenBitfieldUnit<[u8; 1usize]>>::raw_set_const::<2usize, 2u8>(
				::core::ptr::addr_of_mut!((*this)._bitfield_1),
				val as u64,
			)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub fn r(&self) -> u8 {
		unsafe { ::core::mem::transmute(self._bitfield_1.get_const::<4usize, 2u8>() as u8) }
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub fn set_r(&mut self, val: u8) {
		unsafe {
			let val: u8 = ::core::mem::transmute(val);
			self._bitfield_1.set_const::<4usize, 2u8>(val as u64)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub unsafe fn r_raw(this: *const Self) -> u8 {
		unsafe {
			::core::mem::transmute(
				<__BindgenBitfieldUnit<[u8; 1usize]>>::raw_get_const::<4usize, 2u8>(
					::core::ptr::addr_of!((*this)._bitfield_1),
				) as u8,
			)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub unsafe fn set_r_raw(this: *mut Self, val: u8) {
		unsafe {
			let val: u8 = ::core::mem::transmute(val);
			<__BindgenBitfieldUnit<[u8; 1usize]>>::raw_set_const::<4usize, 2u8>(
				::core::ptr::addr_of_mut!((*this)._bitfield_1),
				val as u64,
			)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub fn a(&self) -> u8 {
		unsafe { ::core::mem::transmute(self._bitfield_1.get_const::<6usize, 2u8>() as u8) }
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub fn set_a(&mut self, val: u8) {
		unsafe {
			let val: u8 = ::core::mem::transmute(val);
			self._bitfield_1.set_const::<6usize, 2u8>(val as u64)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub unsafe fn a_raw(this: *const Self) -> u8 {
		unsafe {
			::core::mem::transmute(
				<__BindgenBitfieldUnit<[u8; 1usize]>>::raw_get_const::<6usize, 2u8>(
					::core::ptr::addr_of!((*this)._bitfield_1),
				) as u8,
			)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub unsafe fn set_a_raw(this: *mut Self, val: u8) {
		unsafe {
			let val: u8 = ::core::mem::transmute(val);
			<__BindgenBitfieldUnit<[u8; 1usize]>>::raw_set_const::<6usize, 2u8>(
				::core::ptr::addr_of_mut!((*this)._bitfield_1),
				val as u64,
			)
		}
	}

	#[inline]
	#[allow(unnecessary_transmutes)]
	pub fn new_bitfield_1(b: u8, g: u8, r: u8, a: u8) -> __BindgenBitfieldUnit<[u8; 1usize]> {
		let mut __bindgen_bitfield_unit: __BindgenBitfieldUnit<[u8; 1usize]> = Default::default();
		__bindgen_bitfield_unit.set_const::<0usize, 2u8>({
			let b: u8 = unsafe { ::core::mem::transmute(b) };
			b as u64
		});
		__bindgen_bitfield_unit.set_const::<2usize, 2u8>({
			let g: u8 = unsafe { ::core::mem::transmute(g) };
			g as u64
		});
		__bindgen_bitfield_unit.set_const::<4usize, 2u8>({
			let r: u8 = unsafe { ::core::mem::transmute(r) };
			r as u64
		});
		__bindgen_bitfield_unit.set_const::<6usize, 2u8>({
			let a: u8 = unsafe { ::core::mem::transmute(a) };
			a as u64
		});
		__bindgen_bitfield_unit
	}
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of GColor8"][::core::mem::size_of::<GColor8>() - 1usize];
	["Alignment of GColor8"][::core::mem::align_of::<GColor8>() - 1usize];
	["Offset of field: GColor8::argb"][::core::mem::offset_of!(GColor8, argb) - 0usize];
};
/**! @addtogroup GraphicsTypes Graphics Types
! \brief Basic graphics types (point, rect, size, color, bitmaps, etc.) and utility functions.
!
! @{*/
pub type GColor = GColor8;
///! True if both colors are identical or both are invisible (i.e. both have alpha values of .a=0).
/** gcolor_equal
  -> Added in 82
gneiss: Vtable Slot 613, Offset 2452*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gcolor_equal")]
pub unsafe extern "C" fn gcolor_equal(x: GColor8, y: GColor8) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2452", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! This method assists in improving the legibility of text on various background colors.
! It takes the background color for the region in question and computes a color for
! maximum legibility.
! @param background_color Background color for the region in question
! @return A legible color for the given background color*/
/** gcolor_legible_over
  -> Added in 60
gneiss: Vtable Slot 533, Offset 2132*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gcolor_legible_over")]
pub unsafe extern "C" fn gcolor_legible_over(background_color: GColor8) -> GColor8 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2132", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Represents a point in a 2-dimensional coordinate system.
! @note Conventionally, the origin of Pebble's 2D coordinate system is in the upper,
! lefthand corner
! its x-axis extends to the right and its y-axis extends to the bottom of the screen.*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GPoint {
	///! The x-coordinate.
	pub x: i16,
	///! The y-coordinate.
	pub y: i16,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of GPoint"][::core::mem::size_of::<GPoint>() - 4usize];
	["Alignment of GPoint"][::core::mem::align_of::<GPoint>() - 2usize];
	["Offset of field: GPoint::x"][::core::mem::offset_of!(GPoint, x) - 0usize];
	["Offset of field: GPoint::y"][::core::mem::offset_of!(GPoint, y) - 2usize];
};
/**! Tests whether 2 points are equal.
! @param point_a Pointer to the first point
! @param point_b Pointer to the second point
! @return `true` if both points are equal, `false` if not.*/
/** gpoint_equal
  -> Added in 0
gneiss: Vtable Slot 111, Offset 444*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gpoint_equal")]
pub unsafe extern "C" fn gpoint_equal(point_a: *const GPoint, point_b: *const GPoint) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =444", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Represents a 2-dimensional size.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GSize {
	///! The width
	pub w: i16,
	///! The height
	pub h: i16,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of GSize"][::core::mem::size_of::<GSize>() - 4usize];
	["Alignment of GSize"][::core::mem::align_of::<GSize>() - 2usize];
	["Offset of field: GSize::w"][::core::mem::offset_of!(GSize, w) - 0usize];
	["Offset of field: GSize::h"][::core::mem::offset_of!(GSize, h) - 2usize];
};
/**! Tests whether 2 sizes are equal.
! @param size_a Pointer to the first size
! @param size_b Pointer to the second size
! @return `true` if both sizes are equal, `false` if not.*/
/** gsize_equal
  -> Added in 0
gneiss: Vtable Slot 134, Offset 536*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gsize_equal")]
pub unsafe extern "C" fn gsize_equal(size_a: *const GSize, size_b: *const GSize) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =536", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Represents a rectangle and defining it using the origin of
! the upper-lefthand corner and its size.*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GRect {
	///! The coordinate of the upper-lefthand corner point of the rectangle.
	pub origin: GPoint,
	///! The size of the rectangle.
	pub size:   GSize,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of GRect"][::core::mem::size_of::<GRect>() - 8usize];
	["Alignment of GRect"][::core::mem::align_of::<GRect>() - 2usize];
	["Offset of field: GRect::origin"][::core::mem::offset_of!(GRect, origin) - 0usize];
	["Offset of field: GRect::size"][::core::mem::offset_of!(GRect, size) - 4usize];
};
/**! Tests whether 2 rectangles are equal.
! @param rect_a Pointer to the first rectangle
! @param rect_b Pointer to the second rectangle
! @return `true` if both rectangles are equal, `false` if not.*/
/** grect_equal
  -> Added in 0
gneiss: Vtable Slot 131, Offset 524*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.grect_equal")]
pub unsafe extern "C" fn grect_equal(rect_a: *const GRect, rect_b: *const GRect) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =524", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Tests whether the size of the rectangle is (0, 0).
! @param rect Pointer to the rectangle
! @return `true` if the rectangle its size is (0, 0), or `false` if not.
! @note If the width and/or height of a rectangle is negative, this
! function will return `true`!*/
/** grect_is_empty
  -> Added in 0
gneiss: Vtable Slot 132, Offset 528*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.grect_is_empty")]
pub unsafe extern "C" fn grect_is_empty(rect: *const GRect) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =528", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Converts a rectangle's values so that the components of its size
! (width and/or height) are both positive. In the width and/or height are negative,
! the origin will offset, so that the final rectangle overlaps with the original.
! For example, a GRect with size (-10, -5) and origin (20, 20), will be standardized
! to size (10, 5) and origin (10, 15).
! @param[in,out] rect The rectangle to convert, set to the standardized rectangle.*/
/** grect_standardize
  -> Added in 0
gneiss: Vtable Slot 133, Offset 532*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.grect_standardize")]
pub unsafe extern "C" fn grect_standardize(rect: *mut GRect) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =532", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Trim one rectangle using the edges of a second rectangle.
! @param[in,out] rect_to_clip The rectangle that needs to be clipped (in place).
! @param rect_clipper The rectangle of which the edges will serve as "scissors"
! in order to trim `rect_to_clip`.*/
/** grect_clip
  -> Added in 0
gneiss: Vtable Slot 128, Offset 512*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.grect_clip")]
pub unsafe extern "C" fn grect_clip(rect_to_clip: *mut GRect, rect_clipper: *const GRect) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =512", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Tests whether a rectangle contains a point.
! @param rect The rectangle
! @param point The point
! @return `true` if the rectangle contains the point, or `false` if it does not.*/
/** grect_contains_point
  -> Added in 0
gneiss: Vtable Slot 129, Offset 516*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.grect_contains_point")]
pub unsafe extern "C" fn grect_contains_point(rect: *const GRect, point: *const GPoint) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =516", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Convenience function to compute the center-point of a given rectangle.
! This is equal to `(rect->x + rect->width / 2, rect->y + rect->height / 2)`.
! @param rect The rectangle for which to calculate the center point.
! @return The point at the center of `rect`*/
/** grect_center_point
  -> Added in 0
gneiss: Vtable Slot 127, Offset 508*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.grect_center_point")]
pub unsafe extern "C" fn grect_center_point(rect: *const GRect) -> GPoint {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =508", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Reduce the width and height of a rectangle by insetting each of the edges with
! a fixed inset. The returned rectangle will be centered relative to the input rectangle.
! @note The function will trip an assertion if the crop yields a rectangle with negative width or height.
! @param rect The rectangle that will be inset
! @param crop_size_px The inset by which each of the rectangle will be inset.
! A positive inset value results in a smaller rectangle, while negative inset value results
! in a larger rectangle.
! @return The cropped rectangle.*/
/** grect_crop
  -> Added in 0
gneiss: Vtable Slot 130, Offset 520*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.grect_crop")]
pub unsafe extern "C" fn grect_crop(rect: GRect, crop_size_px: i32) -> GRect {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =520", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
pub const GBitmapFormat_GBitmapFormat1Bit: GBitmapFormat = 0;
pub const GBitmapFormat_GBitmapFormat8Bit: GBitmapFormat = 1;
pub const GBitmapFormat_GBitmapFormat1BitPalette: GBitmapFormat = 2;
pub const GBitmapFormat_GBitmapFormat2BitPalette: GBitmapFormat = 3;
pub const GBitmapFormat_GBitmapFormat4BitPalette: GBitmapFormat = 4;
pub const GBitmapFormat_GBitmapFormat8BitCircular: GBitmapFormat = 5;
///! The format of a GBitmap can either be 1-bit or 8-bit.
pub type GBitmapFormat = ::core::ffi::c_uchar;
#[repr(C)]
#[derive(Debug)]
pub struct GBitmap {
	_unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct GBitmapSequence {
	_unused: [u8; 0],
}
/**! Get the number of bytes per row in the bitmap data for the given \ref GBitmap.
! On rectangular displays, this can be used as a safe way of iterating over the rows in the
! bitmap, since bytes per row should be set according to format. On circular displays with pixel
! format of \ref GBitmapFormat8BitCircular this will return 0, and should not be used for
! iteration over frame buffer pixels. Instead, use \ref GBitmapDataRowInfo, which provides safe
! minimum and maximum x values for a given row's y value.
! @param bitmap A pointer to the GBitmap to get the bytes per row
! @return The number of bytes per row of the GBitmap
! @see \ref gbitmap_get_data*/
/** gbitmap_get_bytes_per_row
  -> Added in 33
gneiss: Vtable Slot 408, Offset 1632*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_get_bytes_per_row")]
pub unsafe extern "C" fn gbitmap_get_bytes_per_row(bitmap: *const GBitmap) -> u16 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1632", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the \ref GBitmapFormat for the \ref GBitmap.
! @param bitmap A pointer to the GBitmap to get the format
! @return The format of the given \ref GBitmap.*/
/** gbitmap_get_format
  -> Added in 33
gneiss: Vtable Slot 410, Offset 1640*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_get_format")]
pub unsafe extern "C" fn gbitmap_get_format(bitmap: *const GBitmap) -> GBitmapFormat {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1640", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get a pointer to the raw image data section of the given \ref GBitmap as specified by the format
! of the bitmap.
! @param bitmap A pointer to the GBitmap to get the data
! @return pointer to the raw image data for the GBitmap
! @see \ref gbitmap_get_bytes_per_row
! @see \ref GBitmap*/
/** gbitmap_get_data
  -> Added in 33
gneiss: Vtable Slot 409, Offset 1636*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_get_data")]
pub unsafe extern "C" fn gbitmap_get_data(bitmap: *const GBitmap) -> *mut u8 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1636", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set the bitmap data for the given \ref GBitmap.
! @param bitmap A pointer to the GBitmap to set data to
! @param data A pointer to the bitmap data
! @param format the format of the bitmap data. If this is a palettized format, make sure that
! there is an accompanying call to \ref gbitmap_set_palette.
! @param row_size_bytes How many bytes a single row takes. For example, bitmap data of format
! \ref GBitmapFormat1Bit must have a row size as a multiple of 4 bytes.
! @param free_on_destroy Set whether the data should be freed when the GBitmap is destroyed.
! @see \ref gbitmap_destroy*/
/** gbitmap_set_data
  -> Added in 33
gneiss: Vtable Slot 413, Offset 1652*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_set_data")]
pub unsafe extern "C" fn gbitmap_set_data(
	bitmap: *mut GBitmap,
	data: *mut u8,
	format: GBitmapFormat,
	row_size_bytes: u16,
	free_on_destroy: bool,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1652", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the bounds of the content for the \ref GBitmap. This is set when loading the image or
! if changed by \ref gbitmap_set_bounds.
! @param bitmap A pointer to the GBitmap to get the bounding box from.
! @return The bounding box for the GBitmap.
! @see \ref gbitmap_set_bounds*/
/** gbitmap_get_bounds
  -> Added in 33
gneiss: Vtable Slot 407, Offset 1628*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_get_bounds")]
pub unsafe extern "C" fn gbitmap_get_bounds(bitmap: *const GBitmap) -> GRect {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1628", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set the bounds of the given \ref GBitmap.
! @param bitmap A pointer to the GBitmap to set the bounding box.
! @param bounds The bounding box to set.
! @see \ref gbitmap_get_bounds*/
/** gbitmap_set_bounds
  -> Added in 33
gneiss: Vtable Slot 412, Offset 1648*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_set_bounds")]
pub unsafe extern "C" fn gbitmap_set_bounds(bitmap: *mut GBitmap, bounds: GRect) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1648", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the palette for the given \ref GBitmap.
! @param bitmap A pointer to the GBitmap to get the palette from.
! @return Pointer to a \ref GColor array containing the palette colors.
! @see \ref gbitmap_set_palette*/
/** gbitmap_get_palette
  -> Added in 33
gneiss: Vtable Slot 411, Offset 1644*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_get_palette")]
pub unsafe extern "C" fn gbitmap_get_palette(bitmap: *const GBitmap) -> *mut GColor {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1644", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set the palette for the given \ref GBitmap.
! @param bitmap A pointer to the GBitmap to set the palette to
! @param palette The palette to be used. Make sure that the palette is large enough for the
! bitmap's format.
! @param free_on_destroy Set whether the palette data should be freed when the GBitmap is
! destroyed or when another palette is set.
! @see \ref gbitmap_get_format
! @see \ref gbitmap_destroy
! @see \ref gbitmap_set_palette*/
/** gbitmap_set_palette
  -> Added in 33
gneiss: Vtable Slot 414, Offset 1656*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_set_palette")]
pub unsafe extern "C" fn gbitmap_set_palette(
	bitmap: *mut GBitmap,
	palette: *mut GColor,
	free_on_destroy: bool,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1656", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Creates a new \ref GBitmap on the heap using a Pebble image file stored as a resource.
! The resulting GBitmap must be destroyed using \ref gbitmap_destroy().
! @param resource_id The ID of the bitmap resource to load
! @return A pointer to the \ref GBitmap. `NULL` if the GBitmap could not
! be created*/
/** gbitmap_create_with_resource
  -> Added in 0
gneiss: Vtable Slot 102, Offset 408*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_create_with_resource")]
pub unsafe extern "C" fn gbitmap_create_with_resource(resource_id: u32) -> *mut GBitmap {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =408", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Creates a new GBitmap on the heap initialized with the provided Pebble image data.
!
! The resulting \ref GBitmap must be destroyed using \ref gbitmap_destroy() but the image
! data will not be freed automatically. The developer is responsible for keeping the image
! data in memory as long as the bitmap is used and releasing it after the bitmap is destroyed.
! @note One way to generate Pebble image data is to use bitmapgen.py in the Pebble
! SDK to generate a .pbi file.
! @param data The Pebble image data. Must not be NULL. The function
! assumes the data to be correct; there are no sanity checks performed on the
! data. The data will not be copied and the pointer must remain valid for the
! lifetime of this GBitmap.
! @return A pointer to the \ref GBitmap. `NULL` if the \ref GBitmap could not
! be created*/
/** gbitmap_create_with_data
  -> Added in 0
gneiss: Vtable Slot 101, Offset 404*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_create_with_data")]
pub unsafe extern "C" fn gbitmap_create_with_data(data: *const u8) -> *mut GBitmap {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =404", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Create a new \ref GBitmap on the heap as a sub-bitmap of a 'base' \ref
! GBitmap, using a GRect to indicate what portion of the base to use. The
! sub-bitmap will just reference the image data and palette of the base bitmap.
! No deep-copying occurs as a result of calling this function, thus the caller
! is responsible for making sure the base bitmap and palette will remain available when
! using the sub-bitmap. Note that you should not destroy the parent bitmap until
! the sub_bitmap has been destroyed.
! The resulting \ref GBitmap must be destroyed using \ref gbitmap_destroy().
! @param[in] base_bitmap The bitmap that the sub-bitmap of which the image data
! will be used by the sub-bitmap
! @param sub_rect The rectangle within the image data of the base bitmap. The
! bounds of the base bitmap will be used to clip `sub_rect`.
! @return A pointer to the \ref GBitmap. `NULL` if the GBitmap could not
! be created*/
/** gbitmap_create_as_sub_bitmap
  -> Added in 0
gneiss: Vtable Slot 100, Offset 400*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_create_as_sub_bitmap")]
pub unsafe extern "C" fn gbitmap_create_as_sub_bitmap(
	base_bitmap: *const GBitmap,
	sub_rect: GRect,
) -> *mut GBitmap {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =400", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Create a \ref GBitmap based on raw PNG data.
! The resulting \ref GBitmap must be destroyed using \ref gbitmap_destroy().
! The developer is responsible for freeing png_data following this call.
! @note PNG decoding currently supports 1,2,4 and 8 bit palettized and grayscale images.
! @param png_data PNG image data.
! @param png_data_size PNG image size in bytes.
! @return A pointer to the \ref GBitmap. `NULL` if the \ref GBitmap could not
! be created*/
/** gbitmap_create_from_png_data
  -> Added in 35
gneiss: Vtable Slot 421, Offset 1684*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_create_from_png_data")]
pub unsafe extern "C" fn gbitmap_create_from_png_data(
	png_data: *const u8,
	png_data_size: usize,
) -> *mut GBitmap {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1684", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Creates a new blank GBitmap on the heap initialized to zeroes.
! In the case that the format indicates a palettized bitmap, a palette of appropriate size will
! also be allocated on the heap.
! The resulting \ref GBitmap must be destroyed using \ref gbitmap_destroy().
! @param size The Pebble image dimensions as a \ref GSize.
! @param format The \ref GBitmapFormat the created image should be in.
! @return A pointer to the \ref GBitmap. `NULL` if the \ref GBitmap could not
! be created*/
/** gbitmap_create_blank
  -> Added in 32
gneiss: Vtable Slot 393, Offset 1572*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_create_blank")]
pub unsafe extern "C" fn gbitmap_create_blank(size: GSize, format: GBitmapFormat) -> *mut GBitmap {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1572", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Creates a new blank GBitmap on the heap, initialized to zeroes, and assigns it the given
! palette.
! No deep-copying of the palette occurs, so the caller is responsible for making sure the palette
! remains available when using the resulting bitmap. Management of that memory can be handed off
! to the system with the free_on_destroy argument.
! @param size The Pebble image dimensions as a \ref GSize.
! @param format the \ref GBitmapFormat the created image and palette should be in.
! @param palette a pointer to a palette that is to be used for this GBitmap. The palette should
! be large enough to hold enough colors for the specified format. For example,
! \ref GBitmapFormat2BitPalette should have 4 colors, since 2^2 = 4.
! @param free_on_destroy Set whether the palette data should be freed along with the bitmap data
! when the GBitmap is destroyed.
! @return A Pointer to the \ref GBitmap. `NULL` if the \ref GBitmap could not be created.*/
/** gbitmap_create_blank_with_palette
  -> Added in 33
gneiss: Vtable Slot 406, Offset 1624*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_create_blank_with_palette")]
pub unsafe extern "C" fn gbitmap_create_blank_with_palette(
	size: GSize,
	format: GBitmapFormat,
	palette: *mut GColor,
	free_on_destroy: bool,
) -> *mut GBitmap {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1624", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Given a 1-bit GBitmap, create a new bitmap of format GBitmapFormat1BitPalette.
! The new data buffer is allocated on the heap, and a 2-color palette is allocated as well.
! @param src_bitmap A GBitmap of format GBitmapFormat1Bit which is to be copied into a newly
! created GBitmap of format GBitmapFormat1BitPalettized.
! @returns The newly created 1-bit palettized GBitmap, or NULL if there is not sufficient space.
! @note The new bitmap does not depend on any data from src_bitmap, so src_bitmap can be freed
! without worry.*/
/** gbitmap_create_palettized_from_1bit
  -> Added in 44
gneiss: Vtable Slot 458, Offset 1832*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_create_palettized_from_1bit")]
pub unsafe extern "C" fn gbitmap_create_palettized_from_1bit(
	src_bitmap: *const GBitmap,
) -> *mut GBitmap {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1832", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Destroy a \ref GBitmap.
! This must be called for every bitmap that's been created with gbitmap_create_*
!
! This function will also free the memory of the bitmap data (bitmap->addr) if the bitmap was created with \ref gbitmap_create_blank()
! or \ref gbitmap_create_with_resource().
!
! If the GBitmap was created with \ref gbitmap_create_with_data(), you must release the memory
! after calling gbitmap_destroy().*/
/** gbitmap_destroy
  -> Added in 0
gneiss: Vtable Slot 103, Offset 412*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_destroy")]
pub unsafe extern "C" fn gbitmap_destroy(bitmap: *mut GBitmap) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =412", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Creates a GBitmapSequence from the specified resource (APNG/PNG files)
! @param resource_id Resource to load and create GBitmapSequence from.
! @return GBitmapSequence pointer if the resource was loaded, NULL otherwise*/
/** gbitmap_sequence_create_with_resource
  -> Added in 34
gneiss: Vtable Slot 415, Offset 1660*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_sequence_create_with_resource")]
pub unsafe extern "C" fn gbitmap_sequence_create_with_resource(
	resource_id: u32,
) -> *mut GBitmapSequence {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1660", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Updates the contents of the bitmap sequence to the next frame
! and optionally returns the delay in milliseconds until the next frame.
! @param bitmap_sequence Pointer to loaded bitmap sequence
! @param bitmap Pointer to the initialized GBitmap in which to render the bitmap sequence
! @param[out] delay_ms If not NULL, returns the delay in milliseconds until the next frame.
! @return True if frame was rendered.  False if all frames (and loops) have been rendered
! for the sequence.  Will also return false if frame could not be rendered
! (includes out of memory errors).
! @note GBitmap must be large enough to accommodate the bitmap_sequence image
! \ref gbitmap_sequence_get_bitmap_size*/
/** gbitmap_sequence_update_bitmap_next_frame
  -> Added in 34
gneiss: Vtable Slot 420, Offset 1680*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_sequence_update_bitmap_next_frame")]
pub unsafe extern "C" fn gbitmap_sequence_update_bitmap_next_frame(
	bitmap_sequence: *mut GBitmapSequence,
	bitmap: *mut GBitmap,
	delay_ms: *mut u32,
) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1680", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Updates the contents of the bitmap sequence to the frame at elapsed in the sequence.
! For looping animations this accounts for the loop, for example an animation of 1 second that
! is configured to loop 2 times updated to 1500 ms elapsed time will display the sequence
! frame at 500 ms.  Elapsed time is the time from the start of the animation, and will
! be ignored if it is for a time earlier than the last rendered frame.
! @param bitmap_sequence Pointer to loaded bitmap sequence
! @param bitmap Pointer to the initialized GBitmap in which to render the bitmap sequence
! @param elapsed_ms Elapsed time in milliseconds in the sequence relative to start
! @return True if a frame was rendered.  False if all frames (and loops) have already
! been rendered for the sequence.  Will also return false if frame could not be rendered
! (includes out of memory errors).
! @note GBitmap must be large enough to accommodate the bitmap_sequence image
! \ref gbitmap_sequence_get_bitmap_size
! @note This function is disabled for play_count 0*/
/** gbitmap_sequence_update_bitmap_by_elapsed
  -> Added in 43
gneiss: Vtable Slot 457, Offset 1828*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_sequence_update_bitmap_by_elapsed")]
pub unsafe extern "C" fn gbitmap_sequence_update_bitmap_by_elapsed(
	bitmap_sequence: *mut GBitmapSequence,
	bitmap: *mut GBitmap,
	elapsed_ms: u32,
) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1828", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Deletes the GBitmapSequence structure and frees any allocated memory/decoder_data
! @param bitmap_sequence Pointer to the bitmap sequence to free (delete)*/
/** gbitmap_sequence_destroy
  -> Added in 34
gneiss: Vtable Slot 416, Offset 1664*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_sequence_destroy")]
pub unsafe extern "C" fn gbitmap_sequence_destroy(bitmap_sequence: *mut GBitmapSequence) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1664", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Restarts the GBitmapSequence to the first frame \ref gbitmap_sequence_update_bitmap_next_frame
! @param bitmap_sequence Pointer to loaded bitmap sequence
! @return True if sequence was restarted, false otherwise*/
/** gbitmap_sequence_restart
  -> Added in 40
gneiss: Vtable Slot 442, Offset 1768*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_sequence_restart")]
pub unsafe extern "C" fn gbitmap_sequence_restart(bitmap_sequence: *mut GBitmapSequence) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1768", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! This function gets the current frame number for the bitmap sequence
! @param bitmap_sequence Pointer to loaded bitmap sequence
! @return index of current frame in the current loop of the bitmap sequence*/
/** gbitmap_sequence_get_current_frame_idx
  -> Added in 34
gneiss: Vtable Slot 418, Offset 1672*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_sequence_get_current_frame_idx")]
pub unsafe extern "C" fn gbitmap_sequence_get_current_frame_idx(
	bitmap_sequence: *mut GBitmapSequence,
) -> i32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1672", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! This function sets the total number of frames for the bitmap sequence
! @param bitmap_sequence Pointer to loaded bitmap sequence
! @return number of frames contained in a single loop of the bitmap sequence*/
/** gbitmap_sequence_get_total_num_frames
  -> Added in 34
gneiss: Vtable Slot 419, Offset 1676*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_sequence_get_total_num_frames")]
pub unsafe extern "C" fn gbitmap_sequence_get_total_num_frames(
	bitmap_sequence: *mut GBitmapSequence,
) -> u32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1676", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! This function gets the play count (number of times to repeat) the bitmap sequence
! @note This value is initialized by the bitmap sequence data, and is modified by
! \ref gbitmap_sequence_set_play_count
! @param bitmap_sequence Pointer to loaded bitmap sequence
! @return Play count of bitmap sequence, PLAY_COUNT_INFINITE for infinite looping*/
/** gbitmap_sequence_get_play_count
  -> Added in 40
gneiss: Vtable Slot 441, Offset 1764*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_sequence_get_play_count")]
pub unsafe extern "C" fn gbitmap_sequence_get_play_count(
	bitmap_sequence: *mut GBitmapSequence,
) -> u32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1764", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! This function sets the play count (number of times to repeat) the bitmap sequence
! @param bitmap_sequence Pointer to loaded bitmap sequence
! @param play_count Number of times to repeat the bitmap sequence
! with 0 disabling update_by_elapsed and update_next_frame, and
! PLAY_COUNT_INFINITE for infinite looping of the animation*/
/** gbitmap_sequence_set_play_count
  -> Added in 40
gneiss: Vtable Slot 443, Offset 1772*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_sequence_set_play_count")]
pub unsafe extern "C" fn gbitmap_sequence_set_play_count(
	bitmap_sequence: *mut GBitmapSequence,
	play_count: u32,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1772", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! This function gets the minimum required size (dimensions) necessary
! to render the bitmap sequence to a GBitmap
! using the /ref gbitmap_sequence_update_bitmap_next_frame
! @param bitmap_sequence Pointer to loaded bitmap sequence
! @return Dimensions required to render the bitmap sequence to a GBitmap*/
/** gbitmap_sequence_get_bitmap_size
  -> Added in 34
gneiss: Vtable Slot 417, Offset 1668*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_sequence_get_bitmap_size")]
pub unsafe extern "C" fn gbitmap_sequence_get_bitmap_size(
	bitmap_sequence: *mut GBitmapSequence,
) -> GSize {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1668", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Description of a single data row in the pixel data of a bitmap
! @note This data type describes the actual pixel data of a bitmap and does not respect the
!       bitmap's bounds.
! @see \ref gbitmap_get_data_row_info
! @see \ref gbitmap_get_data
! @see \ref gbitmap_get_bounds*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GBitmapDataRowInfo {
	/**! Address of the byte at column 0 of a given data row in a bitmap. Use this to calculate the
	! memory address of a pixel. For GBitmapFormat8BitCircular or GBitmapFormat8Bit this would
	! be: `uint8_t *pixel_addr = row_info.addr + x`.
	! Note that this byte can be outside of the valid range for this row.
	! For example: The first valid pixel (`min_x=76`) of a row might start at 76 bytes after the
	! given data pointer (assuming that 1 pixel is represented as 1 byte as in
	! GBitmapFormat8BitCircular or GBitmapFormat8Bit).*/
	pub data:  *mut u8,
	///! The absolute column of a first valid pixel for a given data row.
	pub min_x: i16,
	/**! The absolute column of the last valid pixel for a given data row.
	! For optimization reasons the result can be anywhere between
	! grect_get_max_x(bitmap_bounds) - 1 and the physical data boundary.*/
	pub max_x: i16,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of GBitmapDataRowInfo"][::core::mem::size_of::<GBitmapDataRowInfo>() - 8usize];
	["Alignment of GBitmapDataRowInfo"][::core::mem::align_of::<GBitmapDataRowInfo>() - 4usize];
	["Offset of field: GBitmapDataRowInfo::data"]
		[::core::mem::offset_of!(GBitmapDataRowInfo, data) - 0usize];
	["Offset of field: GBitmapDataRowInfo::min_x"]
		[::core::mem::offset_of!(GBitmapDataRowInfo, min_x) - 4usize];
	["Offset of field: GBitmapDataRowInfo::max_x"]
		[::core::mem::offset_of!(GBitmapDataRowInfo, max_x) - 6usize];
};
/**! Provides information about a pixel data row
! @param bitmap A pointer to the GBitmap to get row info
! @param y Absolute row number in the pixel data, independent from the bitmap's bounds
! @return Description of the row
! @note This function does not respect the bitmap's bounds but purely operates on the pixel data.
!       This function works with every bitmap format including GBitmapFormat1Bit.
!       The result of the function for invalid rows is undefined.
! @see \ref gbitmap_get_data*/
/** gbitmap_get_data_row_info
  -> Added in 70
gneiss: Vtable Slot 571, Offset 2284*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gbitmap_get_data_row_info")]
pub unsafe extern "C" fn gbitmap_get_data_row_info(
	bitmap: *const GBitmap,
	y: u16,
) -> GBitmapDataRowInfo {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2284", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Align by centering
pub const GAlign_GAlignCenter: GAlign = 0;
///! Align by making the top edges overlap and left edges overlap
pub const GAlign_GAlignTopLeft: GAlign = 1;
///! Align by making the top edges overlap and left edges overlap
pub const GAlign_GAlignTopRight: GAlign = 2;
///! Align by making the top edges overlap and centered horizontally
pub const GAlign_GAlignTop: GAlign = 3;
///! Align by making the left edges overlap and centered vertically
pub const GAlign_GAlignLeft: GAlign = 4;
///! Align by making the bottom edges overlap and centered horizontally
pub const GAlign_GAlignBottom: GAlign = 5;
///! Align by making the right edges overlap and centered vertically
pub const GAlign_GAlignRight: GAlign = 6;
///! Align by making the bottom edges overlap and right edges overlap
pub const GAlign_GAlignBottomRight: GAlign = 7;
///! Align by making the bottom edges overlap and left edges overlap
pub const GAlign_GAlignBottomLeft: GAlign = 8;
/**! Values to specify how two things should be aligned relative to each other.
! ![](galign.png)
! @see \ref bitmap_layer_set_alignment()*/
pub type GAlign = ::core::ffi::c_uchar;
/**! Aligns one rectangle within another rectangle, using an alignment parameter.
! The relative coordinate systems of both rectangles are assumed to be the same.
! When clip is true, `rect` is also clipped by the constraint.
! @param[in,out] rect The rectangle to align (in place), set to the aligned and
! optionally clipped rectangle
! @param inside_rect The rectangle in which to align `rect`
! @param alignment Determines the alignment of `rect` within `inside_rect` by
! specifying what edges of should overlap.
! @param clip Determines whether `rect` should be trimmed using the edges of `inside_rect`
! in case `rect` extends outside of the area that `inside_rect` covers after the alignment.*/
/** grect_align
  -> Added in 0
gneiss: Vtable Slot 126, Offset 504*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.grect_align")]
pub unsafe extern "C" fn grect_align(
	rect: *mut GRect,
	inside_rect: *const GRect,
	alignment: GAlign,
	clip: bool,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =504", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Assign the pixel values of the source image to the destination pixels,
! effectively replacing the previous values for those pixels. For color displays, when drawing
! a palettized or 8-bit \ref GBitmap image, the opacity value is ignored.*/
pub const GCompOp_GCompOpAssign: GCompOp = 0;
/**! Assign the **inverted** pixel values of the source image to the destination pixels,
! effectively replacing the previous values for those pixels.
! @note For bitmaps with a format different from GBitmapFormat1Bit, this mode is not supported
!     and the resulting behavior is undefined.*/
pub const GCompOp_GCompOpAssignInverted: GCompOp = 1;
/**! Use the boolean operator `OR` to composite the source and destination pixels.
! The visual result of this compositing mode is the source's white pixels
! are painted onto the destination and the source's black pixels are treated
! as clear.
! @note For bitmaps with a format different from GBitmapFormat1Bit, this mode is not supported
!     and the resulting behavior is undefined.*/
pub const GCompOp_GCompOpOr: GCompOp = 2;
/**! Use the boolean operator `AND` to composite the source and destination pixels.
! The visual result of this compositing mode is the source's black pixels
! are painted onto the destination and the source's white pixels are treated
! as clear.
! @note For bitmaps with a format different from GBitmapFormat1Bit, this mode is not supported
!     and the resulting behavior is undefined.*/
pub const GCompOp_GCompOpAnd: GCompOp = 3;
/**! Clears the bits in the destination image, using the source image as mask.
! The visual result of this compositing mode is that for the parts where the source image is
! white, the destination image will be painted black. Other parts will be left untouched.
! @note For bitmaps with a format different from GBitmapFormat1Bit, this mode is not supported
!     and the resulting behavior is undefined.*/
pub const GCompOp_GCompOpClear: GCompOp = 4;
/**! Sets the bits in the destination image, using the source image as mask.
! This mode is required to apply any transparency of your bitmap.
! @note For bitmaps of the format GBitmapFormat1Bit, the visual result of this compositing
!   mode is that for the parts where the source image is black, the destination image will be
!   painted white. Other parts will be left untouched.*/
pub const GCompOp_GCompOpSet: GCompOp = 5;
/**! Values to specify how the source image should be composited onto the destination image.
!
! ![](compops.png)
! Contrived example of how the different compositing modes affect drawing.
! Often, the "destination image" is the render buffer and thus contains the image of
! what has been drawn before or "underneath".
!
! For color displays, only two compositing modes are supported, \ref GCompOpAssign and
! \ref GCompOpSet. The behavior of other compositing modes are undefined and may change in the
! future. Transparency can be achieved using \ref GCompOpSet and requires pixel values with alpha
! value .a < 3.
! @see \ref bitmap_layer_set_compositing_mode()
! @see \ref graphics_context_set_compositing_mode()
! @see \ref graphics_draw_bitmap_in_rect()
! @see \ref graphics_draw_rotated_bitmap()*/
pub type GCompOp = ::core::ffi::c_uchar;
#[repr(C)]
#[derive(Debug)]
pub struct GContext {
	_unused: [u8; 0],
}
/**! Represents insets for four sides. Negative values mean a side extends.
! @see \ref grect_inset*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GEdgeInsets {
	///! The inset at the top of an object.
	pub top:    i16,
	///! The inset at the right of an object.
	pub right:  i16,
	///! The inset at the bottom of an object.
	pub bottom: i16,
	///! The inset at the left of an object.
	pub left:   i16,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of GEdgeInsets"][::core::mem::size_of::<GEdgeInsets>() - 8usize];
	["Alignment of GEdgeInsets"][::core::mem::align_of::<GEdgeInsets>() - 2usize];
	["Offset of field: GEdgeInsets::top"][::core::mem::offset_of!(GEdgeInsets, top) - 0usize];
	["Offset of field: GEdgeInsets::right"][::core::mem::offset_of!(GEdgeInsets, right) - 2usize];
	["Offset of field: GEdgeInsets::bottom"][::core::mem::offset_of!(GEdgeInsets, bottom) - 4usize];
	["Offset of field: GEdgeInsets::left"][::core::mem::offset_of!(GEdgeInsets, left) - 6usize];
};
/**! Returns a rectangle that is shrinked or expanded by the given edge insets.
! @note The rectangle is standardized and then the inset parameters are applied.
! If the resulting rectangle would have a negative height or width, a GRectZero is returned.
! @param rect The rectangle that will be inset
! @param insets The insets that will be applied
! @return The resulting rectangle
! @note Use this function in together with the \ref GEdgeInsets macro
! \code{.c}
! GRect r_inset_all_sides = grect_inset(r, GEdgeInsets(10));
! GRect r_inset_vertical_horizontal = grect_inset(r, GEdgeInsets(10, 20));
! GRect r_expand_top_right_shrink_bottom_left = grect_inset(r, GEdgeInsets(-10, -10, 10, 10));
! \endcode*/
/** grect_inset
  -> Added in 73
gneiss: Vtable Slot 580, Offset 2320*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.grect_inset")]
pub unsafe extern "C" fn grect_inset(rect: GRect, insets: GEdgeInsets) -> GRect {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2320", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the current stroke color of the graphics context.
! @param ctx The graphics context onto which to set the stroke color
! @param color The new stroke color*/
/** graphics_context_set_stroke_color
  -> Added in 30
gneiss: Vtable Slot 372, Offset 1488*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_context_set_stroke_color")]
pub unsafe extern "C" fn graphics_context_set_stroke_color(ctx: *mut GContext, color: GColor) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1488", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the current fill color of the graphics context.
! @param ctx The graphics context onto which to set the fill color
! @param color The new fill color*/
/** graphics_context_set_fill_color
  -> Added in 30
gneiss: Vtable Slot 371, Offset 1484*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_context_set_fill_color")]
pub unsafe extern "C" fn graphics_context_set_fill_color(ctx: *mut GContext, color: GColor) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1484", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the current text color of the graphics context.
! @param ctx The graphics context onto which to set the text color
! @param color The new text color*/
/** graphics_context_set_text_color
  -> Added in 30
gneiss: Vtable Slot 373, Offset 1492*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_context_set_text_color")]
pub unsafe extern "C" fn graphics_context_set_text_color(ctx: *mut GContext, color: GColor) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1492", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the current bitmap compositing mode of the graphics context.
! The default mode is GCompOpAssign i.e. bitmap transparency disabled.
! @param ctx The graphics context onto which to set the compositing mode
! @param mode The new compositing mode
! @see \ref GCompOp
! @see \ref bitmap_layer_set_compositing_mode()
! @note At the moment, this only affects the bitmaps drawing operations
! -- \ref graphics_draw_bitmap_in_rect(), \ref graphics_draw_rotated_bitmap, and
! anything that uses those APIs --, but it currently does not affect the filling or stroking
! operations.*/
/** graphics_context_set_compositing_mode
  -> Added in 0
gneiss: Vtable Slot 112, Offset 448*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_context_set_compositing_mode")]
pub unsafe extern "C" fn graphics_context_set_compositing_mode(ctx: *mut GContext, mode: GCompOp) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =448", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets whether antialiasing is applied to stroke drawing
! @param ctx The graphics context onto which to set the antialiasing
! @param enable True = antialiasing enabled, False = antialiasing disabled
! @note Default value is true.*/
/** graphics_context_set_antialiased
  -> Added in 41
gneiss: Vtable Slot 444, Offset 1776*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_context_set_antialiased")]
pub unsafe extern "C" fn graphics_context_set_antialiased(ctx: *mut GContext, enable: bool) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1776", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the width of the stroke for drawing routines
! @param ctx The graphics context onto which to set the stroke width
! @param stroke_width Width in pixels of the stroke.
! @note If stroke width of zero is passed, it will be ignored and will not change the value
! stored in GContext. Currently, only odd stroke_width values are supported. If an even value
! is passed in, the value will be stored as is, but the drawing routines will round down to the
! previous integral value when drawing. Default value is 1.*/
/** graphics_context_set_stroke_width
  -> Added in 41
gneiss: Vtable Slot 445, Offset 1780*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_context_set_stroke_width")]
pub unsafe extern "C" fn graphics_context_set_stroke_width(ctx: *mut GContext, stroke_width: u8) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1780", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! No corners
pub const GCornerMask_GCornerNone: GCornerMask = 0;
///! Top-Left corner
pub const GCornerMask_GCornerTopLeft: GCornerMask = 1;
///! Top-Right corner
pub const GCornerMask_GCornerTopRight: GCornerMask = 2;
///! Bottom-Left corner
pub const GCornerMask_GCornerBottomLeft: GCornerMask = 4;
///! Bottom-Right corner
pub const GCornerMask_GCornerBottomRight: GCornerMask = 8;
///! All corners
pub const GCornerMask_GCornersAll: GCornerMask = 15;
///! Top corners
pub const GCornerMask_GCornersTop: GCornerMask = 3;
///! Bottom corners
pub const GCornerMask_GCornersBottom: GCornerMask = 12;
///! Left corners
pub const GCornerMask_GCornersLeft: GCornerMask = 5;
///! Right corners
pub const GCornerMask_GCornersRight: GCornerMask = 10;
/**! Bit mask values to specify the corners of a rectangle.
! The values can be combines using binary OR (`|`),
! For example: the mask to indicate top left and bottom right corners can:
! be created as follows: `(GCornerTopLeft | GCornerBottomRight)`*/
pub type GCornerMask = ::core::ffi::c_uchar;
/**! Draws a pixel at given point in the current stroke color
! @param ctx The destination graphics context in which to draw
! @param point The point at which to draw the pixel*/
/** graphics_draw_pixel
  -> Added in 0
gneiss: Vtable Slot 119, Offset 476*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_draw_pixel")]
pub unsafe extern "C" fn graphics_draw_pixel(ctx: *mut GContext, point: GPoint) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =476", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Draws line in the current stroke color, current stroke width and AA flag
! @param ctx The destination graphics context in which to draw
! @param p0 The starting point of the line
! @param p1 The ending point of the line*/
/** graphics_draw_line
  -> Added in 0
gneiss: Vtable Slot 118, Offset 472*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_draw_line")]
pub unsafe extern "C" fn graphics_draw_line(ctx: *mut GContext, p0: GPoint, p1: GPoint) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =472", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Draws a 1-pixel wide rectangle outline in the current stroke color
! @param ctx The destination graphics context in which to draw
! @param rect The rectangle for which to draw the outline*/
/** graphics_draw_rect
  -> Added in 0
gneiss: Vtable Slot 120, Offset 480*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_draw_rect")]
pub unsafe extern "C" fn graphics_draw_rect(ctx: *mut GContext, rect: GRect) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =480", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Fills a rectangle with the current fill color, optionally rounding all or a
! selection of its corners.
! @param ctx The destination graphics context in which to draw
! @param rect The rectangle to fill
! @param corner_radius The rounding radius of the corners in pixels (maximum is 8 pixels)
! @param corner_mask Bitmask of the corners that need to be rounded.
! @see \ref GCornerMask*/
/** graphics_fill_rect
  -> Added in 0
gneiss: Vtable Slot 123, Offset 492*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_fill_rect")]
pub unsafe extern "C" fn graphics_fill_rect(
	ctx: *mut GContext,
	rect: GRect,
	corner_radius: u16,
	corner_mask: GCornerMask,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =492", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Draws the outline of a circle in the current stroke color
! @param ctx The destination graphics context in which to draw
! @param p The center point of the circle
! @param radius The radius in pixels*/
/** graphics_draw_circle
  -> Added in 0
gneiss: Vtable Slot 117, Offset 468*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_draw_circle")]
pub unsafe extern "C" fn graphics_draw_circle(ctx: *mut GContext, p: GPoint, radius: u16) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =468", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Fills a circle in the current fill color
! @param ctx The destination graphics context in which to draw
! @param p The center point of the circle
! @param radius The radius in pixels*/
/** graphics_fill_circle
  -> Added in 0
gneiss: Vtable Slot 122, Offset 488*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_fill_circle")]
pub unsafe extern "C" fn graphics_fill_circle(ctx: *mut GContext, p: GPoint, radius: u16) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =488", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Draws the outline of a rounded rectangle in the current stroke color
! @param ctx The destination graphics context in which to draw
! @param rect The rectangle defining the dimensions of the rounded rectangle to draw
! @param radius The corner radius in pixels*/
/** graphics_draw_round_rect
  -> Added in 0
gneiss: Vtable Slot 121, Offset 484*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_draw_round_rect")]
pub unsafe extern "C" fn graphics_draw_round_rect(ctx: *mut GContext, rect: GRect, radius: u16) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =484", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Draws a bitmap into the graphics context, inside the specified rectangle
! @param ctx The destination graphics context in which to draw the bitmap
! @param bitmap The bitmap to draw
! @param rect The rectangle in which to draw the bitmap
! @note If the size of `rect` is smaller than the size of the bitmap,
! the bitmap will be clipped on right and bottom edges.
! If the size of `rect` is larger than the size of the bitmap,
! the bitmap will be tiled automatically in both horizontal and vertical
! directions, effectively drawing a repeating pattern.
! @see GBitmap
! @see GContext*/
/** graphics_draw_bitmap_in_rect
  -> Added in 0
gneiss: Vtable Slot 116, Offset 464*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_draw_bitmap_in_rect")]
pub unsafe extern "C" fn graphics_draw_bitmap_in_rect(
	ctx: *mut GContext,
	bitmap: *const GBitmap,
	rect: GRect,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =464", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! A shortcut to capture the framebuffer in the native format of the watch.
! @see graphics_capture_frame_buffer_format*/
/** graphics_capture_frame_buffer
  -> Added in 32
gneiss: Vtable Slot 394, Offset 1576*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_capture_frame_buffer")]
pub unsafe extern "C" fn graphics_capture_frame_buffer(ctx: *mut GContext) -> *mut GBitmap {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1576", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Captures the frame buffer for direct access, using the given format.
! Graphics functions will not affect the frame buffer while it is captured.
! The frame buffer is released when {@link graphics_release_frame_buffer} is called.
! The frame buffer must be released before the end of a layer's `.update_proc`
! for the layer to be drawn properly.
!
! While the frame buffer is captured calling {@link graphics_capture_frame_buffer}
! will fail and return `NULL`.
! @note When writing to the frame buffer, you should respect the visible boundaries of a
! window on the screen. Use layer_get_frame(window_get_root_layer(window)).origin to obtain its
! position relative to the frame buffer. For example, drawing to (5, 5) in the frame buffer
! while the window is transitioning to the left with its origin at (-20, 0) would
! effectively draw that point at (25, 5) relative to the window. For this reason you should
! consider the window's root layer frame when calculating drawing coordinates.
! @see GBitmap
! @see GBitmapFormat
! @see layer_get_frame
! @see window_get_root_layer
! @param ctx The graphics context providing the frame buffer
! @param format The format in which the framebuffer should be captured. Supported formats
! are \ref GBitmapFormat1Bit and \ref GBitmapFormat8Bit.
! @return A pointer to the frame buffer. `NULL` if failed.*/
/** graphics_capture_frame_buffer_format
  -> Added in 32
gneiss: Vtable Slot 395, Offset 1580*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_capture_frame_buffer_format")]
pub unsafe extern "C" fn graphics_capture_frame_buffer_format(
	ctx: *mut GContext,
	format: GBitmapFormat,
) -> *mut GBitmap {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1580", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Releases the frame buffer.
! Must be called before the end of a layer's `.update_proc` for the layer to be drawn properly.
!
! If `buffer` does not point to the address previously returned by
! {@link graphics_capture_frame_buffer} the frame buffer will not be released.
! @param ctx The graphics context providing the frame buffer
! @param buffer The pointer to frame buffer
! @return True if the frame buffer was released successfully*/
/** graphics_release_frame_buffer
  -> Added in 23
gneiss: Vtable Slot 350, Offset 1400*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_release_frame_buffer")]
pub unsafe extern "C" fn graphics_release_frame_buffer(
	ctx: *mut GContext,
	buffer: *mut GBitmap,
) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1400", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Whether or not the frame buffer has been captured by {@link graphics_capture_frame_buffer}.
! Graphics functions will not affect the frame buffer until it has been released by
! {@link graphics_release_frame_buffer}.
! @param ctx The graphics context providing the frame buffer
! @return True if the frame buffer has been captured*/
/** graphics_frame_buffer_is_captured
  -> Added in 23
gneiss: Vtable Slot 349, Offset 1396*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_frame_buffer_is_captured")]
pub unsafe extern "C" fn graphics_frame_buffer_is_captured(ctx: *mut GContext) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1396", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Draws a rotated bitmap with a memory-sensitive 2x anti-aliasing technique
! (using ray-finding instead of super-sampling), which is thresholded into a b/w bitmap for 1-bit
! and color blended for 8-bit.
! @note This API has performance limitations that can degrade user experience. Use sparingly.
! @param ctx The destination graphics context in which to draw
! @param src The source bitmap to draw
! @param src_ic Instance center (single point unaffected by rotation) relative to source bitmap
! @param rotation Angle of rotation. Rotation is an integer between 0 (no rotation)
! and TRIG_MAX_ANGLE (360 degree rotation). Use \ref DEG_TO_TRIGANGLE to easily convert degrees
! to the appropriate value.
! @param dest_ic Where to draw the instance center of the rotated bitmap in the context.*/
/** graphics_draw_rotated_bitmap
  -> Added in 45
gneiss: Vtable Slot 460, Offset 1840*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_draw_rotated_bitmap")]
pub unsafe extern "C" fn graphics_draw_rotated_bitmap(
	ctx: *mut GContext,
	src: *mut GBitmap,
	src_ic: GPoint,
	rotation: ::core::ffi::c_int,
	dest_ic: GPoint,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1840", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Places a circle at the center of the rectangle, with a diameter that matches
! the rectangle's shortest side.*/
pub const GOvalScaleMode_GOvalScaleModeFitCircle: GOvalScaleMode = 0;
/**! Places a circle at the center of the rectangle, with a diameter that matches
! the rectangle's longest side.
! The circle may overflow the bounds of the rectangle.*/
pub const GOvalScaleMode_GOvalScaleModeFillCircle: GOvalScaleMode = 1;
/**! Values to specify how a given rectangle should be used to derive an oval shape.
! @see \ref graphics_fill_radial_internal
! @see \ref graphics_draw_arc_internal
! @see \ref gpoint_from_polar_internal
! @see \ref grect_centered_from_polar*/
pub type GOvalScaleMode = ::core::ffi::c_uchar;
/**! Draws a line arc clockwise between `angle_start` and `angle_end`, where 0° is
! the top of the circle. If the difference between `angle_start` and `angle_end` is greater
! than 360°, a full circle will be drawn.
! @param ctx The destination graphics context in which to draw using the current
!        stroke color and antialiasing setting.
! @param rect The reference rectangle to derive the center point and radius (see scale_mode).
! @param scale_mode Determines how rect will be used to derive the center point and radius.
! @param angle_start Radial starting angle. Use \ref DEG_TO_TRIGANGLE to easily convert degrees
! to the appropriate value.
! @param angle_end Radial finishing angle. If smaller than `angle_start`, nothing will be drawn.*/
/** graphics_draw_arc
  -> Added in 74
gneiss: Vtable Slot 582, Offset 2328*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_draw_arc")]
pub unsafe extern "C" fn graphics_draw_arc(
	ctx: *mut GContext,
	rect: GRect,
	scale_mode: GOvalScaleMode,
	angle_start: i32,
	angle_end: i32,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2328", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Fills a circle clockwise between `angle_start` and `angle_end`, where 0° is
! the top of the circle. If the difference between `angle_start` and `angle_end` is greater
! than 360°, a full circle will be drawn and filled. If `angle_start` is greater than
! `angle_end` nothing will be drawn.
! @note A simple example is drawing a 'Pacman' shape, with a starting angle of -225°, and
! ending angle of 45°. By setting `inset_thickness` to a non-zero value (such as 30) this
! example will produce the letter C.
! @param ctx The destination graphics context in which to draw using the current
! fill color and antialiasing setting.
! @param rect The reference rectangle to derive the center point and radius (see scale).
! @param scale_mode Determines how rect will be used to derive the center point and radius.
! @param inset_thickness Describes how thick in pixels the radial will be drawn towards its
!        center measured from the outside.
! @param angle_start Radial starting angle. Use \ref DEG_TO_TRIGANGLE to easily convert degrees
! to the appropriate value.
! @param angle_end Radial finishing angle. If smaller than `angle_start`, nothing will be drawn.*/
/** graphics_fill_radial
  -> Added in 74
gneiss: Vtable Slot 583, Offset 2332*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_fill_radial")]
pub unsafe extern "C" fn graphics_fill_radial(
	ctx: *mut GContext,
	rect: GRect,
	scale_mode: GOvalScaleMode,
	inset_thickness: u16,
	angle_start: i32,
	angle_end: i32,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2332", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Calculates a GPoint located at the angle provided on the perimeter of a circle defined by the
! provided GRect.
! @param rect The reference rectangle to derive the center point and radius (see scale_mode).
! @param scale_mode Determines how rect will be used to derive the center point and radius.
! @param angle The angle at which the point on the circle's perimeter should be calculated.
! Use \ref DEG_TO_TRIGANGLE to easily convert degrees to the appropriate value.
! @return The point on the circle's perimeter.*/
/** gpoint_from_polar
  -> Added in 74
gneiss: Vtable Slot 581, Offset 2324*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gpoint_from_polar")]
pub unsafe extern "C" fn gpoint_from_polar(
	rect: GRect,
	scale_mode: GOvalScaleMode,
	angle: i32,
) -> GPoint {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2324", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Calculates a rectangle centered on the perimeter of a circle at a given angle.
! Use this to construct rectangles that follow the perimeter of a circle as an input for
! \ref graphics_fill_radial_internal or \ref graphics_draw_arc_internal,
! e.g. to draw circles every 30 degrees on a watchface.
! @param rect The reference rectangle to derive the circle's center point and radius (see
!        scale_mode).
! @param scale_mode Determines how rect will be used to derive the circle's center point and
!        radius.
! @param angle The angle at which the point on the circle's perimeter should be calculated.
! Use \ref DEG_TO_TRIGANGLE to easily convert degrees to the appropriate value.
! @param size Width and height of the desired rectangle.
! @return The rectangle centered on the circle's perimeter.*/
/** grect_centered_from_polar
  -> Added in 74
gneiss: Vtable Slot 584, Offset 2336*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.grect_centered_from_polar")]
pub unsafe extern "C" fn grect_centered_from_polar(
	rect: GRect,
	scale_mode: GOvalScaleMode,
	angle: i32,
	size: GSize,
) -> GRect {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2336", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Draw commands are the basic building block of the draw command system, encoding the type of
! command to draw, the stroke width and color, fill color, and points that define the path (or
! center of a circle*/
#[repr(C)]
#[derive(Debug)]
pub struct GDrawCommand {
	_unused: [u8; 0],
}
/**! Draw command frames contain a list of commands to draw for that frame and a duration,
! indicating the length of time for which the frame should be drawn in an animation sequence.
! Frames form the building blocks of a \ref GDrawCommandSequence, which consists of multiple
! frames.*/
#[repr(C)]
#[derive(Debug)]
pub struct GDrawCommandFrame {
	_unused: [u8; 0],
}
/**! Draw command images contain a list of commands that can be drawn. An image can be loaded from
! PDC file data.*/
#[repr(C)]
#[derive(Debug)]
pub struct GDrawCommandImage {
	_unused: [u8; 0],
}
///! Draw command lists contain a list of commands that can be iterated over and drawn all at once
#[repr(C)]
#[derive(Debug)]
pub struct GDrawCommandList {
	_unused: [u8; 0],
}
/**! Callback for iterating over draw command list
! @param command current \ref GDrawCommand in iteration
! @param index index of the current command in the list
! @param context context pointer for the iteration operation
! @return true if the iteration should continue after this command is processed*/
pub type GDrawCommandListIteratorCb = ::core::option::Option<
	unsafe extern "C" fn(
		command: *mut GDrawCommand,
		index: u32,
		context: *mut ::core::ffi::c_void,
	) -> bool,
>;
/**! Draw command sequences allow the animation of frames over time. Each sequence has a list of
! frames that can be accessed by the elapsed duration of the animation (not maintained internally)
! or by index. Sequences can be loaded from PDC file data.*/
#[repr(C)]
#[derive(Debug)]
pub struct GDrawCommandSequence {
	_unused: [u8; 0],
}
///!< Invalid draw command type
pub const GDrawCommandType_GDrawCommandTypeInvalid: GDrawCommandType = 0;
///!< Arbitrary path draw command type
pub const GDrawCommandType_GDrawCommandTypePath: GDrawCommandType = 1;
///!< Circle draw command type
pub const GDrawCommandType_GDrawCommandTypeCircle: GDrawCommandType = 2;
///!< Arbitrary path drawn with sub-pixel precision (1/8th precision)
pub const GDrawCommandType_GDrawCommandTypePrecisePath: GDrawCommandType = 3;
pub type GDrawCommandType = ::core::ffi::c_uchar;
/**! Draw a command
! @param ctx The destination graphics context in which to draw
! @param command \ref GDrawCommand to draw*/
/** gdraw_command_draw
  -> Added in 48
gneiss: Vtable Slot 474, Offset 1896*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_draw")]
pub unsafe extern "C" fn gdraw_command_draw(ctx: *mut GContext, command: *mut GDrawCommand) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1896", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the command type
! @param command \ref GDrawCommand from which to get the type
! @return The type of the given \ref GDrawCommand*/
/** gdraw_command_get_type
  -> Added in 48
gneiss: Vtable Slot 486, Offset 1944*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_get_type")]
pub unsafe extern "C" fn gdraw_command_get_type(command: *mut GDrawCommand) -> GDrawCommandType {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1944", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set the fill color of a command
! @param command ref DrawCommand for which to set the fill color
! @param fill_color \ref GColor to set for the fill*/
/** gdraw_command_set_fill_color
  -> Added in 48
gneiss: Vtable Slot 509, Offset 2036*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_set_fill_color")]
pub unsafe extern "C" fn gdraw_command_set_fill_color(
	command: *mut GDrawCommand,
	fill_color: GColor,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2036", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the fill color of a command
! @param command \ref GDrawCommand from which to get the fill color
! @return fill color of the given \ref GDrawCommand*/
/** gdraw_command_get_fill_color
  -> Added in 48
gneiss: Vtable Slot 478, Offset 1912*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_get_fill_color")]
pub unsafe extern "C" fn gdraw_command_get_fill_color(command: *mut GDrawCommand) -> GColor {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1912", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set the stroke color of a command
! @param command \ref GDrawCommand for which to set the stroke color
! @param stroke_color \ref GColor to set for the stroke*/
/** gdraw_command_set_stroke_color
  -> Added in 48
gneiss: Vtable Slot 514, Offset 2056*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_set_stroke_color")]
pub unsafe extern "C" fn gdraw_command_set_stroke_color(
	command: *mut GDrawCommand,
	stroke_color: GColor,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2056", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the stroke color of a command
! @param command \ref GDrawCommand from which to get the stroke color
! @return The stroke color of the given \ref GDrawCommand*/
/** gdraw_command_get_stroke_color
  -> Added in 48
gneiss: Vtable Slot 484, Offset 1936*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_get_stroke_color")]
pub unsafe extern "C" fn gdraw_command_get_stroke_color(command: *mut GDrawCommand) -> GColor {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1936", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set the stroke width of a command
! @param command \ref GDrawCommand for which to set the stroke width
! @param stroke_width stroke width to set for the command*/
/** gdraw_command_set_stroke_width
  -> Added in 48
gneiss: Vtable Slot 515, Offset 2060*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_set_stroke_width")]
pub unsafe extern "C" fn gdraw_command_set_stroke_width(
	command: *mut GDrawCommand,
	stroke_width: u8,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2060", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the stroke width of a command
! @param command \ref GDrawCommand from which to get the stroke width
! @return The stroke width of the given \ref GDrawCommand*/
/** gdraw_command_get_stroke_width
  -> Added in 48
gneiss: Vtable Slot 485, Offset 1940*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_get_stroke_width")]
pub unsafe extern "C" fn gdraw_command_get_stroke_width(command: *mut GDrawCommand) -> u8 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1940", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Get the number of points in a command
/** gdraw_command_get_num_points
  -> Added in 48
gneiss: Vtable Slot 480, Offset 1920*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_get_num_points")]
pub unsafe extern "C" fn gdraw_command_get_num_points(command: *mut GDrawCommand) -> u16 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1920", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set the value of the point in a command at the specified index
! @param command \ref GDrawCommand for which to set the value of a point
! @param point_idx Index of the point to set the value for
! @param point new point value to set*/
/** gdraw_command_set_point
  -> Added in 48
gneiss: Vtable Slot 512, Offset 2048*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_set_point")]
pub unsafe extern "C" fn gdraw_command_set_point(
	command: *mut GDrawCommand,
	point_idx: u16,
	point: GPoint,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2048", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the value of a point in a command from the specified index
! @param command \ref GDrawCommand from which to get a point
! @param point_idx The index to get the point for
! @return The point in the \ref GDrawCommand specified by point_idx
! @note The index \b must be less than the number of points*/
/** gdraw_command_get_point
  -> Added in 48
gneiss: Vtable Slot 482, Offset 1928*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_get_point")]
pub unsafe extern "C" fn gdraw_command_get_point(
	command: *mut GDrawCommand,
	point_idx: u16,
) -> GPoint {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1928", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set the radius of a circle command
! @note This only works for commands of type \ref GDrawCommandTypeCircle
! @param command \ref GDrawCommand from which to set the circle radius
! @param radius The radius to set for the circle.*/
/** gdraw_command_set_radius
  -> Added in 48
gneiss: Vtable Slot 513, Offset 2052*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_set_radius")]
pub unsafe extern "C" fn gdraw_command_set_radius(command: *mut GDrawCommand, radius: u16) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2052", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the radius of a circle command.
! @note this only works for commands of type \ref GDrawCommandTypeCircle.
! @param command \ref GDrawCommand from which to get the circle radius
! @return The radius in pixels if command is of type \ref GDrawCommandTypeCircle*/
/** gdraw_command_get_radius
  -> Added in 48
gneiss: Vtable Slot 483, Offset 1932*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_get_radius")]
pub unsafe extern "C" fn gdraw_command_get_radius(command: *mut GDrawCommand) -> u16 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1932", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set the path of a stroke command to be open
! @note This only works for commands of type \ref GDrawCommandTypePath and
! \ref GDrawCommandTypePrecisePath
! @param command \ref GDrawCommand for which to set the path open status
! @param path_open true if path should be hidden*/
/** gdraw_command_set_path_open
  -> Added in 48
gneiss: Vtable Slot 511, Offset 2044*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_set_path_open")]
pub unsafe extern "C" fn gdraw_command_set_path_open(command: *mut GDrawCommand, path_open: bool) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2044", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Return whether a stroke command path is open
! @note This only works for commands of type \ref GDrawCommandTypePath and
! \ref GDrawCommandTypePrecisePath
! @param command \ref GDrawCommand from which to get the path open status
! @return true if the path is open*/
/** gdraw_command_get_path_open
  -> Added in 48
gneiss: Vtable Slot 481, Offset 1924*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_get_path_open")]
pub unsafe extern "C" fn gdraw_command_get_path_open(command: *mut GDrawCommand) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1924", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set a command as hidden. This command will not be drawn when \ref gdraw_command_draw is called
! with this command
! @param command \ref GDrawCommand for which to set the hidden status
! @param hidden true if command should be hidden*/
/** gdraw_command_set_hidden
  -> Added in 48
gneiss: Vtable Slot 510, Offset 2040*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_set_hidden")]
pub unsafe extern "C" fn gdraw_command_set_hidden(command: *mut GDrawCommand, hidden: bool) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2040", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Return whether a command is hidden
! @param command \ref GDrawCommand from which to get the hidden status
! @return true if command is hidden*/
/** gdraw_command_get_hidden
  -> Added in 48
gneiss: Vtable Slot 479, Offset 1916*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_get_hidden")]
pub unsafe extern "C" fn gdraw_command_get_hidden(command: *mut GDrawCommand) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1916", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Draw a frame
! @param ctx The destination graphics context in which to draw
! @param sequence The sequence from which the frame comes from (this is required)
! @param frame Frame to draw
! @param offset Offset from draw context origin to draw the frame*/
/** gdraw_command_frame_draw
  -> Added in 48
gneiss: Vtable Slot 475, Offset 1900*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_frame_draw")]
pub unsafe extern "C" fn gdraw_command_frame_draw(
	ctx: *mut GContext,
	sequence: *mut GDrawCommandSequence,
	frame: *mut GDrawCommandFrame,
	offset: GPoint,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1900", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set the duration of the frame
! @param frame \ref GDrawCommandFrame for which to set the duration
! @param duration duration of the frame in milliseconds*/
/** gdraw_command_frame_set_duration
  -> Added in 48
gneiss: Vtable Slot 477, Offset 1908*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_frame_set_duration")]
pub unsafe extern "C" fn gdraw_command_frame_set_duration(
	frame: *mut GDrawCommandFrame,
	duration: u32,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1908", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the duration of the frame
! @param frame \ref GDrawCommandFrame from which to get the duration
! @return duration of the frame in milliseconds*/
/** gdraw_command_frame_get_duration
  -> Added in 48
gneiss: Vtable Slot 476, Offset 1904*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_frame_get_duration")]
pub unsafe extern "C" fn gdraw_command_frame_get_duration(frame: *mut GDrawCommandFrame) -> u32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1904", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Creates a GDrawCommandImage from the specified resource (PDC file)
! @param resource_id Resource containing data to load and create GDrawCommandImage from.
! @return GDrawCommandImage pointer if the resource was loaded, NULL otherwise*/
/** gdraw_command_image_create_with_resource
  -> Added in 48
gneiss: Vtable Slot 488, Offset 1952*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_image_create_with_resource")]
pub unsafe extern "C" fn gdraw_command_image_create_with_resource(
	resource_id: u32,
) -> *mut GDrawCommandImage {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1952", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Creates a GDrawCommandImage as a copy from a given image
! @param image Image to copy.
! @return cloned image or NULL if the operation failed*/
/** gdraw_command_image_clone
  -> Added in 48
gneiss: Vtable Slot 487, Offset 1948*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_image_clone")]
pub unsafe extern "C" fn gdraw_command_image_clone(
	image: *mut GDrawCommandImage,
) -> *mut GDrawCommandImage {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1948", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Deletes the GDrawCommandImage structure and frees associated data
! @param image Pointer to the image to free (delete)*/
/** gdraw_command_image_destroy
  -> Added in 48
gneiss: Vtable Slot 489, Offset 1956*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_image_destroy")]
pub unsafe extern "C" fn gdraw_command_image_destroy(image: *mut GDrawCommandImage) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1956", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Draw an image
! @param ctx The destination graphics context in which to draw
! @param image Image to draw
! @param offset Offset from draw context origin to draw the image*/
/** gdraw_command_image_draw
  -> Added in 48
gneiss: Vtable Slot 490, Offset 1960*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_image_draw")]
pub unsafe extern "C" fn gdraw_command_image_draw(
	ctx: *mut GContext,
	image: *mut GDrawCommandImage,
	offset: GPoint,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1960", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get size of the bounding box surrounding all draw commands in the image. This bounding
! box can be used to set the graphics context or layer bounds when drawing the image.
! @param image \ref GDrawCommandImage from which to get the bounding box size
! @return bounding box size*/
/** gdraw_command_image_get_bounds_size
  -> Added in 48
gneiss: Vtable Slot 491, Offset 1964*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_image_get_bounds_size")]
pub unsafe extern "C" fn gdraw_command_image_get_bounds_size(
	image: *mut GDrawCommandImage,
) -> GSize {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1964", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set size of the bounding box surrounding all draw commands in the image. This bounding
! box can be used to set the graphics context or layer bounds when drawing the image.
! @param image \ref GDrawCommandImage for which to set the bounding box size
! @param size bounding box size*/
/** gdraw_command_image_set_bounds_size
  -> Added in 48
gneiss: Vtable Slot 493, Offset 1972*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_image_set_bounds_size")]
pub unsafe extern "C" fn gdraw_command_image_set_bounds_size(
	image: *mut GDrawCommandImage,
	size: GSize,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1972", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the command list of the image
! @param image \ref GDrawCommandImage from which to get the command list
! @return command list*/
/** gdraw_command_image_get_command_list
  -> Added in 48
gneiss: Vtable Slot 492, Offset 1968*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_image_get_command_list")]
pub unsafe extern "C" fn gdraw_command_image_get_command_list(
	image: *mut GDrawCommandImage,
) -> *mut GDrawCommandList {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1968", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Iterate over all commands in a command list
! @param command_list \ref GDrawCommandList over which to iterate
! @param handle_command iterator callback
! @param callback_context context pointer to be passed into the iterator callback*/
/** gdraw_command_list_iterate
  -> Added in 48
gneiss: Vtable Slot 497, Offset 1988*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_list_iterate")]
pub unsafe extern "C" fn gdraw_command_list_iterate(
	command_list: *mut GDrawCommandList,
	handle_command: GDrawCommandListIteratorCb,
	callback_context: *mut ::core::ffi::c_void,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1988", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Draw all commands in a command list
! @param ctx The destination graphics context in which to draw
! @param command_list list of commands to draw*/
/** gdraw_command_list_draw
  -> Added in 48
gneiss: Vtable Slot 494, Offset 1976*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_list_draw")]
pub unsafe extern "C" fn gdraw_command_list_draw(
	ctx: *mut GContext,
	command_list: *mut GDrawCommandList,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1976", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the command at the specified index
! @note the specified index must be less than the number of commands in the list
! @param command_list \ref GDrawCommandList from which to get a command
! @param command_idx index of the command to get
! @return pointer to \ref GDrawCommand at the specified index*/
/** gdraw_command_list_get_command
  -> Added in 48
gneiss: Vtable Slot 495, Offset 1980*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_list_get_command")]
pub unsafe extern "C" fn gdraw_command_list_get_command(
	command_list: *mut GDrawCommandList,
	command_idx: u16,
) -> *mut GDrawCommand {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1980", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the number of commands in the list
! @param command_list \ref GDrawCommandList from which to get the number of commands
! @return number of commands in command list*/
/** gdraw_command_list_get_num_commands
  -> Added in 48
gneiss: Vtable Slot 496, Offset 1984*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_list_get_num_commands")]
pub unsafe extern "C" fn gdraw_command_list_get_num_commands(
	command_list: *mut GDrawCommandList,
) -> u32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1984", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Creates a \ref GDrawCommandSequence from the specified resource (PDC file)
! @param resource_id Resource containing data to load and create GDrawCommandSequence from.
! @return GDrawCommandSequence pointer if the resource was loaded, NULL otherwise*/
/** gdraw_command_sequence_create_with_resource
  -> Added in 48
gneiss: Vtable Slot 499, Offset 1996*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_sequence_create_with_resource")]
pub unsafe extern "C" fn gdraw_command_sequence_create_with_resource(
	resource_id: u32,
) -> *mut GDrawCommandSequence {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1996", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Creates a \ref GDrawCommandSequence as a copy from a given sequence
! @param sequence Sequence to copy
! @return cloned sequence or NULL if the operation failed*/
/** gdraw_command_sequence_clone
  -> Added in 48
gneiss: Vtable Slot 498, Offset 1992*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_sequence_clone")]
pub unsafe extern "C" fn gdraw_command_sequence_clone(
	sequence: *mut GDrawCommandSequence,
) -> *mut GDrawCommandSequence {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1992", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Deletes the \ref GDrawCommandSequence structure and frees associated data
! @param sequence Pointer to the sequence to destroy*/
/** gdraw_command_sequence_destroy
  -> Added in 48
gneiss: Vtable Slot 500, Offset 2000*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_sequence_destroy")]
pub unsafe extern "C" fn gdraw_command_sequence_destroy(sequence: *mut GDrawCommandSequence) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2000", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the frame that should be shown after the specified amount of elapsed time
! The last frame will be returned if the elapsed time exceeds the total time
! @param sequence \ref GDrawCommandSequence from which to get the frame
! @param elapsed_ms elapsed time in milliseconds
! @return pointer to \ref GDrawCommandFrame that should be displayed at the elapsed time*/
/** gdraw_command_sequence_get_frame_by_elapsed
  -> Added in 48
gneiss: Vtable Slot 502, Offset 2008*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_sequence_get_frame_by_elapsed")]
pub unsafe extern "C" fn gdraw_command_sequence_get_frame_by_elapsed(
	sequence: *mut GDrawCommandSequence,
	elapsed_ms: u32,
) -> *mut GDrawCommandFrame {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2008", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the frame at the specified index
! @param sequence \ref GDrawCommandSequence from which to get the frame
! @param index Index of frame to get
! @return pointer to \ref GDrawCommandFrame at the specified index*/
/** gdraw_command_sequence_get_frame_by_index
  -> Added in 48
gneiss: Vtable Slot 503, Offset 2012*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_sequence_get_frame_by_index")]
pub unsafe extern "C" fn gdraw_command_sequence_get_frame_by_index(
	sequence: *mut GDrawCommandSequence,
	index: u32,
) -> *mut GDrawCommandFrame {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2012", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the size of the bounding box surrounding all draw commands in the sequence. This bounding
! box can be used to set the graphics context or layer bounds when drawing the frames in the
! sequence.
! @param sequence \ref GDrawCommandSequence from which to get the bounds
! @return bounding box size*/
/** gdraw_command_sequence_get_bounds_size
  -> Added in 48
gneiss: Vtable Slot 501, Offset 2004*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_sequence_get_bounds_size")]
pub unsafe extern "C" fn gdraw_command_sequence_get_bounds_size(
	sequence: *mut GDrawCommandSequence,
) -> GSize {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2004", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set size of the bounding box surrounding all draw commands in the sequence. This bounding
! box can be used to set the graphics context or layer bounds when drawing the frames in the
! sequence.
! @param sequence \ref GDrawCommandSequence for which to set the bounds
! @param size bounding box size*/
/** gdraw_command_sequence_set_bounds_size
  -> Added in 48
gneiss: Vtable Slot 507, Offset 2028*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_sequence_set_bounds_size")]
pub unsafe extern "C" fn gdraw_command_sequence_set_bounds_size(
	sequence: *mut GDrawCommandSequence,
	size: GSize,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2028", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the play count of the sequence
! @param sequence \ref GDrawCommandSequence from which to get the play count
! @return play count of sequence*/
/** gdraw_command_sequence_get_play_count
  -> Added in 48
gneiss: Vtable Slot 505, Offset 2020*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_sequence_get_play_count")]
pub unsafe extern "C" fn gdraw_command_sequence_get_play_count(
	sequence: *mut GDrawCommandSequence,
) -> u32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2020", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set the play count of the sequence
! @param sequence \ref GDrawCommandSequence for which to set the play count
! @param play_count play count*/
/** gdraw_command_sequence_set_play_count
  -> Added in 48
gneiss: Vtable Slot 508, Offset 2032*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_sequence_set_play_count")]
pub unsafe extern "C" fn gdraw_command_sequence_set_play_count(
	sequence: *mut GDrawCommandSequence,
	play_count: u32,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2032", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the total duration of the sequence.
! @param sequence \ref GDrawCommandSequence from which to get the total duration
! @return total duration of the sequence in milliseconds*/
/** gdraw_command_sequence_get_total_duration
  -> Added in 48
gneiss: Vtable Slot 506, Offset 2024*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_sequence_get_total_duration")]
pub unsafe extern "C" fn gdraw_command_sequence_get_total_duration(
	sequence: *mut GDrawCommandSequence,
) -> u32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2024", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the number of frames in the sequence
! @param sequence \ref GDrawCommandSequence from which to get the number of frames
! @return number of frames in the sequence*/
/** gdraw_command_sequence_get_num_frames
  -> Added in 48
gneiss: Vtable Slot 504, Offset 2016*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_sequence_get_num_frames")]
pub unsafe extern "C" fn gdraw_command_sequence_get_num_frames(
	sequence: *mut GDrawCommandSequence,
) -> u32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2016", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the command list of the frame
! @param frame \ref GDrawCommandFrame from which to get the command list
! @return command list*/
/** gdraw_command_frame_get_command_list
  -> Added in 81
gneiss: Vtable Slot 612, Offset 2448*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gdraw_command_frame_get_command_list")]
pub unsafe extern "C" fn gdraw_command_frame_get_command_list(
	frame: *mut GDrawCommandFrame,
) -> *mut GDrawCommandList {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2448", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Data structure describing a naked path
! @note Note that this data structure only refers to an array of points;
! the points are not stored inside this data structure itself.
! In most cases, one cannot use a stack-allocated array of GPoints. Instead
! one often needs to provide longer-lived (static or "global") storage for the points.*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GPathInfo {
	///! The number of points in the `points` array
	pub num_points: u32,
	///! Pointer to an array of points.
	pub points:     *mut GPoint,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of GPathInfo"][::core::mem::size_of::<GPathInfo>() - 8usize];
	["Alignment of GPathInfo"][::core::mem::align_of::<GPathInfo>() - 4usize];
	["Offset of field: GPathInfo::num_points"]
		[::core::mem::offset_of!(GPathInfo, num_points) - 0usize];
	["Offset of field: GPathInfo::points"][::core::mem::offset_of!(GPathInfo, points) - 4usize];
};
/**! Data structure describing a path, plus its rotation and translation.
! @note See the remark with \ref GPathInfo*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GPath {
	///! The number of points in the `points` array
	pub num_points: u32,
	///! Pointer to an array of points.
	pub points:     *mut GPoint,
	/**! The rotation that will be used when drawing the path with
	! \ref gpath_draw_filled() or \ref gpath_draw_outline()*/
	pub rotation:   i32,
	/**! The translation that will to be used when drawing the path with
	! \ref gpath_draw_filled() or \ref gpath_draw_outline()*/
	pub offset:     GPoint,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of GPath"][::core::mem::size_of::<GPath>() - 16usize];
	["Alignment of GPath"][::core::mem::align_of::<GPath>() - 4usize];
	["Offset of field: GPath::num_points"][::core::mem::offset_of!(GPath, num_points) - 0usize];
	["Offset of field: GPath::points"][::core::mem::offset_of!(GPath, points) - 4usize];
	["Offset of field: GPath::rotation"][::core::mem::offset_of!(GPath, rotation) - 8usize];
	["Offset of field: GPath::offset"][::core::mem::offset_of!(GPath, offset) - 12usize];
};
/**! Creates a new GPath on the heap based on a series of points described by a GPathInfo.
!
! Values after initialization:
! * `num_points` and `points` pointer: copied from the GPathInfo.
! * `rotation`: 0
! * `offset`: (0, 0)
! @return A pointer to the GPath. `NULL` if the GPath could not
! be created*/
/** gpath_create
  -> Added in 0
gneiss: Vtable Slot 105, Offset 420*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gpath_create")]
pub unsafe extern "C" fn gpath_create(init: *const GPathInfo) -> *mut GPath {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =420", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Free a dynamically allocated gpath created with \ref gpath_create()
/** gpath_destroy
  -> Added in 0
gneiss: Vtable Slot 106, Offset 424*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gpath_destroy")]
pub unsafe extern "C" fn gpath_destroy(gpath: *mut GPath) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =424", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Draws the fill of a path into a graphics context, using the current fill color,
! relative to the drawing area as set up by the layering system.
! @param ctx The graphics context to draw into
! @param path The path to fill
! @see \ref graphics_context_set_fill_color()*/
/** gpath_draw_filled
  -> Added in 20
gneiss: Vtable Slot 343, Offset 1372*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gpath_draw_filled")]
pub unsafe extern "C" fn gpath_draw_filled(ctx: *mut GContext, path: *mut GPath) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1372", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Draws the outline of a path into a graphics context, using the current stroke color and
! width, relative to the drawing area as set up by the layering system. The first and last points
! in the path do have a line between them.
! @param ctx The graphics context to draw into
! @param path The path to draw
! @see \ref graphics_context_set_stroke_color()
! @see \ref gpath_draw_outline_open()*/
/** gpath_draw_outline
  -> Added in 0
gneiss: Vtable Slot 108, Offset 432*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gpath_draw_outline")]
pub unsafe extern "C" fn gpath_draw_outline(ctx: *mut GContext, path: *mut GPath) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =432", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the absolute rotation of the path.
! The current rotation will be replaced by the specified angle.
! @param path The path onto which to set the rotation
! @param angle The absolute angle of the rotation. The angle is represented in the same way
! that is used with \ref sin_lookup(). See \ref TRIG_MAX_ANGLE for more information.
! @note Setting a rotation does not affect the points in the path directly.
! The rotation is applied on-the-fly during drawing, either using \ref gpath_draw_filled() or
! \ref gpath_draw_outline().*/
/** gpath_rotate_to
  -> Added in 0
gneiss: Vtable Slot 110, Offset 440*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gpath_rotate_to")]
pub unsafe extern "C" fn gpath_rotate_to(path: *mut GPath, angle: i32) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =440", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the absolute offset of the path.
! The current translation will be replaced by the specified offset.
! @param path The path onto which to set the translation
! @param point The point which is used as the vector for the translation.
! @note Setting a translation does not affect the points in the path directly.
! The translation is applied on-the-fly during drawing, either using \ref gpath_draw_filled() or
! \ref gpath_draw_outline().*/
/** gpath_move_to
  -> Added in 0
gneiss: Vtable Slot 109, Offset 436*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gpath_move_to")]
pub unsafe extern "C" fn gpath_move_to(path: *mut GPath, point: GPoint) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =436", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Draws an open outline of a path into a graphics context, using the current stroke color and
! width, relative to the drawing area as set up by the layering system. The first and last points
! in the path do not have a line between them.
! @param ctx The graphics context to draw into
! @param path The path to draw
! @see \ref graphics_context_set_stroke_color()
! @see \ref gpath_draw_outline()*/
/** gpath_draw_outline_open
  -> Added in 50
gneiss: Vtable Slot 518, Offset 2072*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gpath_draw_outline_open")]
pub unsafe extern "C" fn gpath_draw_outline_open(ctx: *mut GContext, path: *mut GPath) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2072", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @addtogroup Fonts
! @see \ref TextLayer
! @see \ref TextDrawing
! @see \ref text_layer_set_font
! @see \ref graphics_draw_text
! @{*/
#[repr(C)]
#[derive(Debug)]
pub struct FontInfo {
	_unused: [u8; 0],
}
/**! Pointer to opaque font data structure.
! @see \ref fonts_load_custom_font()
! @see \ref text_layer_set_font()
! @see \ref graphics_draw_text()*/
pub type GFont = *mut FontInfo;
/**! Loads a system font corresponding to the specified font key.
! @param font_key The string key of the font to load. See
! <a href="https://developer.pebble.com/guides/app-resources/system-fonts/">System
! Fonts</a> guide for a list of system fonts.
! @return An opaque pointer to the loaded font, or, a pointer to the default
! (fallback) font if the specified font cannot be loaded.
! @note This may load a font from the flash peripheral into RAM.*/
/** fonts_get_system_font
  -> Added in 0
gneiss: Vtable Slot 96, Offset 384*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.fonts_get_system_font")]
pub unsafe extern "C" fn fonts_get_system_font(font_key: *const ::core::ffi::c_char) -> GFont {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =384", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Loads a custom font.
! @param handle The resource handle of the font to load. See resource_ids.auto.h
! for a list of resource IDs, and use @c resource_get_handle() to obtain the resource handle.
! @return An opaque pointer to the loaded font, or a pointer to the default
! (fallback) font if the specified font cannot be loaded.
! @see Read the <a href="http://developer.getpebble.com/guides/pebble-apps/resources/">App
! Resources</a> guide on how to embed a font into your app.
! @note this may load a font from the flash peripheral into RAM.*/
/** fonts_load_custom_font
  -> Added in 0
gneiss: Vtable Slot 97, Offset 388*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.fonts_load_custom_font")]
pub unsafe extern "C" fn fonts_load_custom_font(handle: ResHandle) -> GFont {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =388", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Unloads the specified custom font and frees the memory that is occupied by
! it.
! @note When an application exits, the system automatically unloads all fonts
! that have been loaded.
! @param font The font to unload.*/
/** fonts_unload_custom_font
  -> Added in 0
gneiss: Vtable Slot 98, Offset 392*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.fonts_unload_custom_font")]
pub unsafe extern "C" fn fonts_unload_custom_font(font: GFont) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =392", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! On overflow, wrap words to a new line below the current one. Once vertical space is consumed,
! the last line may be clipped.*/
pub const GTextOverflowMode_GTextOverflowModeWordWrap: GTextOverflowMode = 0;
/**! On overflow, wrap words to a new line below the current one.
! Once vertical space is consumed, truncate as needed to fit a trailing ellipsis (...).
! Clipping may occur if the vertical space cannot accomodate the first line of text.*/
pub const GTextOverflowMode_GTextOverflowModeTrailingEllipsis: GTextOverflowMode = 1;
/**! Acts like \ref GTextOverflowModeTrailingEllipsis, plus trims leading and trailing newlines,
! while treating all other newlines as spaces.*/
pub const GTextOverflowMode_GTextOverflowModeFill: GTextOverflowMode = 2;
/**! Text overflow mode controls the way text overflows when the string that is drawn does not fit
! inside the area constraint.
! @see graphics_draw_text
! @see text_layer_set_overflow_mode*/
pub type GTextOverflowMode = ::core::ffi::c_uchar;
///! Aligns the text to the left of the drawing box
pub const GTextAlignment_GTextAlignmentLeft: GTextAlignment = 0;
///! Aligns the text centered inside the drawing box
pub const GTextAlignment_GTextAlignmentCenter: GTextAlignment = 1;
///! Aligns the text to the right of the drawing box
pub const GTextAlignment_GTextAlignmentRight: GTextAlignment = 2;
/**! Text aligment controls the way the text is aligned inside the box the text is drawn into.
! @see graphics_draw_text
! @see text_layer_set_text_alignment*/
pub type GTextAlignment = ::core::ffi::c_uchar;
#[repr(C)]
#[derive(Debug)]
pub struct GTextAttributes {
	_unused: [u8; 0],
}
/**! Creates an instance of GTextAttributes for advanced control when rendering text.
! @return New instance of GTextAttributes
! @see \ref graphics_draw_text*/
/** graphics_text_attributes_create
  -> Added in 75
gneiss: Vtable Slot 585, Offset 2340*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_text_attributes_create")]
pub unsafe extern "C" fn graphics_text_attributes_create() -> *mut GTextAttributes {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2340", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Destroys a previously created instance of GTextAttributes
/** graphics_text_attributes_destroy
  -> Added in 75
gneiss: Vtable Slot 586, Offset 2344*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_text_attributes_destroy")]
pub unsafe extern "C" fn graphics_text_attributes_destroy(text_attributes: *mut GTextAttributes) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2344", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Restores text flow to the rectangular default.
! @param text_attributes The attributes for which to disable text flow
! @see graphics_text_attributes_enable_screen_text_flow
! @see text_layer_restore_default_text_flow_and_paging*/
/** graphics_text_attributes_restore_default_text_flow
  -> Added in 75
gneiss: Vtable Slot 590, Offset 2360*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_text_attributes_restore_default_text_flow")]
pub unsafe extern "C" fn graphics_text_attributes_restore_default_text_flow(
	text_attributes: *mut GTextAttributes,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2360", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Enables text flow that follows the boundaries of the screen.
! @param text_attributes The attributes for which text flow should be enabled
! @param inset Additional amount of pixels to inset to the inside of the screen for text flow
! calculation. Can be zero.
! @see graphics_text_attributes_restore_default_text_flow
! @see text_layer_enable_screen_text_flow_and_paging*/
/** graphics_text_attributes_enable_screen_text_flow
  -> Added in 75
gneiss: Vtable Slot 588, Offset 2352*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_text_attributes_enable_screen_text_flow")]
pub unsafe extern "C" fn graphics_text_attributes_enable_screen_text_flow(
	text_attributes: *mut GTextAttributes,
	inset: u8,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2352", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Restores paging and locked content origin to the defaults.
! @param text_attributes The attributes for which to restore paging and locked content origin
! @see graphics_text_attributes_enable_paging
! @see text_layer_restore_default_text_flow_and_paging*/
/** graphics_text_attributes_restore_default_paging
  -> Added in 75
gneiss: Vtable Slot 589, Offset 2356*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_text_attributes_restore_default_paging")]
pub unsafe extern "C" fn graphics_text_attributes_restore_default_paging(
	text_attributes: *mut GTextAttributes,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2356", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Enables paging and locks the text flow calculation to a fixed point on the screen.
! @param text_attributes Attributes for which to enable paging and locked content origin
! @param content_origin_on_screen Absolute coordinate on the screen where the text content
!     starts before an animation or scrolling takes place. Usually the frame's origin of a layer
!     in screen coordinates.
! @param paging_on_screen Rectangle in absolute coordinates on the screen that describes where
!     text content pages. Usually the container's absolute frame in screen coordinates.
! @see graphics_text_attributes_restore_default_paging
! @see graphics_text_attributes_enable_screen_text_flow
! @see text_layer_enable_screen_text_flow_and_paging
! @see layer_convert_point_to_screen*/
/** graphics_text_attributes_enable_paging
  -> Added in 75
gneiss: Vtable Slot 587, Offset 2348*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_text_attributes_enable_paging")]
pub unsafe extern "C" fn graphics_text_attributes_enable_paging(
	text_attributes: *mut GTextAttributes,
	content_origin_on_screen: GPoint,
	paging_on_screen: GRect,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2348", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Draw text into the current graphics context, using the context's current text color.
! The text will be drawn inside a box with the specified dimensions and
! configuration, with clipping occuring automatically.
! @param ctx The destination graphics context in which to draw
! @param text The zero terminated UTF-8 string to draw
! @param font The font in which the text should be set
! @param box The bounding box in which to draw the text. The first line of text will be drawn
! against the top of the box.
! @param overflow_mode The overflow behavior, in case the text is larger than what fits inside
! the box.
! @param alignment The horizontal alignment of the text
! @param text_attributes Optional text attributes to describe the characteristics of the text*/
/** graphics_draw_text
  -> Added in 4
gneiss: Vtable Slot 309, Offset 1236*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_draw_text")]
pub unsafe extern "C" fn graphics_draw_text(
	ctx: *mut GContext,
	text: *const ::core::ffi::c_char,
	font: GFont,
	box_: GRect,
	overflow_mode: GTextOverflowMode,
	alignment: GTextAlignment,
	text_attributes: *mut GTextAttributes,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1236", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Obtain the maximum size that a text with given font, overflow mode and alignment occupies
! within a given rectangular constraint.
! @param text The zero terminated UTF-8 string for which to calculate the size
! @param font The font in which the text should be set while calculating the size
! @param box The bounding box in which the text should be constrained
! @param overflow_mode The overflow behavior, in case the text is larger than what fits
! inside the box.
! @param alignment The horizontal alignment of the text
! @return The maximum size occupied by the text
! @see app_graphics_text_layout_get_content_size_with_attributes*/
/** graphics_text_layout_get_content_size
  -> Added in 7
gneiss: Vtable Slot 315, Offset 1260*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_text_layout_get_content_size")]
pub unsafe extern "C" fn graphics_text_layout_get_content_size(
	text: *const ::core::ffi::c_char,
	font: GFont,
	box_: GRect,
	overflow_mode: GTextOverflowMode,
	alignment: GTextAlignment,
) -> GSize {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1260", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Obtain the maximum size that a text with given font, overflow mode and alignment occupies
! within a given rectangular constraint.
! @param text The zero terminated UTF-8 string for which to calculate the size
! @param font The font in which the text should be set while calculating the size
! @param box The bounding box in which the text should be constrained
! @param overflow_mode The overflow behavior, in case the text is larger than what fits
! inside the box.
! @param alignment The horizontal alignment of the text
! @param text_attributes Optional text attributes to describe the characteristics of the text
! @return The maximum size occupied by the text
! @see app_graphics_text_layout_get_content_size*/
/** graphics_text_layout_get_content_size_with_attributes
  -> Added in 75
gneiss: Vtable Slot 591, Offset 2364*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.graphics_text_layout_get_content_size_with_attributes")]
pub unsafe extern "C" fn graphics_text_layout_get_content_size_with_attributes(
	text: *const ::core::ffi::c_char,
	font: GFont,
	box_: GRect,
	overflow_mode: GTextOverflowMode,
	alignment: GTextAlignment,
	text_attributes: *mut GTextAttributes,
) -> GSize {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2364", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! No error occured.
pub const SmartstrapResult_SmartstrapResultOk: SmartstrapResult = 0;
///! Invalid function arguments were supplied.
pub const SmartstrapResult_SmartstrapResultInvalidArgs: SmartstrapResult = 1;
///! The smartstrap port is not present on this watch.
pub const SmartstrapResult_SmartstrapResultNotPresent: SmartstrapResult = 2;
///! A request is already pending on the specified attribute.
pub const SmartstrapResult_SmartstrapResultBusy: SmartstrapResult = 3;
/**! Either a smartstrap is not connected or the connected smartstrap does not support the
! specified service.*/
pub const SmartstrapResult_SmartstrapResultServiceUnavailable: SmartstrapResult = 4;
///! The smartstrap reported that it does not support the requested attribute.
pub const SmartstrapResult_SmartstrapResultAttributeUnsupported: SmartstrapResult = 5;
///! A time-out occured during the request.
pub const SmartstrapResult_SmartstrapResultTimeOut: SmartstrapResult = 6;
///! Error values which may be returned from the smartstrap APIs.
pub type SmartstrapResult = ::core::ffi::c_uchar;
///! A type representing a smartstrap ServiceId.
pub type SmartstrapServiceId = u16;
///! A type representing a smartstrap AttributeId.
pub type SmartstrapAttributeId = u16;
/**! A type representing an attribute of a service provided by a smartstrap. This type is used when
! issuing requests to the smartstrap.*/
#[repr(C)]
#[derive(Debug)]
pub struct SmartstrapAttribute {
	_unused: [u8; 0],
}
/**! The type of function which is called after the smartstrap connection status changes.
! @param service_id The ServiceId for which the availability changed.
! @param is_available Whether or not this service is now available.*/
pub type SmartstrapServiceAvailabilityHandler = ::core::option::Option<
	unsafe extern "C" fn(service_id: SmartstrapServiceId, is_available: bool),
>;
/**! The type of function which can be called when a read request is completed.
! @note Any write request made to the same attribute within this function will fail with
! SmartstrapResultBusy.
! @param attribute The attribute which was read.
! @param result The result of the read.
! @param data The data read from the smartstrap or NULL if the read was not successful.
! @param length The length of the data or 0 if the read was not successful.*/
pub type SmartstrapReadHandler = ::core::option::Option<
	unsafe extern "C" fn(
		attribute: *mut SmartstrapAttribute,
		result: SmartstrapResult,
		data: *const u8,
		length: usize,
	),
>;
/**! The type of function which can be called when a write request is completed.
! @param attribute The attribute which was written.
! @param result The result of the write.*/
pub type SmartstrapWriteHandler = ::core::option::Option<
	unsafe extern "C" fn(attribute: *mut SmartstrapAttribute, result: SmartstrapResult),
>;
/**! The type of function which can be called when the smartstrap sends a notification to the watch
! @param attribute The attribute which the notification came from.*/
pub type SmartstrapNotifyHandler =
	::core::option::Option<unsafe extern "C" fn(attribute: *mut SmartstrapAttribute)>;
///! Handlers which are passed to smartstrap_subscribe.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SmartstrapHandlers {
	///! The connection handler is called after the connection state changes.
	pub availability_did_change: SmartstrapServiceAvailabilityHandler,
	///! The read handler is called whenever a read is complete or the read times-out.
	pub did_read:                SmartstrapReadHandler,
	///! The did_write handler is called when a write has completed.
	pub did_write:               SmartstrapWriteHandler,
	///! The notified handler is called whenever a notification is received for an attribute.
	pub notified:                SmartstrapNotifyHandler,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of SmartstrapHandlers"][::core::mem::size_of::<SmartstrapHandlers>() - 16usize];
	["Alignment of SmartstrapHandlers"][::core::mem::align_of::<SmartstrapHandlers>() - 4usize];
	["Offset of field: SmartstrapHandlers::availability_did_change"]
		[::core::mem::offset_of!(SmartstrapHandlers, availability_did_change) - 0usize];
	["Offset of field: SmartstrapHandlers::did_read"]
		[::core::mem::offset_of!(SmartstrapHandlers, did_read) - 4usize];
	["Offset of field: SmartstrapHandlers::did_write"]
		[::core::mem::offset_of!(SmartstrapHandlers, did_write) - 8usize];
	["Offset of field: SmartstrapHandlers::notified"]
		[::core::mem::offset_of!(SmartstrapHandlers, notified) - 12usize];
};
/**! Subscribes handlers to be called after certain smartstrap events occur.
! @note Registering an availability_did_change handler will cause power to be applied to the
! smartstrap port and connection establishment to begin.
! @see smartstrap_unsubscribe
! @returns `SmartstrapResultNotPresent` if the watch does not have a smartstrap port or
! `SmartstrapResultOk` otherwise.*/
/** smartstrap_subscribe
  -> Added in 64
gneiss: Vtable Slot 564, Offset 2256*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.smartstrap_subscribe")]
pub unsafe extern "C" fn smartstrap_subscribe(handlers: SmartstrapHandlers) -> SmartstrapResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2256", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Unsubscribes the handlers. The handlers will no longer be called, but in-flight requests will
! otherwise be unaffected.
! @note If power was being applied to the smartstrap port and there are no attributes have been
! created (or they have all been destroyed), this will cause the smartstrap power to be turned
! off.*/
/** smartstrap_unsubscribe
  -> Added in 64
gneiss: Vtable Slot 565, Offset 2260*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.smartstrap_unsubscribe")]
pub unsafe extern "C" fn smartstrap_unsubscribe() {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2260", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Changes the value of the timeout which is used for smartstrap requests. This timeout is started
! after the request is completely sent to the smartstrap and will be canceled only if the entire
! response is received before it triggers. The new timeout value will take affect only for
! requests made after this API is called.
! @param timeout_ms The duration of the timeout to set, in milliseconds.
! @note The maximum allowed timeout is currently 1000ms. If a larger value is passed, it will be
! internally lowered to the maximum.
! @see SMARTSTRAP_TIMEOUT_DEFAULT*/
/** smartstrap_set_timeout
  -> Added in 64
gneiss: Vtable Slot 563, Offset 2252*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.smartstrap_set_timeout")]
pub unsafe extern "C" fn smartstrap_set_timeout(timeout_ms: u16) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2252", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Creates and returns a SmartstrapAttribute for the specified service and attribute. This API
! will allocate an internal buffer of the requested length on the app's heap.
! @note Creating an attribute will result in power being applied to the smartstrap port (if it
! isn't already) and connection establishment to begin.
! @param service_id The ServiceId to create the attribute for.
! @param attribute_id The AttributeId to create the attribute for.
! @param buffer_length The length of the internal buffer which will be used to store the read
! and write requests for this attribute.
! @returns The newly created SmartstrapAttribute or NULL if an internal error occured or if the
! specified length is greater than SMARTSTRAP_ATTRIBUTE_LENGTH_MAXIMUM.*/
/** smartstrap_attribute_create
  -> Added in 64
gneiss: Vtable Slot 556, Offset 2224*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.smartstrap_attribute_create")]
pub unsafe extern "C" fn smartstrap_attribute_create(
	service_id: SmartstrapServiceId,
	attribute_id: SmartstrapAttributeId,
	buffer_length: usize,
) -> *mut SmartstrapAttribute {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2224", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Destroys a SmartstrapAttribute. No further handlers will be called for this attribute and it
! may not be used for any future requests.
! @param[in] attribute The SmartstrapAttribute which should be destroyed.
! @note If power was being applied to the smartstrap port, no availability_did_change handler is
! subscribed, and the last attribute is being destroyed, this will cause the smartstrap power to
! be turned off.*/
/** smartstrap_attribute_destroy
  -> Added in 64
gneiss: Vtable Slot 557, Offset 2228*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.smartstrap_attribute_destroy")]
pub unsafe extern "C" fn smartstrap_attribute_destroy(attribute: *mut SmartstrapAttribute) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2228", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Checks whether or not the specified service is currently supported by a connected smartstrap.
! @param service_id The SmartstrapServiceId of the service to check for availability.
! @returns Whether or not the service is available.*/
/** smartstrap_service_is_available
  -> Added in 64
gneiss: Vtable Slot 562, Offset 2248*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.smartstrap_service_is_available")]
pub unsafe extern "C" fn smartstrap_service_is_available(service_id: SmartstrapServiceId) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2248", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Returns the ServiceId which the attribute was created for (see @c
! smartstrap_attribute_create).
! @param attribute The SmartstrapAttribute for which to obtain the service ID.
! @returns The SmartstrapServiceId which the attribute was created with.*/
/** smartstrap_attribute_get_service_id
  -> Added in 64
gneiss: Vtable Slot 560, Offset 2240*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.smartstrap_attribute_get_service_id")]
pub unsafe extern "C" fn smartstrap_attribute_get_service_id(
	attribute: *mut SmartstrapAttribute,
) -> SmartstrapServiceId {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2240", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the AttributeId which the attribute was created for (see @c smartstrap_attribute_create).
! @param attribute The SmartstrapAttribute for which to obtain the attribute ID.
! @returns The SmartstrapAttributeId which the attribute was created with.*/
/** smartstrap_attribute_get_attribute_id
  -> Added in 64
gneiss: Vtable Slot 559, Offset 2236*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.smartstrap_attribute_get_attribute_id")]
pub unsafe extern "C" fn smartstrap_attribute_get_attribute_id(
	attribute: *mut SmartstrapAttribute,
) -> SmartstrapAttributeId {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2236", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Performs a read request for the specified attribute. The `did_read` callback will be called when
! the response is received from the smartstrap or when an error occurs.
! @param attribute The attribute to be perform the read request on.
! @returns `SmartstrapResultOk` if the read operation was started. The `did_read` callback will
! be called once the read request has been completed.*/
/** smartstrap_attribute_read
  -> Added in 64
gneiss: Vtable Slot 561, Offset 2244*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.smartstrap_attribute_read")]
pub unsafe extern "C" fn smartstrap_attribute_read(
	attribute: *mut SmartstrapAttribute,
) -> SmartstrapResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2244", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Begins a write request for the specified attribute and returns a buffer into which the app
! should write the data before calling smartstrap_attribute_end_write.
! @note The buffer must not be used after smartstrap_attribute_end_write is called.
! @param[in] attribute The attribute to begin writing for.
! @param[out] buffer The buffer to write the data into.
! @param[out] buffer_length The length of the buffer in bytes.
! @returns `SmartstrapResultOk` if a write operation was started and the `buffer` and
! `buffer_length` parameters were set, or an error otherwise.*/
/** smartstrap_attribute_begin_write
  -> Added in 64
gneiss: Vtable Slot 555, Offset 2220*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.smartstrap_attribute_begin_write")]
pub unsafe extern "C" fn smartstrap_attribute_begin_write(
	attribute: *mut SmartstrapAttribute,
	buffer: *mut *mut u8,
	buffer_length: *mut usize,
) -> SmartstrapResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2220", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! This should be called by the app when it is done writing to the buffer provided by
! smartstrap_begin_write and the data is ready to be sent to the smartstrap.
! @param[in] attribute The attribute to begin writing for.
! @param write_length The length of the data to be written, in bytes.
! @param request_read Whether or not a read request on this attribute should be
! automatically triggered following a successful write request.
! @returns `SmartstrapResultOk` if a write operation was queued to be sent to the smartstrap. The
! `did_write` handler will be called when the request is written to the smartstrap, and if
! `request_read` was set to true, the `did_read` handler will be called when the read is complete.*/
/** smartstrap_attribute_end_write
  -> Added in 64
gneiss: Vtable Slot 558, Offset 2232*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.smartstrap_attribute_end_write")]
pub unsafe extern "C" fn smartstrap_attribute_end_write(
	attribute: *mut SmartstrapAttribute,
	write_length: usize,
	request_read: bool,
) -> SmartstrapResult {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2232", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Reference to opaque click recognizer
! When a \ref ClickHandler callback is called, the recognizer that fired the handler is passed in.
! @see \ref ClickHandler
! @see \ref click_number_of_clicks_counted()
! @see \ref click_recognizer_get_button_id()
! @see \ref click_recognizer_is_repeating()*/
pub type ClickRecognizerRef = *mut ::core::ffi::c_void;
/**! Function signature of the callback that handles a recognized click pattern
! @param recognizer The click recognizer that detected a "click" pattern
! @param context Pointer to application specified data (see \ref window_set_click_config_provider_with_context() and
! \ref window_set_click_context()). This defaults to the window.
! @see \ref ClickConfigProvider*/
pub type ClickHandler = ::core::option::Option<
	unsafe extern "C" fn(recognizer: ClickRecognizerRef, context: *mut ::core::ffi::c_void),
>;
/**! This callback is called every time the window becomes visible (and when you call \ref window_set_click_config_provider() if
! the window is already visible).
!
! Subscribe to click events using
!   \ref window_single_click_subscribe()
!   \ref window_single_repeating_click_subscribe()
!   \ref window_multi_click_subscribe()
!   \ref window_long_click_subscribe()
!   \ref window_raw_click_subscribe()
! These subscriptions will get used by the click recognizers of each of the 4 buttons.
! @param context Pointer to application specific data (see \ref window_set_click_config_provider_with_context()).*/
pub type ClickConfigProvider =
	::core::option::Option<unsafe extern "C" fn(context: *mut ::core::ffi::c_void)>;
/**! Gets the click count.
! You can use this inside a click handler implementation to get the click count for multi_click
! and (repeated) click events.
! @param recognizer The click recognizer for which to get the click count
! @return The number of consecutive clicks, and for auto-repeating the number of repetitions.*/
/** click_number_of_clicks_counted
  -> Added in 0
gneiss: Vtable Slot 66, Offset 264*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.click_number_of_clicks_counted")]
pub unsafe extern "C" fn click_number_of_clicks_counted(recognizer: ClickRecognizerRef) -> u8 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =264", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the button identifier.
! You can use this inside a click handler implementation to get the button id for the click event.
! @param recognizer The click recognizer for which to get the button id that caused the click event
! @return the ButtonId of the click recognizer*/
/** click_recognizer_get_button_id
  -> Added in 0
gneiss: Vtable Slot 67, Offset 268*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.click_recognizer_get_button_id")]
pub unsafe extern "C" fn click_recognizer_get_button_id(
	recognizer: ClickRecognizerRef,
) -> ButtonId {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =268", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Is this a repeating click.
! You can use this inside a click handler implementation to find out whether this is a repeating click or not.
! @param recognizer The click recognizer for which to find out whether this is a repeating click.
! @return true if this is a repeating click.*/
/** click_recognizer_is_repeating
  -> Added in 14
gneiss: Vtable Slot 325, Offset 1300*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.click_recognizer_is_repeating")]
pub unsafe extern "C" fn click_recognizer_is_repeating(recognizer: ClickRecognizerRef) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1300", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @addtogroup Layer Layers
! \brief User interface layers for displaying graphic components
!
! Layers are objects that can be displayed on a Pebble watchapp window, enabling users to see
! visual objects, like text or images. Each layer stores the information about its state
! necessary to draw or redraw the object that it represents and uses graphics routines along with
! this state to draw itself when asked. Layers can be used to display various graphics.
!
! Layers are the basic building blocks for your application UI. Layers can be nested inside each other.
! Every window has a root layer which is always the topmost layer.
! You provide a function that is called to draw the content of the layer when needed; or
! you can use standard layers that are provided by the system, such as text layer, image layer,
! menu layer, action bar layer, and so on.
!
! The Pebble layer hierarchy is the list of things that need to be drawn to the screen.
! Multiple layers can be arranged into a hierarchy. This enables ordering (front to back),
! layout and hierarchy. Through relative positioning, visual objects that are grouped together by
! adding them into the same layer can be moved all at once. This means that the child layers
! will move accordingly. If a parent layer has clipping enabled, all the children will be clipped
! to the frame of the parent.
!
! Pebble OS provides convenience layers with built-in logic for displaying different graphic
! components, like text and bitmap layers.
!
! Refer to the \htmlinclude UiFramework.html (chapter "Layers") for a conceptual overview
! of Layers and relevant code examples.
!
! The Modules listed here contain what can be thought of conceptually as subclasses of Layer. The
! listed types can be safely type-casted to `Layer` (or `Layer *` in case of a pointer).
! The `layer_...` functions can then be used with the data structures of these subclasses.
! <br/>For example, the following is legal:
! \code{.c}
! TextLayer *text_layer;
! ...
! layer_set_hidden((Layer *)text_layer, true);
! \endcode
! @{*/
#[repr(C)]
#[derive(Debug)]
pub struct Layer {
	_unused: [u8; 0],
}
/**! Function signature for a Layer's render callback (the name of the type
! is derived from the words 'update procedure').
! The system will call the `.update_proc` callback whenever the Layer needs
! to be rendered.
! @param layer The layer that needs to be rendered
! @param ctx The destination graphics context to draw into
! @see \ref Graphics
! @see \ref layer_set_update_proc()*/
pub type LayerUpdateProc =
	::core::option::Option<unsafe extern "C" fn(layer: *mut Layer, ctx: *mut GContext)>;
/**! Creates a layer on the heap and sets its frame and bounds.
! Default values:
! * `bounds` : origin (0, 0) and a size equal to the frame that is passed in.
! * `clips` : `true`
! * `hidden` : `false`
! * `update_proc` : `NULL` (draws nothing)
! @param frame The frame at which the layer should be initialized.
! @see \ref layer_set_frame()
! @see \ref layer_set_bounds()
! @return A pointer to the layer. `NULL` if the layer could not
! be created*/
/** layer_create
  -> Added in 0
gneiss: Vtable Slot 139, Offset 556*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_create")]
pub unsafe extern "C" fn layer_create(frame: GRect) -> *mut Layer {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =556", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Creates a layer on the heap with extra space for callback data, and set its frame andbounds.
! Default values:
! * `bounds` : origin (0, 0) and a size equal to the frame that is passed in.
! * `clips` : `true`
! * `hidden` : `false`
! * `update_proc` : `NULL` (draws nothing)
! @param frame The frame at which the layer should be initialized.
! @param data_size The size (in bytes) of memory to allocate for callback data.
! @see \ref layer_create()
! @see \ref layer_set_frame()
! @see \ref layer_set_bounds()
! @return A pointer to the layer. `NULL` if the layer could not be created*/
/** layer_create_with_data
  -> Added in 0
gneiss: Vtable Slot 140, Offset 560*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_create_with_data")]
pub unsafe extern "C" fn layer_create_with_data(frame: GRect, data_size: usize) -> *mut Layer {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =560", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Destroys a layer previously created by layer_create
/** layer_destroy
  -> Added in 0
gneiss: Vtable Slot 141, Offset 564*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_destroy")]
pub unsafe extern "C" fn layer_destroy(layer: *mut Layer) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =564", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Marks the complete layer as "dirty", awaiting to be asked by the system to redraw itself.
! Typically, this function is called whenever state has changed that affects what the layer
! is displaying.
! * The layer's `.update_proc` will not be called before this function returns,
! but will be called asynchronously, shortly.
! * Internally, a call to this function will schedule a re-render of the window that the
! layer belongs to. In effect, all layers in that window's layer hierarchy will be asked to redraw.
! * If an earlier re-render request is still pending, this function is a no-op.
! @param layer The layer to mark dirty*/
/** layer_mark_dirty
  -> Added in 0
gneiss: Vtable Slot 150, Offset 600*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_mark_dirty")]
pub unsafe extern "C" fn layer_mark_dirty(layer: *mut Layer) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =600", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the layer's render function.
! The system will call the `update_proc` automatically when the layer needs to redraw itself, see
! also \ref layer_mark_dirty().
! @param layer Pointer to the layer structure.
! @param update_proc Pointer to the function that will be called when the layer needs to be rendered.
! Typically, one performs a series of drawing commands in the implementation of the `update_proc`,
! see \ref Drawing, \ref PathDrawing and \ref TextDrawing.*/
/** layer_set_update_proc
  -> Added in 0
gneiss: Vtable Slot 157, Offset 628*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_set_update_proc")]
pub unsafe extern "C" fn layer_set_update_proc(layer: *mut Layer, update_proc: LayerUpdateProc) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =628", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the frame of the layer, which is it's bounding box relative to the coordinate
! system of its parent layer.
! The size of the layer's bounds will be extended automatically, so that the bounds
! cover the new frame.
! @param layer The layer for which to set the frame
! @param frame The new frame
! @see \ref layer_set_bounds()*/
/** layer_set_frame
  -> Added in 0
gneiss: Vtable Slot 155, Offset 620*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_set_frame")]
pub unsafe extern "C" fn layer_set_frame(layer: *mut Layer, frame: GRect) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =620", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the frame of the layer, which is it's bounding box relative to the coordinate
! system of its parent layer.
! If the frame has changed, \ref layer_mark_dirty() will be called automatically.
! @param layer The layer for which to get the frame
! @return The frame of the layer
! @see layer_set_frame*/
/** layer_get_frame
  -> Added in 0
gneiss: Vtable Slot 145, Offset 580*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_get_frame")]
pub unsafe extern "C" fn layer_get_frame(layer: *const Layer) -> GRect {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =580", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the bounds of the layer, which is it's bounding box relative to its frame.
! If the bounds has changed, \ref layer_mark_dirty() will be called automatically.
! @param layer The layer for which to set the bounds
! @param bounds The new bounds
! @see \ref layer_set_frame()*/
/** layer_set_bounds
  -> Added in 0
gneiss: Vtable Slot 153, Offset 612*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_set_bounds")]
pub unsafe extern "C" fn layer_set_bounds(layer: *mut Layer, bounds: GRect) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =612", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the bounds of the layer
! @param layer The layer for which to get the bounds
! @return The bounds of the layer
! @see layer_set_bounds*/
/** layer_get_bounds
  -> Added in 0
gneiss: Vtable Slot 142, Offset 568*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_get_bounds")]
pub unsafe extern "C" fn layer_get_bounds(layer: *const Layer) -> GRect {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =568", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the largest unobstructed bounds rectangle of a layer.
! @param layer The layer for which to get the unobstructed bounds.
! @return The unobstructed bounds of the layer.
! @see UnobstructedArea*/
/** layer_get_unobstructed_bounds
  -> Added in 83
gneiss: Vtable Slot 622, Offset 2488*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_get_unobstructed_bounds")]
pub unsafe extern "C" fn layer_get_unobstructed_bounds(layer: *const Layer) -> GRect {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2488", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Converts a point from the layer's local coordinate system to screen coordinates.
! @note If the layer isn't part of the view hierarchy the result is undefined.
! @param layer The view whose coordinate system will be used to convert the value to the screen.
! @param point A point specified in the local coordinate system (bounds) of the layer.
! @return The point converted to the coordinate system of the screen.*/
/** layer_convert_point_to_screen
  -> Added in 75
gneiss: Vtable Slot 592, Offset 2368*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_convert_point_to_screen")]
pub unsafe extern "C" fn layer_convert_point_to_screen(
	layer: *const Layer,
	point: GPoint,
) -> GPoint {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2368", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Converts a rectangle from the layer's local coordinate system to screen coordinates.
! @note If the layer isn't part of the view hierarchy the result is undefined.
! @param layer The view whose coordinate system will be used to convert the value to the screen.
! @param rect A rectangle specified in the local coordinate system (bounds) of the layer.
! @return The rectangle converted to the coordinate system of the screen.*/
/** layer_convert_rect_to_screen
  -> Added in 75
gneiss: Vtable Slot 593, Offset 2372*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_convert_rect_to_screen")]
pub unsafe extern "C" fn layer_convert_rect_to_screen(layer: *const Layer, rect: GRect) -> GRect {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2372", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @addtogroup Window
! \brief The basic building block of the user interface
!
! Windows are the top-level elements in the UI hierarchy and the basic building blocks for a Pebble
! UI. A single window is always displayed at a time on Pebble, with the exception of when animating
! from one window to the other, which, in that case, is managed by the window stack. You can stack
! windows on top of each other, but only the topmost window will be visible.
!
! Users wearing a Pebble typically interact with the content and media displayed in a window, clicking
! and pressing buttons on the watch, depending on what they see and wish to respond to in a window.
!
! Windows serve to display a hierarchy of layers on the screen and handle user input. When a window is
! visible, its root Layer (and all its child layers) are drawn onto the screen automatically.
!
! You need a window, which always fills the entire screen, to display images, text, and graphics in
! your Pebble app. A layer by itself doesn’t display on Pebble; it must be in the current window’s
! layer hierarchy to be visible.
!
! The Window Stack serves as the global manager of what window is presented and makes sure that input
! events are forwarded to the topmost window.
!
! Refer to the \htmlinclude UiFramework.html (chapter "Window") for a conceptual
! overview of Window, the Window Stack and relevant code examples.
! @{*/
#[repr(C)]
#[derive(Debug)]
pub struct Window {
	_unused: [u8; 0],
}
/**! Gets the window that the layer is currently attached to.
! @param layer The layer for which to get the window
! @return The window that this layer is currently attached to, or `NULL` if it has
! not been added to a window's layer hierarchy.
! @see \ref window_get_root_layer()
! @see \ref layer_add_child()*/
/** layer_get_window
  -> Added in 0
gneiss: Vtable Slot 147, Offset 588*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_get_window")]
pub unsafe extern "C" fn layer_get_window(layer: *const Layer) -> *mut Window {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =588", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Removes the layer from its current parent layer
! If removed successfully, the child's parent layer will be marked dirty
! automatically.
! @param child The layer to remove*/
/** layer_remove_from_parent
  -> Added in 0
gneiss: Vtable Slot 152, Offset 608*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_remove_from_parent")]
pub unsafe extern "C" fn layer_remove_from_parent(child: *mut Layer) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =608", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Removes child layers from given layer
! If removed successfully, the child's parent layer will be marked dirty
! automatically.
! @param parent The layer from which to remove all child layers*/
/** layer_remove_child_layers
  -> Added in 0
gneiss: Vtable Slot 151, Offset 604*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_remove_child_layers")]
pub unsafe extern "C" fn layer_remove_child_layers(parent: *mut Layer) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =604", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Adds the child layer to a given parent layer, making it appear
! in front of its parent and in front of any existing child layers
! of the parent.
! If the child layer was already part of a layer hierarchy, it will
! be removed from its old parent first.
! If added successfully, the parent (and children) will be marked dirty
! automatically.
! @param parent The layer to which to add the child layer
! @param child The layer to add to the parent layer*/
/** layer_add_child
  -> Added in 0
gneiss: Vtable Slot 138, Offset 552*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_add_child")]
pub unsafe extern "C" fn layer_add_child(parent: *mut Layer, child: *mut Layer) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =552", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Inserts the layer as a sibling behind another layer. If the layer to insert was
! already part of a layer hierarchy, it will be removed from its old parent first.
! The below_layer has to be a child of a parent layer,
! otherwise this function will be a noop.
! If inserted successfully, the parent (and children) will be marked dirty
! automatically.
! @param layer_to_insert The layer to insert into the hierarchy
! @param below_sibling_layer The layer that will be used as the sibling layer
! above which the insertion will take place*/
/** layer_insert_below_sibling
  -> Added in 0
gneiss: Vtable Slot 149, Offset 596*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_insert_below_sibling")]
pub unsafe extern "C" fn layer_insert_below_sibling(
	layer_to_insert: *mut Layer,
	below_sibling_layer: *mut Layer,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =596", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Inserts the layer as a sibling in front of another layer.
! The above_layer has to be a child of a parent layer,
! otherwise this function will be a noop.
! If inserted successfully, the parent (and children) will be marked dirty
! automatically.
! @param layer_to_insert The layer to insert into the hierarchy
! @param above_sibling_layer The layer that will be used as the sibling layer
! below which the insertion will take place*/
/** layer_insert_above_sibling
  -> Added in 0
gneiss: Vtable Slot 148, Offset 592*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_insert_above_sibling")]
pub unsafe extern "C" fn layer_insert_above_sibling(
	layer_to_insert: *mut Layer,
	above_sibling_layer: *mut Layer,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =592", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the visibility of the layer.
! If the visibility has changed, \ref layer_mark_dirty() will be called automatically
! on the parent layer.
! @param layer The layer for which to set the visibility
! @param hidden Supply `true` to make the layer hidden, or `false` to make it
! non-hidden.*/
/** layer_set_hidden
  -> Added in 0
gneiss: Vtable Slot 156, Offset 624*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_set_hidden")]
pub unsafe extern "C" fn layer_set_hidden(layer: *mut Layer, hidden: bool) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =624", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the visibility of the layer.
! @param layer The layer for which to get the visibility
! @return True if the layer is hidden, false if it is not hidden.*/
/** layer_get_hidden
  -> Added in 0
gneiss: Vtable Slot 146, Offset 584*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_get_hidden")]
pub unsafe extern "C" fn layer_get_hidden(layer: *const Layer) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =584", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets whether clipping is enabled for the layer. If enabled, whatever the layer _and
! its children_ will draw using their `.update_proc` callbacks, will be clipped by the
! this layer's frame.
! If the clipping has changed, \ref layer_mark_dirty() will be called automatically.
! @param layer The layer for which to set the clipping property
! @param clips Supply `true` to make the layer clip to its frame, or `false`
! to make it non-clipping.*/
/** layer_set_clips
  -> Added in 0
gneiss: Vtable Slot 154, Offset 616*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_set_clips")]
pub unsafe extern "C" fn layer_set_clips(layer: *mut Layer, clips: bool) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =616", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets whether clipping is enabled for the layer.  If enabled, whatever the layer _and
! its children_ will draw using their `.update_proc` callbacks, will be clipped by the
! this layer's frame.
! @param layer The layer for which to get the clipping property
! @return True if clipping is enabled for the layer, false if clipping is not enabled for
! the layer.*/
/** layer_get_clips
  -> Added in 0
gneiss: Vtable Slot 143, Offset 572*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_get_clips")]
pub unsafe extern "C" fn layer_get_clips(layer: *const Layer) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =572", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the data from a layer that has been created with an extra data region.
! @param layer The layer to get the data region from.
! @return A void pointer to the data region.*/
/** layer_get_data
  -> Added in 0
gneiss: Vtable Slot 144, Offset 576*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.layer_get_data")]
pub unsafe extern "C" fn layer_get_data(layer: *const Layer) -> *mut ::core::ffi::c_void {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =576", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @addtogroup Recognizer Gesture Recognizers
! \brief Low-level touch-gesture recognizers for building fully custom touch
! interactions on touch-capable hardware.
!
! A recognizer watches the raw touch stream for one gesture — a tap, a pan
! (single-axis drag) or a swipe — and calls a \ref RecognizerEventCb as the
! gesture progresses. Create one with a per-gesture constructor
! (\ref tap_recognizer_create, \ref pan_recognizer_create,
! \ref swipe_recognizer_create), attach it to a \ref Window, and read the
! current gesture data from the event callback. Recognizers are the building
! block for a custom scroll or drag.
!
! For an ordinary scrollable list you do not need recognizers at all: on touch
! hardware \ref MenuLayer already scrolls by touch. Reach for recognizers to
! build an interaction the built-in list does not give you — a custom menu, a
! drag, or acting on a raw tap or swipe. Note that a bare \ref ScrollLayer does
! not scroll by touch on its own; you drive it with a pan recognizer, as below.
!
! <h3>Usage example: scrolling a custom menu</h3>
! A \ref ScrollLayer holds your own content but does not scroll by touch by
! itself. Attach a vertical pan recognizer to the window and feed its delta into
! \ref scroll_layer_set_content_offset() to scroll the content by finger:
! \code{.c}
! static ScrollLayer *s_scroll;
! static int16_t s_base;  // content offset committed on Complete
!
! static void pan_handler(const Recognizer *recognizer, RecognizerEvent event) {
!   switch (event) {
!     case RecognizerEvent_Updated: {
!       // delta_since_start is (0, 0) at Start, so the content does not jump.
!       GPoint d = pan_recognizer_get_delta_since_start(recognizer);
!       scroll_layer_set_content_offset(s_scroll, GPoint(0, s_base + d.y), false);
!       break;
!     }
!     case RecognizerEvent_Completed:
!       s_base = scroll_layer_get_content_offset(s_scroll).y;  // commit
!       break;
!     case RecognizerEvent_Cancelled:
!       scroll_layer_set_content_offset(s_scroll, GPoint(0, s_base), true);  // roll back
!       break;
!     default:
!       break;
!   }
! }
!
! static void window_load(Window *window) {
!   Layer *root = window_get_root_layer(window);
!   s_scroll = scroll_layer_create(layer_get_bounds(root));
!   scroll_layer_set_content_size(s_scroll, GSize(layer_get_bounds(root).size.w,
!                                                 total_content_height));
!   // Add your custom row layers as children of s_scroll here.
!   layer_add_child(root, scroll_layer_get_layer(s_scroll));
!
!   // The window owns the recognizer and destroys it when the window unloads.
!   Recognizer *pan = pan_recognizer_create(pan_handler, NULL, PanAxis_Vertical);
!   window_attach_recognizer(window, pan);
! }
! \endcode
! While the global system recognizer set is live, attaching alone is not enough:
! a window opts out of it with window_set_touch_bridge_disabled() (added by the
! touch-bridge change) so its own recognizers receive the touch stream.
! @{*/
#[repr(C)]
#[derive(Debug)]
pub struct Recognizer {
	_unused: [u8; 0],
}
pub const RecognizerEventType_RecognizerEvent_Started: RecognizerEventType = 0;
pub const RecognizerEventType_RecognizerEvent_Updated: RecognizerEventType = 1;
pub const RecognizerEventType_RecognizerEvent_Completed: RecognizerEventType = 2;
pub const RecognizerEventType_RecognizerEvent_Cancelled: RecognizerEventType = 3;
pub type RecognizerEventType = ::core::ffi::c_uchar;
pub use self::RecognizerEventType as RecognizerEvent;
/**! User event callback. When a recognizer changes state to any state other than the failed state
! the user callback of this type will be called.
! @param recognizer recognizer affected by the event
! @param event_type event that occurred*/
pub type RecognizerEventCb = ::core::option::Option<
	unsafe extern "C" fn(recognizer: *const Recognizer, event_type: RecognizerEvent),
>;
/**! This function is called to determine whether \a recognizer should be evaluated simultaneously
! with \a simultaneous_with
! @param recognizer recognizer to be tested
! @param simultaneous_with recognizer with which \a recognizer would be evaluated simultaneously
! should this test pass
! @return true if recognizers should be evaluated simultaneously*/
pub type RecognizerSimultaneousWithCb = ::core::option::Option<
	unsafe extern "C" fn(
		recognizer: *const Recognizer,
		simultaneous_with: *const Recognizer,
	) -> bool,
>;
pub const PanAxis_PanAxis_Horizontal: PanAxis = 0;
pub const PanAxis_PanAxis_Vertical: PanAxis = 1;
/**! Axis to which a pan recognizer is locked. A pan is only recognized when the
! finger moves unambiguously along this axis.*/
pub type PanAxis = ::core::ffi::c_uchar;
pub const SwipeDirection_SwipeDirection_None: SwipeDirection = 0;
pub const SwipeDirection_SwipeDirection_Up: SwipeDirection = 1;
pub const SwipeDirection_SwipeDirection_Down: SwipeDirection = 2;
pub const SwipeDirection_SwipeDirection_Left: SwipeDirection = 4;
pub const SwipeDirection_SwipeDirection_Right: SwipeDirection = 8;
/**! Swipe direction, also used as a bitmask when configuring which directions a swipe recognizer
! accepts. Screen coordinates grow downward, so a positive y delta is a downward swipe.*/
pub type SwipeDirection = ::core::ffi::c_uchar;
/**! Function signature for a handler that deals with transition events of a window.
! @see WindowHandlers
! @see \ref window_set_window_handlers()*/
pub type WindowHandler = ::core::option::Option<unsafe extern "C" fn(window: *mut Window)>;
/**! WindowHandlers
! These handlers are called by the \ref WindowStack as windows get pushed on / popped.
! All these handlers use \ref WindowHandler as their function signature.
! @see \ref window_set_window_handlers()
! @see \ref WindowStack*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct WindowHandlers {
	/**! Called when the window is pushed to the screen when it's not loaded.
	! This is a good moment to do the layout of the window.*/
	pub load:      WindowHandler,
	/**! Called when the window comes on the screen (again). E.g. when
	! second-top-most window gets revealed (again) after popping the top-most
	! window, but also when the window is pushed for the first time. This is a
	! good moment to start timers related to the window, or reset the UI, etc.*/
	pub appear:    WindowHandler,
	/**! Called when the window leaves the screen, e.g. when another window
	! is pushed, or this window is popped. Good moment to stop timers related
	! to the window.*/
	pub disappear: WindowHandler,
	/**! Called when the window is deinited, but could be used in the future to
	! free resources bound to windows that are not on screen.*/
	pub unload:    WindowHandler,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of WindowHandlers"][::core::mem::size_of::<WindowHandlers>() - 16usize];
	["Alignment of WindowHandlers"][::core::mem::align_of::<WindowHandlers>() - 4usize];
	["Offset of field: WindowHandlers::load"]
		[::core::mem::offset_of!(WindowHandlers, load) - 0usize];
	["Offset of field: WindowHandlers::appear"]
		[::core::mem::offset_of!(WindowHandlers, appear) - 4usize];
	["Offset of field: WindowHandlers::disappear"]
		[::core::mem::offset_of!(WindowHandlers, disappear) - 8usize];
	["Offset of field: WindowHandlers::unload"]
		[::core::mem::offset_of!(WindowHandlers, unload) - 12usize];
};
/**! Creates a new Window on the heap and initalizes it with the default values.
!
! * Background color : `GColorWhite`
! * Root layer's `update_proc` : function that fills the window's background using `background_color`.
! * `click_config_provider` : `NULL`
! * `window_handlers` : all `NULL`
! @return A pointer to the window. `NULL` if the window could not
! be created*/
/** window_create
  -> Added in 0
gneiss: Vtable Slot 271, Offset 1084*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_create")]
pub unsafe extern "C" fn window_create() -> *mut Window {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1084", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Destroys a Window previously created by window_create.
/** window_destroy
  -> Added in 0
gneiss: Vtable Slot 272, Offset 1088*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_destroy")]
pub unsafe extern "C" fn window_destroy(window: *mut Window) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1088", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the click configuration provider callback function on the window.
! This will automatically setup the input handlers of the window as well to use
! the click recognizer subsystem.
! @param window The window for which to set the click config provider
! @param click_config_provider The callback that will be called to configure the click recognizers with the window
! @see Clicks
! @see ClickConfigProvider*/
/** window_set_click_config_provider
  -> Added in 0
gneiss: Vtable Slot 278, Offset 1112*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_set_click_config_provider")]
pub unsafe extern "C" fn window_set_click_config_provider(
	window: *mut Window,
	click_config_provider: ClickConfigProvider,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1112", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Same as window_set_click_config_provider(), but will assign a custom context pointer
! (instead of the window pointer) that will be passed into the ClickHandler click event handlers.
! @param window The window for which to set the click config provider
! @param click_config_provider The callback that will be called to configure the click recognizers with the window
! @param context Pointer to application specific data that will be passed to the click configuration provider callback (defaults to the window).
! @see Clicks
! @see window_set_click_config_provider*/
/** window_set_click_config_provider_with_context
  -> Added in 0
gneiss: Vtable Slot 279, Offset 1116*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_set_click_config_provider_with_context")]
pub unsafe extern "C" fn window_set_click_config_provider_with_context(
	window: *mut Window,
	click_config_provider: ClickConfigProvider,
	context: *mut ::core::ffi::c_void,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1116", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the current click configuration provider of the window.
! @param window The window for which to get the click config provider*/
/** window_get_click_config_provider
  -> Added in 0
gneiss: Vtable Slot 273, Offset 1092*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_get_click_config_provider")]
pub unsafe extern "C" fn window_get_click_config_provider(
	window: *const Window,
) -> ClickConfigProvider {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1092", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the current click configuration provider context of the window.
! @param window The window for which to get the click config provider context*/
/** window_get_click_config_context
  -> Added in 11
gneiss: Vtable Slot 321, Offset 1284*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_get_click_config_context")]
pub unsafe extern "C" fn window_get_click_config_context(
	window: *mut Window,
) -> *mut ::core::ffi::c_void {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1284", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the window handlers of the window.
! These handlers get called e.g. when the user enters or leaves the window.
! @param window The window for which to set the window handlers
! @param handlers The handlers for the specified window
! @see \ref WindowHandlers*/
/** window_set_window_handlers
  -> Added in 0
gneiss: Vtable Slot 282, Offset 1128*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_set_window_handlers")]
pub unsafe extern "C" fn window_set_window_handlers(window: *mut Window, handlers: WindowHandlers) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1128", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the root Layer of the window.
! The root layer is the layer at the bottom of the layer hierarchy for this window.
! It is the window's "canvas" if you will. By default, the root layer only draws
! a solid fill with the window's background color.
! @param window The window for which to get the root layer
! @return The window's root layer*/
/** window_get_root_layer
  -> Added in 0
gneiss: Vtable Slot 275, Offset 1100*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_get_root_layer")]
pub unsafe extern "C" fn window_get_root_layer(window: *const Window) -> *mut Layer {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1100", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the background color of the window, which is drawn automatically by the
! root layer of the window.
! @param window The window for which to set the background color
! @param background_color The new background color
! @see \ref window_get_root_layer()*/
/** window_set_background_color
  -> Added in 30
gneiss: Vtable Slot 377, Offset 1508*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_set_background_color")]
pub unsafe extern "C" fn window_set_background_color(
	window: *mut Window,
	background_color: GColor,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1508", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets whether the window has been loaded.
! If a window is loaded, its `.load` handler has been called (and the `.unload` handler
! has not been called since).
! @return true if the window is currently loaded or false if not.
! @param window The window to query its loaded status
! @see \ref WindowHandlers*/
/** window_is_loaded
  -> Added in 0
gneiss: Vtable Slot 276, Offset 1104*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_is_loaded")]
pub unsafe extern "C" fn window_is_loaded(window: *mut Window) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1104", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets a pointer to developer-supplied data that the window uses, to
! provide a means to access the data at later times in one of the window event handlers.
! @see window_get_user_data
! @param window The window for which to set the user data
! @param data A pointer to user data.*/
/** window_set_user_data
  -> Added in 2
gneiss: Vtable Slot 292, Offset 1168*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_set_user_data")]
pub unsafe extern "C" fn window_set_user_data(window: *mut Window, data: *mut ::core::ffi::c_void) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1168", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the pointer to developer-supplied data that was previously
! set using window_set_user_data().
! @see window_set_user_data
! @param window The window for which to get the user data*/
/** window_get_user_data
  -> Added in 2
gneiss: Vtable Slot 291, Offset 1164*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_get_user_data")]
pub unsafe extern "C" fn window_get_user_data(window: *const Window) -> *mut ::core::ffi::c_void {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1164", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Subscribe to single click events.
! @note Must be called from the \ref ClickConfigProvider.
! @note \ref window_single_click_subscribe() and \ref window_single_repeating_click_subscribe() conflict, and cannot both be used on the same button.
! @note When there is a multi_click and/or long_click setup, there will be a delay before the single click
! @param button_id The button events to subscribe to.
! @param handler The \ref ClickHandler to fire on this event.
! handler will get fired. On the other hand, when there is no multi_click nor long_click setup, the single click handler will fire directly on button down.
! @see ButtonId
! @see Clicks
! @see window_single_repeating_click_subscribe*/
/** window_single_click_subscribe
  -> Added in 3
gneiss: Vtable Slot 307, Offset 1228*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_single_click_subscribe")]
pub unsafe extern "C" fn window_single_click_subscribe(button_id: ButtonId, handler: ClickHandler) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1228", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Subscribe to single click event, with a repeat interval. A single click is detected every time "repeat_interval_ms" has been reached.
! @note Must be called from the \ref ClickConfigProvider.
! @note \ref window_single_click_subscribe() and \ref window_single_repeating_click_subscribe() conflict, and cannot both be used on the same button.
! @note The back button cannot be overridden with a repeating click.
! @param button_id The button events to subscribe to.
! @param repeat_interval_ms When holding down, how many milliseconds before the handler is fired again.
! A value of 0ms means "no repeat timer". The minimum is 30ms, and values below will be disregarded.
! If there is a long-click handler subscribed on this button, `repeat_interval_ms` will not be used.
! @param handler The \ref ClickHandler to fire on this event.
! @see window_single_click_subscribe*/
/** window_single_repeating_click_subscribe
  -> Added in 3
gneiss: Vtable Slot 308, Offset 1232*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_single_repeating_click_subscribe")]
pub unsafe extern "C" fn window_single_repeating_click_subscribe(
	button_id: ButtonId,
	repeat_interval_ms: u16,
	handler: ClickHandler,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1232", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Subscribe to multi click events.
! @note Must be called from the \ref ClickConfigProvider.
! @param button_id The button events to subscribe to.
! @param min_clicks Minimum number of clicks before handler is fired. Defaults to 2.
! @param max_clicks Maximum number of clicks after which the click counter is reset. A value of 0 means use "min" also as "max".
! @param timeout The delay after which a sequence of clicks is considered finished, and the click counter is reset. A value of 0 means to use the system default 300ms.
! @param last_click_only Defaults to false. When true, only the handler for the last multi-click is called.
! @param handler The \ref ClickHandler to fire on this event. Fired for multi-clicks, as "filtered" by the `last_click_only`, `min`, and `max` parameters.*/
/** window_multi_click_subscribe
  -> Added in 3
gneiss: Vtable Slot 304, Offset 1216*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_multi_click_subscribe")]
pub unsafe extern "C" fn window_multi_click_subscribe(
	button_id: ButtonId,
	min_clicks: u8,
	max_clicks: u8,
	timeout: u16,
	last_click_only: bool,
	handler: ClickHandler,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1216", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Subscribe to long click events.
! @note Must be called from the \ref ClickConfigProvider.
! @note The back button cannot be overridden with a long click.
! @param button_id The button events to subscribe to.
! @param delay_ms Milliseconds after which "handler" is fired. A value of 0 means to use the system default 500ms.
! @param down_handler The \ref ClickHandler to fire as soon as the button has been held for `delay_ms`. This may be NULL to have no down handler.
! @param up_handler The \ref ClickHandler to fire on the release of a long click. This may be NULL to have no up handler.*/
/** window_long_click_subscribe
  -> Added in 3
gneiss: Vtable Slot 303, Offset 1212*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_long_click_subscribe")]
pub unsafe extern "C" fn window_long_click_subscribe(
	button_id: ButtonId,
	delay_ms: u16,
	down_handler: ClickHandler,
	up_handler: ClickHandler,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1212", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Subscribe to raw click events.
! @note Must be called from within the \ref ClickConfigProvider.
! @note The back button cannot be overridden with a raw click.
! @param button_id The button events to subscribe to.
! @param down_handler The \ref ClickHandler to fire as soon as the button has been pressed. This may be NULL to have no down handler.
! @param up_handler The \ref ClickHandler to fire on the release of the button. This may be NULL to have no up handler.
! @param context If this context is not NULL, it will override the general context.*/
/** window_raw_click_subscribe
  -> Added in 3
gneiss: Vtable Slot 305, Offset 1220*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_raw_click_subscribe")]
pub unsafe extern "C" fn window_raw_click_subscribe(
	button_id: ButtonId,
	down_handler: ClickHandler,
	up_handler: ClickHandler,
	context: *mut ::core::ffi::c_void,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1220", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set the context that will be passed to handlers for the given button's events. By default the context passed to handlers
! is equal to the \ref ClickConfigProvider context (defaults to the window).
! @note Must be called from within the \ref ClickConfigProvider.
! @param button_id The button to set the context for.
! @param context Set the context that will be passed to handlers for the given button's events.*/
/** window_set_click_context
  -> Added in 3
gneiss: Vtable Slot 306, Offset 1224*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_set_click_context")]
pub unsafe extern "C" fn window_set_click_context(
	button_id: ButtonId,
	context: *mut ::core::ffi::c_void,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1224", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Pushes the given window on the window navigation stack,
! on top of the current topmost window of the app.
! @param window The window to push on top
! @param animated Pass in `true` to animate the push using a sliding animation,
! or `false` to skip the animation.*/
/** window_stack_push
  -> Added in 0
gneiss: Vtable Slot 287, Offset 1148*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_stack_push")]
pub unsafe extern "C" fn window_stack_push(window: *mut Window, animated: bool) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1148", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Pops the topmost window on the navigation stack
! @param animated See \ref window_stack_remove()
! @return The window that is popped, or NULL if there are no windows to pop.*/
/** window_stack_pop
  -> Added in 0
gneiss: Vtable Slot 285, Offset 1140*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_stack_pop")]
pub unsafe extern "C" fn window_stack_pop(animated: bool) -> *mut Window {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1140", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Pops all windows.
! See \ref window_stack_remove() for a description of the `animated` parameter and notes.*/
/** window_stack_pop_all
  -> Added in 0
gneiss: Vtable Slot 286, Offset 1144*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_stack_pop_all")]
pub unsafe extern "C" fn window_stack_pop_all(animated: bool) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1144", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Removes a given window from the window stack
! that belongs to the app task.
! @note If there are no windows for the app left on the stack, the app
! will be killed by the system, shortly. To avoid this, make sure
! to push another window shortly after or before removing the last window.
! @param window The window to remove. If the window is NULL or if it
! is not on the stack, this function is a no-op.
! @param animated Pass in `true` to animate the removal of the window using
! a side-to-side sliding animation to reveal the next window.
! This is only used in case the window happens to be on top of the window
! stack (thus visible).
! @return True if window was successfully removed, false otherwise.*/
/** window_stack_remove
  -> Added in 0
gneiss: Vtable Slot 288, Offset 1152*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_stack_remove")]
pub unsafe extern "C" fn window_stack_remove(window: *mut Window, animated: bool) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1152", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the topmost window on the stack that belongs to the app.
! @return The topmost window on the stack that belongs to the app or
! NULL if no app window could be found.*/
/** window_stack_get_top_window
  -> Added in 0
gneiss: Vtable Slot 284, Offset 1136*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_stack_get_top_window")]
pub unsafe extern "C" fn window_stack_get_top_window() -> *mut Window {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1136", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Checks if the window is on the window stack
! @param window The window to look for on the window stack
! @return true if the window is currently on the window stack.*/
/** window_stack_contains_window
  -> Added in 0
gneiss: Vtable Slot 283, Offset 1132*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.window_stack_contains_window")]
pub unsafe extern "C" fn window_stack_contains_window(window: *mut Window) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1132", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @addtogroup Animation
!   \brief Abstract framework to create arbitrary animations
!
! The Animation framework provides your Pebble app with an base layer to create arbitrary
! animations. The simplest way to work with animations is to use the layer frame
! \ref PropertyAnimation, which enables you to move a Layer around on the screen.
! Using animation_set_implementation(), you can implement a custom animation.
!
! Refer to the \htmlinclude UiFramework.html (chapter "Animation") for a conceptual overview
! of the animation framework and on how to write custom animations.
! @{*/
#[repr(C)]
#[derive(Debug)]
pub struct Animation {
	_unused: [u8; 0],
}
/**! The type used to represent how far an animation has progressed. This is passed to the
! animation's update handler*/
pub type AnimationProgress = i32;
///! Linear curve: the velocity is constant.
pub const AnimationCurve_AnimationCurveLinear: AnimationCurve = 0;
///! Bicubic ease-in: accelerate from zero velocity
pub const AnimationCurve_AnimationCurveEaseIn: AnimationCurve = 1;
///! Bicubic ease-in: decelerate to zero velocity
pub const AnimationCurve_AnimationCurveEaseOut: AnimationCurve = 2;
///! Bicubic ease-in-out: accelerate from zero velocity, decelerate to zero velocity
pub const AnimationCurve_AnimationCurveEaseInOut: AnimationCurve = 3;
///! Bicubic ease-in-out: accelerate from zero velocity, decelerate to zero velocity
pub const AnimationCurve_AnimationCurveDefault: AnimationCurve = 3;
///! Custom (user-provided) animation curve
pub const AnimationCurve_AnimationCurveCustomFunction: AnimationCurve = 4;
///! User-provided interpolation function
pub const AnimationCurve_AnimationCurveCustomInterpolationFunction: AnimationCurve = 5;
///! User-provided interpolation function
pub const AnimationCurve_AnimationCurve_Reserved1: AnimationCurve = 6;
///! User-provided interpolation function
pub const AnimationCurve_AnimationCurve_Reserved2: AnimationCurve = 7;
/**! Values that are used to indicate the different animation curves,
! which determine the speed at which the animated value(s) change(s).*/
pub type AnimationCurve = ::core::ffi::c_uchar;
/**! Creates a new Animation on the heap and initalizes it with the default values.
!
! * Duration: 250ms,
! * Curve: \ref AnimationCurveEaseInOut (ease-in-out),
! * Delay: 0ms,
! * Handlers: `{NULL, NULL}` (none),
! * Context: `NULL` (none),
! * Implementation: `NULL` (no implementation),
! * Scheduled: no
! @return A pointer to the animation. `NULL` if the animation could not
! be created*/
/** animation_create
  -> Added in 32
gneiss: Vtable Slot 380, Offset 1520*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_create")]
pub unsafe extern "C" fn animation_create() -> *mut Animation {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1520", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Destroys an Animation previously created by animation_create.
! @return true if successful, false on failure*/
/** animation_destroy
  -> Added in 32
gneiss: Vtable Slot 381, Offset 1524*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_destroy")]
pub unsafe extern "C" fn animation_destroy(animation: *mut Animation) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1524", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/** animation_clone
  -> Added in 36
gneiss: Vtable Slot 422, Offset 1688*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_clone")]
pub unsafe extern "C" fn animation_clone(from: *mut Animation) -> *mut Animation {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1688", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Create a new sequence animation from a list of 2 or more other animations. The returned
! animation owns the animations that were provided as arguments and no further write operations
! on those handles are allowed. The variable length argument list must be terminated with a NULL
! ptr
! @note the maximum number of animations that can be supplied to this method is 20
! @param animation_a the first required component animation
! @param animation_b the second required component animation
! @param animation_c either the third component, or NULL if only adding 2 components
! @return The newly created sequence animation*/
/** animation_sequence_create
  -> Added in 36
gneiss: Vtable Slot 428, Offset 1712*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_sequence_create")]
pub unsafe extern "C" fn animation_sequence_create(
	animation_a: *mut Animation,
	animation_b: *mut Animation,
	animation_c: *mut Animation,
	args: ...
) -> *mut Animation {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1712", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! An alternate form of animation_sequence_create() that accepts an array of other animations.
! @note the maximum number of elements allowed in animation_array is 256
! @param animation_array an array of component animations to include
! @param array_len the number of elements in the animation_array
! @return The newly created sequence animation*/
/** animation_sequence_create_from_array
  -> Added in 36
gneiss: Vtable Slot 429, Offset 1716*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_sequence_create_from_array")]
pub unsafe extern "C" fn animation_sequence_create_from_array(
	animation_array: *mut *mut Animation,
	array_len: u32,
) -> *mut Animation {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1716", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Create a new spawn animation from a list of 2 or more other animations. The returned
! animation owns the animations that were provided as arguments and no further write operations
! on those handles are allowed. The variable length argument list must be terminated with a NULL
! ptr
! @note the maximum number of animations that can be supplied to this method is 20
! @param animation_a the first required component animation
! @param animation_b the second required component animation
! @param animation_c either the third component, or NULL if only adding 2 components
! @return The newly created spawn animation or NULL on failure*/
/** animation_spawn_create
  -> Added in 36
gneiss: Vtable Slot 433, Offset 1732*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_spawn_create")]
pub unsafe extern "C" fn animation_spawn_create(
	animation_a: *mut Animation,
	animation_b: *mut Animation,
	animation_c: *mut Animation,
	args: ...
) -> *mut Animation {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1732", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! An alternate form of animation_spawn_create() that accepts an array of other animations.
! @note the maximum number of elements allowed in animation_array is 256
! @param animation_array an array of component animations to include
! @param array_len the number of elements in the animation_array
! @return The newly created spawn animation or NULL on failure*/
/** animation_spawn_create_from_array
  -> Added in 36
gneiss: Vtable Slot 434, Offset 1736*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_spawn_create_from_array")]
pub unsafe extern "C" fn animation_spawn_create_from_array(
	animation_array: *mut *mut Animation,
	array_len: u32,
) -> *mut Animation {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1736", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Seek to a specific location in the animation. Only forward seeking is allowed. Returns true
! if successful, false if the passed in seek location is invalid.
! @param animation the animation for which to set the elapsed.
! @param elapsed_ms the new elapsed time in milliseconds
! @return true if successful, false if the requested elapsed is invalid.*/
/** animation_set_elapsed
  -> Added in 36
gneiss: Vtable Slot 431, Offset 1724*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_set_elapsed")]
pub unsafe extern "C" fn animation_set_elapsed(animation: *mut Animation, elapsed_ms: u32) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1724", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the current location in the animation.
! @note The animation must be scheduled to get the elapsed time. If it is not schedule,
! this method will return false.
! @param animation The animation for which to fetch the elapsed.
! @param[out] elapsed_ms pointer to variable that will contain the elapsed time in milliseconds
! @return true if successful, false on failure*/
/** animation_get_elapsed
  -> Added in 36
gneiss: Vtable Slot 426, Offset 1704*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_get_elapsed")]
pub unsafe extern "C" fn animation_get_elapsed(
	animation: *mut Animation,
	elapsed_ms: *mut i32,
) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1704", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set an animation to run in reverse (or forward)
! @note Trying to set an attribute when an animation is immutable will return false (failure). An
! animation is immutable once it has been added to a sequence or spawn animation or has been
! scheduled.
! @param animation the animation to operate on
! @param reverse set to true to run in reverse, false to run forward
! @return true if successful, false on failure*/
/** animation_set_reverse
  -> Added in 36
gneiss: Vtable Slot 432, Offset 1728*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_set_reverse")]
pub unsafe extern "C" fn animation_set_reverse(animation: *mut Animation, reverse: bool) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1728", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the reverse setting of an animation
! @param animation The animation for which to get the setting
! @return the reverse setting*/
/** animation_get_reverse
  -> Added in 36
gneiss: Vtable Slot 427, Offset 1708*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_get_reverse")]
pub unsafe extern "C" fn animation_get_reverse(animation: *mut Animation) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1708", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set an animation to play N times. The default is 1.
! @note Trying to set an attribute when an animation is immutable will return false (failure). An
! animation is immutable once it has been added to a sequence or spawn animation or has been
! scheduled.
! @param animation the animation to set the play count of
! @param play_count number of times to play this animation. Set to ANIMATION_PLAY_COUNT_INFINITE
! to make an animation repeat indefinitely.
! @return true if successful, false on failure*/
/** animation_set_play_count
  -> Added in 36
gneiss: Vtable Slot 430, Offset 1720*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_set_play_count")]
pub unsafe extern "C" fn animation_set_play_count(
	animation: *mut Animation,
	play_count: u32,
) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1720", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the play count of an animation
! @param animation The animation for which to get the setting
! @return the play count*/
/** animation_get_play_count
  -> Added in 36
gneiss: Vtable Slot 425, Offset 1700*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_get_play_count")]
pub unsafe extern "C" fn animation_get_play_count(animation: *mut Animation) -> u32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1700", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the time in milliseconds that an animation takes from start to finish.
! @note Trying to set an attribute when an animation is immutable will return false (failure). An
! animation is immutable once it has been added to a sequence or spawn animation or has been
! scheduled.
! @param animation The animation for which to set the duration.
! @param duration_ms The duration in milliseconds of the animation. This excludes
! any optional delay as set using \ref animation_set_delay().
! @return true if successful, false on failure*/
/** animation_set_duration
  -> Added in 32
gneiss: Vtable Slot 388, Offset 1552*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_set_duration")]
pub unsafe extern "C" fn animation_set_duration(
	animation: *mut Animation,
	duration_ms: u32,
) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1552", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the static duration of an animation from start to end (ignoring how much has already
! played, if any).
! @param animation The animation for which to get the duration
! @param include_delay if true, include the delay time
! @param include_play_count if true, incorporate the play_count
! @return the duration, in milliseconds. This includes any optional delay a set using
! \ref animation_set_delay.*/
/** animation_get_duration
  -> Added in 36
gneiss: Vtable Slot 424, Offset 1696*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_get_duration")]
pub unsafe extern "C" fn animation_get_duration(
	animation: *mut Animation,
	include_delay: bool,
	include_play_count: bool,
) -> u32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1696", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets an optional delay for the animation.
! @note Trying to set an attribute when an animation is immutable will return false (failure). An
! animation is immutable once it has been added to a sequence or spawn animation or has been
! scheduled.
! @param animation The animation for which to set the delay.
! @param delay_ms The delay in milliseconds that the animation system should
! wait from the moment the animation is scheduled to starting the animation.
! @return true if successful, false on failure*/
/** animation_set_delay
  -> Added in 32
gneiss: Vtable Slot 387, Offset 1548*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_set_delay")]
pub unsafe extern "C" fn animation_set_delay(animation: *mut Animation, delay_ms: u32) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1548", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the delay of an animation in milliseconds
! @param animation The animation for which to get the setting
! @return the delay in milliseconds*/
/** animation_get_delay
  -> Added in 36
gneiss: Vtable Slot 423, Offset 1692*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_get_delay")]
pub unsafe extern "C" fn animation_get_delay(animation: *mut Animation) -> u32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1692", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the animation curve for the animation.
! @note Trying to set an attribute when an animation is immutable will return false (failure). An
! animation is immutable once it has been added to a sequence or spawn animation or has been
! scheduled.
! @param animation The animation for which to set the curve.
! @param curve The type of curve.
! @see AnimationCurve
! @return true if successful, false on failure*/
/** animation_set_curve
  -> Added in 32
gneiss: Vtable Slot 385, Offset 1540*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_set_curve")]
pub unsafe extern "C" fn animation_set_curve(
	animation: *mut Animation,
	curve: AnimationCurve,
) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1540", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the animation curve for the animation.
! @param animation The animation for which to get the curve.
! @return The type of curve.*/
/** animation_get_curve
  -> Added in 37
gneiss: Vtable Slot 435, Offset 1740*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_get_curve")]
pub unsafe extern "C" fn animation_get_curve(animation: *mut Animation) -> AnimationCurve {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1740", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! The function pointer type of a custom animation curve.
! @param linear_distance The linear normalized animation distance to be curved.
! @see animation_set_custom_curve*/
pub type AnimationCurveFunction = ::core::option::Option<
	unsafe extern "C" fn(linear_distance: AnimationProgress) -> AnimationProgress,
>;
/**! Sets a custom animation curve function.
! @note Trying to set an attribute when an animation is immutable will return false (failure). An
! animation is immutable once it has been added to a sequence or spawn animation or has been
! scheduled.
! @param animation The animation for which to set the curve.
! @param curve_function The custom animation curve function.
! @see AnimationCurveFunction
! @return true if successful, false on failure*/
/** animation_set_custom_curve
  -> Added in 32
gneiss: Vtable Slot 386, Offset 1544*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_set_custom_curve")]
pub unsafe extern "C" fn animation_set_custom_curve(
	animation: *mut Animation,
	curve_function: AnimationCurveFunction,
) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1544", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the custom animation curve function for the animation.
! @param animation The animation for which to get the curve.
! @return The custom animation curve function for the given animation. NULL if not set.*/
/** animation_get_custom_curve
  -> Added in 37
gneiss: Vtable Slot 436, Offset 1744*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_get_custom_curve")]
pub unsafe extern "C" fn animation_get_custom_curve(
	animation: *mut Animation,
) -> AnimationCurveFunction {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1744", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! The function pointer type of the handler that will be called when an animation is started,
! just before updating the first frame of the animation.
! @param animation The animation that was started.
! @param context The pointer to custom, application specific data, as set using
! \ref animation_set_handlers()
! @note This is called after any optional delay as set by \ref animation_set_delay() has expired.
! @see animation_set_handlers*/
pub type AnimationStartedHandler = ::core::option::Option<
	unsafe extern "C" fn(animation: *mut Animation, context: *mut ::core::ffi::c_void),
>;
/**! The function pointer type of the handler that will be called when the animation is stopped.
! @param animation The animation that was stopped.
! @param finished True if the animation was stopped because it was finished normally,
! or False if the animation was stopped prematurely, because it was unscheduled before finishing.
! @param context The pointer to custom, application specific data, as set using
! \ref animation_set_handlers()
! @see animation_set_handlers
! \note
! This animation (i.e.: the `animation` parameter) may be destroyed here.
! It is not recommended to unschedule or destroy a **different** Animation within this
! Animation's `stopped` handler.*/
pub type AnimationStoppedHandler = ::core::option::Option<
	unsafe extern "C" fn(
		animation: *mut Animation,
		finished: bool,
		context: *mut ::core::ffi::c_void,
	),
>;
/**! The handlers that will get called when an animation starts and stops.
! See documentation with the function pointer types for more information.
! @see animation_set_handlers*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct AnimationHandlers {
	///! The handler that will be called when an animation is started.
	pub started: AnimationStartedHandler,
	///! The handler that will be called when an animation is stopped.
	pub stopped: AnimationStoppedHandler,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of AnimationHandlers"][::core::mem::size_of::<AnimationHandlers>() - 8usize];
	["Alignment of AnimationHandlers"][::core::mem::align_of::<AnimationHandlers>() - 4usize];
	["Offset of field: AnimationHandlers::started"]
		[::core::mem::offset_of!(AnimationHandlers, started) - 0usize];
	["Offset of field: AnimationHandlers::stopped"]
		[::core::mem::offset_of!(AnimationHandlers, stopped) - 4usize];
};
/**! Sets the callbacks for the animation.
! Often an application needs to run code at the start or at the end of an animation.
! Using this function is possible to register callback functions with an animation,
! that will get called at the start and end of the animation.
! @note Trying to set an attribute when an animation is immutable will return false (failure). An
! animation is immutable once it has been added to a sequence or spawn animation or has been
! scheduled.
! @param animation The animation for which to set up the callbacks.
! @param callbacks The callbacks.
! @param context A pointer to application specific data, that will be passed as an argument by
! the animation subsystem when a callback is called.
! @return true if successful, false on failure*/
/** animation_set_handlers
  -> Added in 32
gneiss: Vtable Slot 389, Offset 1556*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_set_handlers")]
pub unsafe extern "C" fn animation_set_handlers(
	animation: *mut Animation,
	callbacks: AnimationHandlers,
	context: *mut ::core::ffi::c_void,
) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1556", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the application-specific callback context of the animation.
! This `void` pointer is passed as an argument when the animation system calls AnimationHandlers
! callbacks. The context pointer can be set to point to any application specific data using
! \ref animation_set_handlers().
! @param animation The animation.
! @see animation_set_handlers*/
/** animation_get_context
  -> Added in 32
gneiss: Vtable Slot 382, Offset 1528*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_get_context")]
pub unsafe extern "C" fn animation_get_context(
	animation: *mut Animation,
) -> *mut ::core::ffi::c_void {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1528", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Schedules the animation. Call this once after configuring an animation to get it to
! start running.
!
! If the animation's implementation has a `.setup` callback it will get called before
! this function returns.
!
! @note If the animation was already scheduled,
! it will first unschedule it and then re-schedule it again.
! Note that in that case, the animation's `.stopped` handler, the implementation's
! `.teardown` and `.setup` will get called, due to the unscheduling and scheduling.
! @param animation The animation to schedule.
! @see \ref animation_unschedule()
! @return true if successful, false on failure*/
/** animation_schedule
  -> Added in 32
gneiss: Vtable Slot 384, Offset 1536*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_schedule")]
pub unsafe extern "C" fn animation_schedule(animation: *mut Animation) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1536", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Unschedules the animation, which in effect stops the animation.
! @param animation The animation to unschedule.
! @note If the animation was not yet finished, unscheduling it will
! cause its `.stopped` handler to get called, with the "finished" argument set to false.
! @note If the animation is not scheduled or NULL, calling this routine is
! effectively a no-op
! @see \ref animation_schedule()
! @return true if successful, false on failure*/
/** animation_unschedule
  -> Added in 32
gneiss: Vtable Slot 391, Offset 1564*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_unschedule")]
pub unsafe extern "C" fn animation_unschedule(animation: *mut Animation) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1564", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Unschedules all animations of the application.
! @see animation_unschedule*/
/** animation_unschedule_all
  -> Added in 32
gneiss: Vtable Slot 392, Offset 1568*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_unschedule_all")]
pub unsafe extern "C" fn animation_unschedule_all() {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1568", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @return True if the animation was scheduled, or false if it was not.
! @note An animation will be scheduled when it is running and not finished yet.
! An animation that has finished is automatically unscheduled.
! For convenience, passing in a NULL animation argument will simply return false
! @param animation The animation for which to get its scheduled state.
! @see animation_schedule
! @see animation_unschedule*/
/** animation_is_scheduled
  -> Added in 32
gneiss: Vtable Slot 383, Offset 1532*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_is_scheduled")]
pub unsafe extern "C" fn animation_is_scheduled(animation: *mut Animation) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1532", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Pointer to function that (optionally) prepares the animation for running.
! This callback is called when the animation is added to the scheduler.
! @param animation The animation that needs to be set up.
! @see animation_schedule
! @see AnimationTeardownImplementation*/
pub type AnimationSetupImplementation =
	::core::option::Option<unsafe extern "C" fn(animation: *mut Animation)>;
/**! Pointer to function that updates the animation according to the given normalized progress.
! This callback will be called repeatedly by the animation scheduler whenever the animation needs
! to be updated.
! @param animation The animation that needs to update; gets passed in by the animation framework.
! @param progress The current normalized progress; gets passed in by the animation
! framework for each animation frame.
! The value \ref ANIMATION_NORMALIZED_MIN represents the start and \ref ANIMATION_NORMALIZED_MAX
! represents the end. Values outside this range (generated by a custom curve function) can be used
! to implement features like a bounce back effect, where the progress exceeds the desired final
! value before returning to complete the animation.
! When using a system provided curve function, each frame during the animation will have a
! progress value between \ref ANIMATION_NORMALIZED_MIN and \ref ANIMATION_NORMALIZED_MAX based on
! the animation duration and the \ref AnimationCurve.
! For example, say an animation was scheduled at t = 1.0s, has a delay of 1.0s, a duration of 2.0s
! and a curve of AnimationCurveLinear. Then the .update callback will get called on t = 2.0s with
! distance_normalized = \ref ANIMATION_NORMALIZED_MIN. For each frame thereafter until t = 4.0s,
! the update callback will get called where distance_normalized is (\ref ANIMATION_NORMALIZED_MIN
! + (((\ref ANIMATION_NORMALIZED_MAX - \ref ANIMATION_NORMALIZED_MIN) * t) / duration)).
! Other system animation curve functions will result in a non-linear relation between
! distance_normalized and time.*/
pub type AnimationUpdateImplementation = ::core::option::Option<
	unsafe extern "C" fn(animation: *mut Animation, progress: AnimationProgress),
>;
/**! Pointer to function that (optionally) cleans up the animation.
! This callback is called when the animation is removed from the scheduler.
! In case the `.setup` implementation
! allocated any memory, this is a good place to release that memory again.
! @param animation The animation that needs to be teared down.
! @see animation_unschedule
! @see AnimationSetupImplementation*/
pub type AnimationTeardownImplementation =
	::core::option::Option<unsafe extern "C" fn(animation: *mut Animation)>;
/**! The 3 callbacks that implement a custom animation.
! Only the `.update` callback is mandatory, `.setup` and `.teardown` are optional.
! See the documentation with the function pointer typedefs for more information.
!
! @note The `.setup` callback is called immediately after scheduling the animation,
! regardless if there is a delay set for that animation using \ref animation_set_delay().
!
! The diagram below illustrates the order in which callbacks can be expected to get called
! over the life cycle of an animation. It also illustrates where the implementation of
! different animation callbacks are intended to be “living”.
! ![](animations.png)
!
! @see AnimationSetupImplementation
! @see AnimationUpdateImplementation
! @see AnimationTeardownImplementation*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct AnimationImplementation {
	/**! Called by the animation system when an animation is scheduled, to prepare it for running.
	! This callback is optional and can be left `NULL` when not needed.*/
	pub setup:    AnimationSetupImplementation,
	/**! Called by the animation system when the animation needs to calculate the next animation frame.
	! This callback is mandatory and should not be left `NULL`.*/
	pub update:   AnimationUpdateImplementation,
	/**! Called by the animation system when an animation is unscheduled, to clean up after it has run.
	! This callback is optional and can be left `NULL` when not needed.*/
	pub teardown: AnimationTeardownImplementation,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of AnimationImplementation"]
		[::core::mem::size_of::<AnimationImplementation>() - 12usize];
	["Alignment of AnimationImplementation"]
		[::core::mem::align_of::<AnimationImplementation>() - 4usize];
	["Offset of field: AnimationImplementation::setup"]
		[::core::mem::offset_of!(AnimationImplementation, setup) - 0usize];
	["Offset of field: AnimationImplementation::update"]
		[::core::mem::offset_of!(AnimationImplementation, update) - 4usize];
	["Offset of field: AnimationImplementation::teardown"]
		[::core::mem::offset_of!(AnimationImplementation, teardown) - 8usize];
};
/**! Sets the implementation of the custom animation.
! When implementing custom animations, use this function to specify what functions need to be
! called to for the setup, frame update and teardown of the animation.
! @note Trying to set an attribute when an animation is immutable will return false (failure). An
! animation is immutable once it has been added to a sequence or spawn animation or has been
! scheduled.
! @param animation The animation for which to set the implementation.
! @param implementation The structure with function pointers to the implementation of the setup,
!  update and teardown functions.
! @see AnimationImplementation
! @return true if successful, false on failure*/
/** animation_set_implementation
  -> Added in 32
gneiss: Vtable Slot 390, Offset 1560*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_set_implementation")]
pub unsafe extern "C" fn animation_set_implementation(
	animation: *mut Animation,
	implementation: *const AnimationImplementation,
) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1560", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the implementation of the custom animation.
! @param animation The animation for which to get the implementation.
! @see AnimationImplementation
! @return NULL if animation implementation has not been setup.*/
/** animation_get_implementation
  -> Added in 37
gneiss: Vtable Slot 437, Offset 1748*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.animation_get_implementation")]
pub unsafe extern "C" fn animation_get_implementation(
	animation: *mut Animation,
) -> *const AnimationImplementation {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1748", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
#[repr(C)]
#[derive(Debug)]
pub struct PropertyAnimation {
	_unused: [u8; 0],
}
/**! Convenience function to create and initialize a property animation that animates the frame of a
! Layer. It sets up the PropertyAnimation to use \ref layer_set_frame() and \ref layer_get_frame()
! as accessors and uses the `layer` parameter as the subject for the animation.
! The same defaults are used as with \ref animation_create().
! @param layer the layer that will be animated
! @param from_frame the frame that the layer should animate from
! @param to_frame the frame that the layer should animate to
! @note Pass in `NULL` as one of the frame arguments to have it set automatically to the layer's
! current frame. This will result in a call to \ref layer_get_frame() to get the current frame of
! the layer.
! @return A handle to the property animation. `NULL` if animation could not be created*/
/** property_animation_create_layer_frame
  -> Added in 32
gneiss: Vtable Slot 397, Offset 1588*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.property_animation_create_layer_frame")]
pub unsafe extern "C" fn property_animation_create_layer_frame(
	layer: *mut Layer,
	from_frame: *mut GRect,
	to_frame: *mut GRect,
) -> *mut PropertyAnimation {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1588", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Convenience function to create and initialize a property animation that animates the bound's
! origin of a Layer. It sets up the PropertyAnimation to use layer_set_bounds() and
! layer_get_bounds() as accessors and uses the `layer` parameter as the subject for the animation.
! The same defaults are used as with \ref animation_create().
! @param layer the layer that will be animated
! @param from the origin that the bounds should animate from
! @param to the origin that the layer should animate to
! @return A handle to the property animation. `NULL` if animation could not be created*/
/** property_animation_create_bounds_origin
  -> Added in 48
gneiss: Vtable Slot 516, Offset 2064*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.property_animation_create_bounds_origin")]
pub unsafe extern "C" fn property_animation_create_bounds_origin(
	layer: *mut Layer,
	from: *mut GPoint,
	to: *mut GPoint,
) -> *mut PropertyAnimation {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2064", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Creates a new PropertyAnimation on the heap and and initializes it with the specified values.
! The same defaults are used as with \ref animation_create().
! If the `from_value` or the `to_value` is `NULL`, the getter accessor will be called to get the
! current value of the property and be used instead.
! @param implementation Pointer to the implementation of the animation. In most cases, it makes
! sense to pass in a `static const` struct pointer.
! @param subject Pointer to the "subject" being animated. This will be passed in when the getter/
! setter accessors are called,
! see \ref PropertyAnimationAccessors, \ref GPointSetter, and friends. The value of this pointer
! will be copied into the `.subject` field of the PropertyAnimation struct.
! @param from_value Pointer to the value that the subject should animate from
! @param to_value Pointer to the value that the subject should animate to
! @note Pass in `NULL` as one of the value arguments to have it set automatically to the subject's
! current property value, as returned by the getter function. Also note that passing in `NULL` for
! both `from_value` and `to_value`, will result in the animation having the same from- and to-
! values, effectively not doing anything.
! @return A handle to the property animation. `NULL` if animation could not be created*/
/** property_animation_create
  -> Added in 32
gneiss: Vtable Slot 396, Offset 1584*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.property_animation_create")]
pub unsafe extern "C" fn property_animation_create(
	implementation: *const PropertyAnimationImplementation,
	subject: *mut ::core::ffi::c_void,
	from_value: *mut ::core::ffi::c_void,
	to_value: *mut ::core::ffi::c_void,
) -> *mut PropertyAnimation {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1584", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Destroy a property animation allocated by property_animation_create() or relatives.
! @param property_animation the return value from property_animation_create*/
/** property_animation_destroy
  -> Added in 32
gneiss: Vtable Slot 398, Offset 1592*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.property_animation_destroy")]
pub unsafe extern "C" fn property_animation_destroy(property_animation: *mut PropertyAnimation) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1592", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Default update callback for a property animations to update a property of type int16_t.
! Assign this function to the `.base.update` callback field of your
! PropertyAnimationImplementation, in combination with a `.getter` and `.setter` accessors of
! types \ref Int16Getter and \ref Int16Setter.
! The implementation of this function will calculate the next value of the animation and call the
! setter to set the new value upon the subject.
! @param property_animation The property animation for which the update is requested.
! @param distance_normalized The current normalized distance. See \ref
! AnimationUpdateImplementation
! @note This function is not supposed to be called "manually", but will be called automatically
! when the animation is being run.*/
/** property_animation_update_int16
  -> Added in 32
gneiss: Vtable Slot 405, Offset 1620*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.property_animation_update_int16")]
pub unsafe extern "C" fn property_animation_update_int16(
	property_animation: *mut PropertyAnimation,
	distance_normalized: u32,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1620", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Default update callback for a property animations to update a property of type uint32_t.
! Assign this function to the `.base.update` callback field of your
! PropertyAnimationImplementation, in combination with a `.getter` and `.setter` accessors of
! types \ref UInt32Getter and \ref UInt32Setter.
! The implementation of this function will calculate the next value of the animation and call the
! setter to set the new value upon the subject.
! @param property_animation The property animation for which the update is requested.
! @param distance_normalized The current normalized distance. See \ref
! AnimationUpdateImplementation
! @note This function is not supposed to be called "manually", but will be called automatically
! when the animation is being run.*/
/** property_animation_update_uint32
  -> Added in 49
gneiss: Vtable Slot 517, Offset 2068*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.property_animation_update_uint32")]
pub unsafe extern "C" fn property_animation_update_uint32(
	property_animation: *mut PropertyAnimation,
	distance_normalized: u32,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2068", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Default update callback for a property animations to update a property of type GPoint.
! Assign this function to the `.base.update` callback field of your
! PropertyAnimationImplementation,
! in combination with a `.getter` and `.setter` accessors of types \ref GPointGetter and \ref
! GPointSetter.
! The implementation of this function will calculate the next point of the animation and call the
! setter to set the new point upon the subject.
! @param property_animation The property animation for which the update is requested.
! @param distance_normalized The current normalized distance. See \ref
! AnimationUpdateImplementation
! @note This function is not supposed to be called "manually", but will be called automatically
! when the animation is being run.*/
/** property_animation_update_gpoint
  -> Added in 32
gneiss: Vtable Slot 403, Offset 1612*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.property_animation_update_gpoint")]
pub unsafe extern "C" fn property_animation_update_gpoint(
	property_animation: *mut PropertyAnimation,
	distance_normalized: u32,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1612", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Default update callback for a property animations to update a property of type GRect.
! Assign this function to the `.base.update` callback field of your
! PropertyAnimationImplementation, in combination with a `.getter` and `.setter` accessors of
! types \ref GRectGetter and \ref GRectSetter. The implementation of this function will calculate
! the next rectangle of the animation and call the setter to set the new rectangle upon the
! subject.
! @param property_animation The property animation for which the update is requested.
! @param distance_normalized The current normalized distance. See \ref
! AnimationUpdateImplementation
! @note This function is not supposed to be called "manually", but will be called automatically
! when the animation is being run.*/
/** property_animation_update_grect
  -> Added in 32
gneiss: Vtable Slot 404, Offset 1616*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.property_animation_update_grect")]
pub unsafe extern "C" fn property_animation_update_grect(
	property_animation: *mut PropertyAnimation,
	distance_normalized: u32,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1616", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Default update callback for a property animations to update a property of type GColor8.
! Assign this function to the `.base.update` callback field of your
! PropertyAnimationImplementation, in combination with a `.getter` and `.setter` accessors of
! types \ref GColor8Getter and \ref GColor8Setter. The implementation of this function will
! calculate the next rectangle of the animation and call the setter to set the new value upon
! the subject.
! @param property_animation The property animation for which the update is requested.
! @param distance_normalized The current normalized distance. See \ref
! AnimationUpdateImplementation
! @note This function is not supposed to be called "manually", but will be called automatically
! when the animation is being run.*/
/** property_animation_update_gcolor8
  -> Added in 61
gneiss: Vtable Slot 534, Offset 2136*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.property_animation_update_gcolor8")]
pub unsafe extern "C" fn property_animation_update_gcolor8(
	property_animation: *mut PropertyAnimation,
	distance_normalized: u32,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2136", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Work-around for function pointer return type GPoint to avoid
! tripping the pre-processor to use the equally named GPoint define*/
pub type GPointReturn = GPoint;
/**! Work-around for function pointer return type GRect to avoid
! tripping the pre-processor to use the equally named GRect define*/
pub type GRectReturn = GRect;
/**! Function signature of a setter function to set a property of type int16_t onto the subject.
! @see \ref property_animation_update_int16()
! @see \ref PropertyAnimationAccessors*/
pub type Int16Setter =
	::core::option::Option<unsafe extern "C" fn(subject: *mut ::core::ffi::c_void, int16: i16)>;
/**! Function signature of a getter function to get the current property of type int16_t of the
! subject.
! @see \ref property_animation_create()
! @see \ref PropertyAnimationAccessors*/
pub type Int16Getter =
	::core::option::Option<unsafe extern "C" fn(subject: *mut ::core::ffi::c_void) -> i16>;
/**! Function signature of a setter function to set a property of type uint32_t onto the subject.
! @see \ref property_animation_update_int16()
! @see \ref PropertyAnimationAccessors*/
pub type UInt32Setter =
	::core::option::Option<unsafe extern "C" fn(subject: *mut ::core::ffi::c_void, uint32: u32)>;
/**! Function signature of a getter function to get the current property of type uint32_t of the
! subject.
! @see \ref property_animation_create()
! @see \ref PropertyAnimationAccessors*/
pub type UInt32Getter =
	::core::option::Option<unsafe extern "C" fn(subject: *mut ::core::ffi::c_void) -> u32>;
/**! Function signature of a setter function to set a property of type GPoint onto the subject.
! @see \ref property_animation_update_gpoint()
! @see \ref PropertyAnimationAccessors*/
pub type GPointSetter =
	::core::option::Option<unsafe extern "C" fn(subject: *mut ::core::ffi::c_void, gpoint: GPoint)>;
/**! Function signature of a getter function to get the current property of type GPoint of the subject.
! @see \ref property_animation_create()
! @see \ref PropertyAnimationAccessors*/
pub type GPointGetter =
	::core::option::Option<unsafe extern "C" fn(subject: *mut ::core::ffi::c_void) -> GPointReturn>;
/**! Function signature of a setter function to set a property of type GRect onto the subject.
! @see \ref property_animation_update_grect()
! @see \ref PropertyAnimationAccessors*/
pub type GRectSetter =
	::core::option::Option<unsafe extern "C" fn(subject: *mut ::core::ffi::c_void, grect: GRect)>;
/**! Function signature of a getter function to get the current property of type GRect of the subject.
! @see \ref property_animation_create()
! @see \ref PropertyAnimationAccessors*/
pub type GRectGetter =
	::core::option::Option<unsafe extern "C" fn(subject: *mut ::core::ffi::c_void) -> GRectReturn>;
/**! Function signature of a setter function to set a property of type GColor8 onto the subject.
! @see \ref property_animation_update_gcolor8()
! @see \ref PropertyAnimationAccessors*/
pub type GColor8Setter = ::core::option::Option<
	unsafe extern "C" fn(subject: *mut ::core::ffi::c_void, gcolor: GColor8),
>;
/**! Function signature of a getter function to get the current property of type GColor8 of the
! subject.
! @see \ref property_animation_create()
! @see \ref PropertyAnimationAccessors*/
pub type GColor8Getter =
	::core::option::Option<unsafe extern "C" fn(subject: *mut ::core::ffi::c_void) -> GColor8>;
/**! @addtogroup PropertyAnimation
! \brief A ProperyAnimation animates the value of a "property" of a "subject" over time.
!
! <h3>Animating a Layer's frame property</h3>
! Currently there is only one specific type of property animation offered off-the-shelf, namely
! one to change the frame (property) of a layer (subject), see \ref
! property_animation_create_layer_frame().
!
! <h3>Implementing a custom PropertyAnimation</h3>
! It is fairly simple to create your own variant of a PropertyAnimation.
!
! Please refer to \htmlinclude UiFramework.html (chapter "Property Animations") for a conceptual
! overview of the animation framework and make sure you understand the underlying \ref Animation,
! in case you are not familiar with it, before trying to implement a variation on
! PropertyAnimation.
!
! To implement a custom property animation, use \ref property_animation_create() and provide a
! function pointers to the accessors (getter and setter) and setup, update and teardown callbacks
! in the implementation argument. Note that the type of property to animate with \ref
! PropertyAnimation is limited to int16_t, GPoint or GRect.
!
! For each of these types, there are implementations provided for the necessary `.update` handler
! of the animation: see \ref property_animation_update_int16(), \ref
! property_animation_update_gpoint() and \ref property_animation_update_grect().
! These update functions expect the `.accessors` to conform to the following interface:
! Any getter needs to have the following function signature: `__type__ getter(void *subject);`
! Any setter needs to have to following function signature: `void setter(void *subject,
! __type__ value);`
! See \ref Int16Getter, \ref Int16Setter, \ref GPointGetter, \ref GPointSetter,
! \ref GRectGetter, \ref GRectSetter for the typedefs that accompany the update fuctions.
!
! \code{.c}
! static const PropertyAnimationImplementation my_implementation = {
!   .base = {
!     // using the "stock" update callback:
!     .update = (AnimationUpdateImplementation) property_animation_update_gpoint,
!   },
!   .accessors = {
!     // my accessors that get/set a GPoint from/onto my subject:
!     .setter = { .gpoint = my_layer_set_corner_point, },
!     .getter = { .gpoint = (const GPointGetter) my_layer_get_corner_point, },
!   },
! };
! static PropertyAnimation* s_my_animation_ptr = NULL;
! static GPoint s_to_point = GPointZero;
! ...
! // Use NULL as 'from' value, this will make the animation framework call the getter
! // to get the current value of the property and use that as the 'from' value:
! s_my_animation_ptr = property_animation_create(&my_implementation, my_layer, NULL, &s_to_point);
! animation_schedule(property_animation_get_animation(s_my_animation_ptr));
! \endcode
! @{*/
#[repr(C)]
#[derive(Copy, Clone)]
pub struct PropertyAnimationAccessors {
	pub setter: PropertyAnimationAccessors__bindgen_ty_1,
	pub getter: PropertyAnimationAccessors__bindgen_ty_2,
}
/**! Function pointer to the implementation of the function that __sets__ the updated property
! value. This function will be called repeatedly for each animation frame.
! @see PropertyAnimationAccessors*/
#[repr(C)]
#[derive(Copy, Clone)]
pub union PropertyAnimationAccessors__bindgen_ty_1 {
	///! Use if the property to animate is of int16_t type
	pub int16:   Int16Setter,
	///! Use if the property to animate is of GPoint type
	pub gpoint:  GPointSetter,
	///! Use if the property to animate is of GRect type
	pub grect:   GRectSetter,
	///! Use if the property to animate is of GColor8 type
	pub gcolor8: GColor8Setter,
	///! Use if the property to animate is of uint32_t type
	pub uint32:  UInt32Setter,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of PropertyAnimationAccessors__bindgen_ty_1"]
		[::core::mem::size_of::<PropertyAnimationAccessors__bindgen_ty_1>() - 4usize];
	["Alignment of PropertyAnimationAccessors__bindgen_ty_1"]
		[::core::mem::align_of::<PropertyAnimationAccessors__bindgen_ty_1>() - 4usize];
	["Offset of field: PropertyAnimationAccessors__bindgen_ty_1::int16"]
		[::core::mem::offset_of!(PropertyAnimationAccessors__bindgen_ty_1, int16) - 0usize];
	["Offset of field: PropertyAnimationAccessors__bindgen_ty_1::gpoint"]
		[::core::mem::offset_of!(PropertyAnimationAccessors__bindgen_ty_1, gpoint) - 0usize];
	["Offset of field: PropertyAnimationAccessors__bindgen_ty_1::grect"]
		[::core::mem::offset_of!(PropertyAnimationAccessors__bindgen_ty_1, grect) - 0usize];
	["Offset of field: PropertyAnimationAccessors__bindgen_ty_1::gcolor8"]
		[::core::mem::offset_of!(PropertyAnimationAccessors__bindgen_ty_1, gcolor8) - 0usize];
	["Offset of field: PropertyAnimationAccessors__bindgen_ty_1::uint32"]
		[::core::mem::offset_of!(PropertyAnimationAccessors__bindgen_ty_1, uint32) - 0usize];
};
/**! Function pointer to the implementation of the function that __gets__ the current property
! value. This function will be called during \ref property_animation_create(), to get the current
! property value, in case the `from_value` or `to_value` argument is `NULL`.
! @see PropertyAnimationAccessors*/
#[repr(C)]
#[derive(Copy, Clone)]
pub union PropertyAnimationAccessors__bindgen_ty_2 {
	///! Use if the property to animate is of int16_t type
	pub int16:   Int16Getter,
	///! Use if the property to animate is of GPoint type
	pub gpoint:  GPointGetter,
	///! Use if the property to animate is of GRect type
	pub grect:   GRectGetter,
	///! Use if the property to animate is of GColor8 type
	pub gcolor8: GColor8Getter,
	///! Use if the property to animate is of uint32_t type
	pub uint32:  UInt32Getter,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of PropertyAnimationAccessors__bindgen_ty_2"]
		[::core::mem::size_of::<PropertyAnimationAccessors__bindgen_ty_2>() - 4usize];
	["Alignment of PropertyAnimationAccessors__bindgen_ty_2"]
		[::core::mem::align_of::<PropertyAnimationAccessors__bindgen_ty_2>() - 4usize];
	["Offset of field: PropertyAnimationAccessors__bindgen_ty_2::int16"]
		[::core::mem::offset_of!(PropertyAnimationAccessors__bindgen_ty_2, int16) - 0usize];
	["Offset of field: PropertyAnimationAccessors__bindgen_ty_2::gpoint"]
		[::core::mem::offset_of!(PropertyAnimationAccessors__bindgen_ty_2, gpoint) - 0usize];
	["Offset of field: PropertyAnimationAccessors__bindgen_ty_2::grect"]
		[::core::mem::offset_of!(PropertyAnimationAccessors__bindgen_ty_2, grect) - 0usize];
	["Offset of field: PropertyAnimationAccessors__bindgen_ty_2::gcolor8"]
		[::core::mem::offset_of!(PropertyAnimationAccessors__bindgen_ty_2, gcolor8) - 0usize];
	["Offset of field: PropertyAnimationAccessors__bindgen_ty_2::uint32"]
		[::core::mem::offset_of!(PropertyAnimationAccessors__bindgen_ty_2, uint32) - 0usize];
};
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of PropertyAnimationAccessors"]
		[::core::mem::size_of::<PropertyAnimationAccessors>() - 8usize];
	["Alignment of PropertyAnimationAccessors"]
		[::core::mem::align_of::<PropertyAnimationAccessors>() - 4usize];
	["Offset of field: PropertyAnimationAccessors::setter"]
		[::core::mem::offset_of!(PropertyAnimationAccessors, setter) - 0usize];
	["Offset of field: PropertyAnimationAccessors::getter"]
		[::core::mem::offset_of!(PropertyAnimationAccessors, getter) - 4usize];
};
/**! Data structure containing a collection of function pointers that form the implementation of the
! property animation.
! See the code example at the top (\ref PropertyAnimation).*/
#[repr(C)]
#[derive(Copy, Clone)]
pub struct PropertyAnimationImplementation {
	///! The "inherited" fields from the Animation "base class".
	pub base:      AnimationImplementation,
	///! The accessors to set/get the property to be animated.
	pub accessors: PropertyAnimationAccessors,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of PropertyAnimationImplementation"]
		[::core::mem::size_of::<PropertyAnimationImplementation>() - 20usize];
	["Alignment of PropertyAnimationImplementation"]
		[::core::mem::align_of::<PropertyAnimationImplementation>() - 4usize];
	["Offset of field: PropertyAnimationImplementation::base"]
		[::core::mem::offset_of!(PropertyAnimationImplementation, base) - 0usize];
	["Offset of field: PropertyAnimationImplementation::accessors"]
		[::core::mem::offset_of!(PropertyAnimationImplementation, accessors) - 12usize];
};
/**! Convenience function to retrieve an animation instance from a property animation instance
! @param property_animation The property animation
! @return The \ref Animation within this PropertyAnimation*/
/** property_animation_get_animation
  -> Added in 32
gneiss: Vtable Slot 400, Offset 1600*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.property_animation_get_animation")]
pub unsafe extern "C" fn property_animation_get_animation(
	property_animation: *mut PropertyAnimation,
) -> *mut Animation {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1600", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Helper function used by the property_animation_get|set_subject macros
! @param property_animation Handle to the property animation
! @param subject The subject to get or set.
! @param set true to set new subject, false to retrieve existing value
! @return true if successful, false on failure (usually a bad animation_h)*/
/** property_animation_subject
  -> Added in 32
gneiss: Vtable Slot 401, Offset 1604*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.property_animation_subject")]
pub unsafe extern "C" fn property_animation_subject(
	property_animation: *mut PropertyAnimation,
	subject: *mut *mut ::core::ffi::c_void,
	set: bool,
) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1604", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Helper function used by the property_animation_get|set_from_.* macros
! @param property_animation Handle to the property animation
! @param from Pointer to the value
! @param size Size of the from value
! @param set true to set new value, false to retrieve existing one
! @return true if successful, false on failure (usually a bad animation_h)*/
/** property_animation_from
  -> Added in 32
gneiss: Vtable Slot 399, Offset 1596*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.property_animation_from")]
pub unsafe extern "C" fn property_animation_from(
	property_animation: *mut PropertyAnimation,
	from: *mut ::core::ffi::c_void,
	size: usize,
	set: bool,
) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1596", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Helper function used by the property_animation_get|set_to_.* macros
! @param property_animation handle to the property animation
! @param to Pointer to the value
! @param size Size of the to value
! @param set true to set new value, false to retrieve existing one
! @return true if successful, false on failure (usually a bad animation_h)*/
/** property_animation_to
  -> Added in 32
gneiss: Vtable Slot 402, Offset 1608*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.property_animation_to")]
pub unsafe extern "C" fn property_animation_to(
	property_animation: *mut PropertyAnimation,
	to: *mut ::core::ffi::c_void,
	size: usize,
	set: bool,
) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1608", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Handler that will be called just before the unobstructed area will begin changing
! @param final_unobstructed_screen_area The final unobstructed screen area after
! the unobstructed area has finished changing.
! @param context A user-provided context.*/
pub type UnobstructedAreaWillChangeHandler = ::core::option::Option<
	unsafe extern "C" fn(final_unobstructed_screen_area: GRect, context: *mut ::core::ffi::c_void),
>;
/**! Handler that will be called every time the unobstructed area changes
! @param progress The progress of the animation changing the unobstructed area.
! @param context A user-provided context.*/
pub type UnobstructedAreaChangeHandler = ::core::option::Option<
	unsafe extern "C" fn(progress: AnimationProgress, context: *mut ::core::ffi::c_void),
>;
/**! Handler that will be called after the unobstructed area has finished changing
! @param context A user-provided context.*/
pub type UnobstructedAreaDidChangeHandler =
	::core::option::Option<unsafe extern "C" fn(context: *mut ::core::ffi::c_void)>;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct UnobstructedAreaHandlers {
	///! Handler that will be called just before the unobstructed area will begin changing.
	pub will_change: UnobstructedAreaWillChangeHandler,
	///! Handler that will be called every time the unobstructed area changes.
	pub change:      UnobstructedAreaChangeHandler,
	///! Handler that will be called after the unobstructed area has finished changing.
	pub did_change:  UnobstructedAreaDidChangeHandler,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of UnobstructedAreaHandlers"]
		[::core::mem::size_of::<UnobstructedAreaHandlers>() - 12usize];
	["Alignment of UnobstructedAreaHandlers"]
		[::core::mem::align_of::<UnobstructedAreaHandlers>() - 4usize];
	["Offset of field: UnobstructedAreaHandlers::will_change"]
		[::core::mem::offset_of!(UnobstructedAreaHandlers, will_change) - 0usize];
	["Offset of field: UnobstructedAreaHandlers::change"]
		[::core::mem::offset_of!(UnobstructedAreaHandlers, change) - 4usize];
	["Offset of field: UnobstructedAreaHandlers::did_change"]
		[::core::mem::offset_of!(UnobstructedAreaHandlers, did_change) - 8usize];
};
/**! Subscribe to be notified when the app's unobstructed area changes. When an unobstructed area
! begins changing, the `will_change` handler will be called, and every `will_change` call is
! always paired with a `did_change` call that occurs when it is done changing given that
! the `will_change` and `did_change` handlers are set. When subscribing while the unobstructed
! area is changing, the `will_change` handler will be called after subscription in the next event
! loop.
! @param handlers The handlers that should be called when the unobstructed area changes.
! @param context A user-provided context that will be passed to the callback handlers.
! @see layer_get_unobstructed_bounds*/
/** unobstructed_area_service_subscribe
  -> Added in 83
gneiss: Vtable Slot 624, Offset 2496*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.unobstructed_area_service_subscribe")]
pub unsafe extern "C" fn unobstructed_area_service_subscribe(
	handlers: UnobstructedAreaHandlers,
	context: *mut ::core::ffi::c_void,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2496", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Unsubscribe from notifications about changes to the app's unobstructed area.
/** unobstructed_area_service_unsubscribe
  -> Added in 83
gneiss: Vtable Slot 625, Offset 2500*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.unobstructed_area_service_unsubscribe")]
pub unsafe extern "C" fn unobstructed_area_service_unsubscribe() {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2500", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @addtogroup TextLayer
! \brief Layer that displays and formats a text string.
!
! ![](text_layer.png)
! The geometric information (bounds, frame) of the Layer
! is used as the "box" in which the text is drawn. The \ref TextLayer also has a number of
! other properties that influence how the text is drawn. Most important of these properties are:
! a pointer to the string to draw itself, the font, the text color, the background color of the
! layer, the overflow mode and alignment of the text inside the layer.
! @see Layer
! @see TextDrawing
! @see Fonts
! @{*/
#[repr(C)]
#[derive(Debug)]
pub struct TextLayer {
	_unused: [u8; 0],
}
/**! Creates a new TextLayer on the heap and initializes it with the default values.
!
! * Font: Raster Gothic 14-point Boldface (system font)
! * Text Alignment: \ref GTextAlignmentLeft
! * Text color: \ref GColorBlack
! * Background color: \ref GColorWhite
! * Clips: `true`
! * Hidden: `false`
! * Caching: `false`
!
! The text layer is automatically marked dirty after this operation.
! @param frame The frame with which to initialze the TextLayer
! @return A pointer to the TextLayer. `NULL` if the TextLayer could not
! be created*/
/** text_layer_create
  -> Added in 47
gneiss: Vtable Slot 462, Offset 1848*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.text_layer_create")]
pub unsafe extern "C" fn text_layer_create(frame: GRect) -> *mut TextLayer {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1848", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Destroys a TextLayer previously created by text_layer_create.
/** text_layer_destroy
  -> Added in 47
gneiss: Vtable Slot 463, Offset 1852*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.text_layer_destroy")]
pub unsafe extern "C" fn text_layer_destroy(text_layer: *mut TextLayer) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1852", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the "root" Layer of the text layer, which is the parent for the sub-
! layers used for its implementation.
! @param text_layer Pointer to the TextLayer for which to get the "root" Layer
! @return The "root" Layer of the text layer.*/
/** text_layer_get_layer
  -> Added in 47
gneiss: Vtable Slot 465, Offset 1860*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.text_layer_get_layer")]
pub unsafe extern "C" fn text_layer_get_layer(text_layer: *mut TextLayer) -> *mut Layer {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1860", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the pointer to the string where the TextLayer is supposed to find the text
! at a later point in time, when it needs to draw itself.
! @param text_layer The TextLayer of which to set the text
! @param text The new text to set onto the TextLayer. This must be a null-terminated and valid UTF-8 string!
! @note The string is not copied, so its buffer most likely cannot be stack allocated,
! but is recommended to be a buffer that is long-lived, at least as long as the TextLayer
! is part of a visible Layer hierarchy.
! @see text_layer_get_text*/
/** text_layer_set_text
  -> Added in 47
gneiss: Vtable Slot 471, Offset 1884*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.text_layer_set_text")]
pub unsafe extern "C" fn text_layer_set_text(
	text_layer: *mut TextLayer,
	text: *const ::core::ffi::c_char,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1884", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the pointer to the string that the TextLayer is using.
! @param text_layer The TextLayer for which to get the text
! @see text_layer_set_text*/
/** text_layer_get_text
  -> Added in 47
gneiss: Vtable Slot 466, Offset 1864*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.text_layer_get_text")]
pub unsafe extern "C" fn text_layer_get_text(
	text_layer: *mut TextLayer,
) -> *const ::core::ffi::c_char {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1864", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the background color of the bounding box that will be drawn behind the text
! @param text_layer The TextLayer of which to set the background color
! @param color The new \ref GColor to set the background to
! @see text_layer_set_text_color*/
/** text_layer_set_background_color
  -> Added in 47
gneiss: Vtable Slot 467, Offset 1868*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.text_layer_set_background_color")]
pub unsafe extern "C" fn text_layer_set_background_color(
	text_layer: *mut TextLayer,
	color: GColor,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1868", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the color of text that will be drawn
! @param text_layer The TextLayer of which to set the text color
! @param color The new \ref GColor to set the text color to
! @see text_layer_set_background_color*/
/** text_layer_set_text_color
  -> Added in 47
gneiss: Vtable Slot 473, Offset 1892*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.text_layer_set_text_color")]
pub unsafe extern "C" fn text_layer_set_text_color(text_layer: *mut TextLayer, color: GColor) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1892", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the line break mode of the TextLayer
! @param text_layer The TextLayer of which to set the overflow mode
! @param line_mode The new \ref GTextOverflowMode to set*/
/** text_layer_set_overflow_mode
  -> Added in 47
gneiss: Vtable Slot 469, Offset 1876*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.text_layer_set_overflow_mode")]
pub unsafe extern "C" fn text_layer_set_overflow_mode(
	text_layer: *mut TextLayer,
	line_mode: GTextOverflowMode,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1876", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the font of the TextLayer
! @param text_layer The TextLayer of which to set the font
! @param font The new \ref GFont for the TextLayer
! @see fonts_get_system_font
! @see fonts_load_custom_font*/
/** text_layer_set_font
  -> Added in 47
gneiss: Vtable Slot 468, Offset 1872*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.text_layer_set_font")]
pub unsafe extern "C" fn text_layer_set_font(text_layer: *mut TextLayer, font: GFont) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1872", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the alignment of the TextLayer
! @param text_layer The TextLayer of which to set the alignment
! @param text_alignment The new text alignment for the TextLayer
! @see GTextAlignment*/
/** text_layer_set_text_alignment
  -> Added in 47
gneiss: Vtable Slot 472, Offset 1888*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.text_layer_set_text_alignment")]
pub unsafe extern "C" fn text_layer_set_text_alignment(
	text_layer: *mut TextLayer,
	text_alignment: GTextAlignment,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1888", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Enables text flow following the boundaries of the screen and pagination that introduces
! extra line spacing at page breaks to avoid partially clipped lines for the TextLayer.
! If the TextLayer is part of a \ref ScrollLayer the ScrollLayer's frame will be used to
! configure paging.
! @note Make sure the TextLayer is part of the view hierarchy before calling this function.
!   Otherwise it has no effect.
! @param text_layer The TextLayer for which to enable text flow and paging
! @param inset Additional amount of pixels to inset to the inside of the screen for text flow
! @see text_layer_restore_default_text_flow_and_paging
! @see graphics_text_attributes_enable_screen_text_flow
! @see graphics_text_attributes_enable_paging*/
/** text_layer_enable_screen_text_flow_and_paging
  -> Added in 75
gneiss: Vtable Slot 596, Offset 2384*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.text_layer_enable_screen_text_flow_and_paging")]
pub unsafe extern "C" fn text_layer_enable_screen_text_flow_and_paging(
	text_layer: *mut TextLayer,
	inset: u8,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2384", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Restores text flow and paging for the TextLayer to the rectangular defaults.
! @param text_layer The TextLayer for which to restore text flow and paging
! @see text_layer_enable_screen_text_flow_and_paging
! @see graphics_text_attributes_restore_default_text_flow
! @see graphics_text_attributes_restore_default_paging*/
/** text_layer_restore_default_text_flow_and_paging
  -> Added in 75
gneiss: Vtable Slot 597, Offset 2388*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.text_layer_restore_default_text_flow_and_paging")]
pub unsafe extern "C" fn text_layer_restore_default_text_flow_and_paging(
	text_layer: *mut TextLayer,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2388", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Calculates the size occupied by the current text of the TextLayer
! @param text_layer the TextLayer for which to calculate the text's size
! @return The size occupied by the current text of the TextLayer*/
/** text_layer_get_content_size
  -> Added in 47
gneiss: Vtable Slot 464, Offset 1856*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.text_layer_get_content_size")]
pub unsafe extern "C" fn text_layer_get_content_size(text_layer: *mut TextLayer) -> GSize {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1856", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Update the size of the text layer
! This is a convenience function to update the frame of the TextLayer.
! @param text_layer The TextLayer of which to set the size
! @param max_size The new size for the TextLayer*/
/** text_layer_set_size
  -> Added in 47
gneiss: Vtable Slot 470, Offset 1880*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.text_layer_set_size")]
pub unsafe extern "C" fn text_layer_set_size(text_layer: *mut TextLayer, max_size: GSize) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1880", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @addtogroup ScrollLayer
! \brief Layer that scrolls its contents, animated.
!
! ![](scroll_layer.png)
! <h3>Key Points</h3>
! * Facilitates vertical scrolling of a layer sub-hierarchy zero or more
! arbitrary layers. The example image shows a scroll layer containing one
! large TextLayer.
! * Shadows to indicate that there is more content are automatically drawn
! on top of the content. When the end of the scroll layer is reached, the
! shadow will automatically be retracted.
! * Scrolling from one offset to another is animated implicitly by default.
! * The scroll layer contains a "content" sub-layer, which is the layer that
! is actually moved up an down. Any layer that is a child of this "content"
! sub-layer, will be moved as well. Effectively, an entire layout of layers
! can be scrolled this way. Use the convenience function
! \ref scroll_layer_add_child() to add child layers to the "content" sub-layer.
! * The scroll layer needs to be informed of the total size of the contents,
! in order to calculate from and to what point it should be able to scroll.
! Use \ref scroll_layer_set_content_size() to set the size of the contents.
! * The button behavior is set up, using the convenience function
! \ref scroll_layer_set_click_config_onto_window(). This will associate the
! UP and DOWN buttons with scrolling up and down.
! * The SELECT button can be configured by installing a click configuration
! provider using \ref scroll_layer_set_callbacks().
! * To scroll programatically to a certain offset, use
! \ref scroll_layer_set_content_offset().
! * It is possible to get called back for each scrolling increment, by
! installing the `.content_offset_changed_handler` callback using
! \ref scroll_layer_set_callbacks().
! * Only vertical scrolling is supported at the moment.
! @{*/
#[repr(C)]
#[derive(Debug)]
pub struct ScrollLayer {
	_unused: [u8; 0],
}
///! Function signature for the `.content_offset_changed_handler` callback.
pub type ScrollLayerCallback = ::core::option::Option<
	unsafe extern "C" fn(scroll_layer: *mut ScrollLayer, context: *mut ::core::ffi::c_void),
>;
/**! All the callbacks that the ScrollLayer exposes for use by applications.
! @note The context parameter can be set using scroll_layer_set_context() and
! gets passed in as context with all of these callbacks.*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ScrollLayerCallbacks {
	/**! Provider function to set up the SELECT button handlers. This will be
	! called after the scroll layer has configured the click configurations for
	! the up/down buttons, so it can also be used to modify the default up/down
	! scrolling behavior.*/
	pub click_config_provider:          ClickConfigProvider,
	/**! Called every time the the content offset changes. During a scrolling
	! animation, it will be called for each intermediary offset as well*/
	pub content_offset_changed_handler: ScrollLayerCallback,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of ScrollLayerCallbacks"][::core::mem::size_of::<ScrollLayerCallbacks>() - 8usize];
	["Alignment of ScrollLayerCallbacks"][::core::mem::align_of::<ScrollLayerCallbacks>() - 4usize];
	["Offset of field: ScrollLayerCallbacks::click_config_provider"]
		[::core::mem::offset_of!(ScrollLayerCallbacks, click_config_provider) - 0usize];
	["Offset of field: ScrollLayerCallbacks::content_offset_changed_handler"]
		[::core::mem::offset_of!(ScrollLayerCallbacks, content_offset_changed_handler) - 4usize];
};
/**! Creates a new ScrollLayer on the heap and initalizes it with the default values:
! * Clips: `true`
! * Hidden: `false`
! * Content size: `frame.size`
! * Content offset: \ref GPointZero
! * Callbacks: None (`NULL` for each one)
! * Callback context: `NULL`
! @return A pointer to the ScrollLayer. `NULL` if the ScrollLayer could not
! be created*/
/** scroll_layer_create
  -> Added in 0
gneiss: Vtable Slot 218, Offset 872*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.scroll_layer_create")]
pub unsafe extern "C" fn scroll_layer_create(frame: GRect) -> *mut ScrollLayer {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =872", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Destroys a ScrollLayer previously created by scroll_layer_create.
/** scroll_layer_destroy
  -> Added in 0
gneiss: Vtable Slot 219, Offset 876*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.scroll_layer_destroy")]
pub unsafe extern "C" fn scroll_layer_destroy(scroll_layer: *mut ScrollLayer) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =876", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the "root" Layer of the scroll layer, which is the parent for the sub-
! layers used for its implementation.
! @param scroll_layer Pointer to the ScrollLayer for which to get the "root" Layer
! @return The "root" Layer of the scroll layer.*/
/** scroll_layer_get_layer
  -> Added in 0
gneiss: Vtable Slot 222, Offset 888*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.scroll_layer_get_layer")]
pub unsafe extern "C" fn scroll_layer_get_layer(scroll_layer: *const ScrollLayer) -> *mut Layer {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =888", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Adds the child layer to the content sub-layer of the ScrollLayer.
! This will make the child layer part of the scrollable contents.
! The content sub-layer of the ScrollLayer will become the parent of the
! child layer.
! @param scroll_layer The ScrollLayer to which to add the child layer.
! @param child The Layer to add to the content sub-layer of the ScrollLayer.
! @note You may need to update the size of the scrollable contents using
! \ref scroll_layer_set_content_size().*/
/** scroll_layer_add_child
  -> Added in 0
gneiss: Vtable Slot 217, Offset 868*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.scroll_layer_add_child")]
pub unsafe extern "C" fn scroll_layer_add_child(scroll_layer: *mut ScrollLayer, child: *mut Layer) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =868", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Convenience function to set the \ref ClickConfigProvider callback on the
! given window to scroll layer's internal click config provider. This internal
! click configuration provider, will set up the default UP & DOWN
! scrolling behavior.
! This function calls \ref window_set_click_config_provider_with_context to
! accomplish this.
!
! If you application has set a `.click_config_provider`
! callback using \ref scroll_layer_set_callbacks(), this will be called
! by the internal click config provider, after configuring the UP & DOWN
! buttons. This allows your application to configure the SELECT button
! behavior and optionally override the UP & DOWN
! button behavior. The callback context for the SELECT click recognizer is
! automatically set to the scroll layer's context (see
! \ref scroll_layer_set_context() ). This context is passed into
! \ref ClickHandler callbacks. For the UP and DOWN buttons, the scroll layer
! itself is passed in by default as the callback context in order to deal with
! those buttons presses to scroll up and down automatically.
! @param scroll_layer The ScrollLayer that needs to receive click events.
! @param window The window for which to set the click configuration.
! @see \ref Clicks
! @see window_set_click_config_provider_with_context*/
/** scroll_layer_set_click_config_onto_window
  -> Added in 0
gneiss: Vtable Slot 227, Offset 908*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.scroll_layer_set_click_config_onto_window")]
pub unsafe extern "C" fn scroll_layer_set_click_config_onto_window(
	scroll_layer: *mut ScrollLayer,
	window: *mut Window,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =908", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the callbacks that the scroll layer exposes.
! The `context` as set by \ref scroll_layer_set_context() is passed into each
! of the callbacks. See \ref ScrollLayerCallbacks for the different callbacks.
! @note If the `context` is NULL, a pointer to scroll_layer is used
! as context parameter instead when calling callbacks.
! @param scroll_layer The ScrollLayer for which to assign new callbacks.
! @param callbacks The new callbacks.*/
/** scroll_layer_set_callbacks
  -> Added in 0
gneiss: Vtable Slot 226, Offset 904*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.scroll_layer_set_callbacks")]
pub unsafe extern "C" fn scroll_layer_set_callbacks(
	scroll_layer: *mut ScrollLayer,
	callbacks: ScrollLayerCallbacks,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =904", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets a new callback context. This context is passed into the scroll layer's
! callbacks and also the \ref ClickHandler for the SELECT button.
! If `NULL` or not set, the context defaults to a pointer to the ScrollLayer
! itself.
! @param scroll_layer The ScrollLayer for which to assign the new callback
! context.
! @param context The new callback context.
! @see scroll_layer_set_click_config_onto_window
! @see scroll_layer_set_callbacks*/
/** scroll_layer_set_context
  -> Added in 0
gneiss: Vtable Slot 230, Offset 920*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.scroll_layer_set_context")]
pub unsafe extern "C" fn scroll_layer_set_context(
	scroll_layer: *mut ScrollLayer,
	context: *mut ::core::ffi::c_void,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =920", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Scrolls to the given offset, optionally animated.
! @note When scrolling down, the offset's `.y` decrements. When scrolling up,
! the offset's `.y` increments. If scrolled completely to the top, the offset
! is \ref GPointZero.
! @note The `.x` field must be `0`. Horizontal scrolling is not supported.
! @param scroll_layer The ScrollLayer for which to set the content offset
! @param offset The final content offset
! @param animated Pass in `true` to animate to the new content offset, or
! `false` to set the new content offset without animating.
! @see scroll_layer_get_content_offset*/
/** scroll_layer_set_content_offset
  -> Added in 0
gneiss: Vtable Slot 228, Offset 912*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.scroll_layer_set_content_offset")]
pub unsafe extern "C" fn scroll_layer_set_content_offset(
	scroll_layer: *mut ScrollLayer,
	offset: GPoint,
	animated: bool,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =912", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the point by which the contents are offset.
! @param scroll_layer The ScrollLayer for which to get the content offset
! @see scroll_layer_set_content_offset*/
/** scroll_layer_get_content_offset
  -> Added in 0
gneiss: Vtable Slot 220, Offset 880*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.scroll_layer_get_content_offset")]
pub unsafe extern "C" fn scroll_layer_get_content_offset(scroll_layer: *mut ScrollLayer) -> GPoint {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =880", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the size of the contents layer. This determines the area that is
! scrollable. At the moment, this needs to be set "manually" and is not
! derived from the geometry of the contents layers.
! @param scroll_layer The ScrollLayer for which to set the content size.
! @param size The new content size.
! @see scroll_layer_get_content_size*/
/** scroll_layer_set_content_size
  -> Added in 0
gneiss: Vtable Slot 229, Offset 916*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.scroll_layer_set_content_size")]
pub unsafe extern "C" fn scroll_layer_set_content_size(
	scroll_layer: *mut ScrollLayer,
	size: GSize,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =916", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the size of the contents layer.
! @param scroll_layer The ScrollLayer for which to get the content size
! @see scroll_layer_set_content_size*/
/** scroll_layer_get_content_size
  -> Added in 0
gneiss: Vtable Slot 221, Offset 884*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.scroll_layer_get_content_size")]
pub unsafe extern "C" fn scroll_layer_get_content_size(scroll_layer: *const ScrollLayer) -> GSize {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =884", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set the frame of the scroll layer and adjusts the internal layers' geometry
! accordingly. The scroll layer is marked dirty automatically.
! @param scroll_layer The ScrollLayer for which to set the frame
! @param frame The new frame*/
/** scroll_layer_set_frame
  -> Added in 0
gneiss: Vtable Slot 231, Offset 924*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.scroll_layer_set_frame")]
pub unsafe extern "C" fn scroll_layer_set_frame(scroll_layer: *mut ScrollLayer, frame: GRect) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =924", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! The click handlers for the UP button that the scroll layer will install as
! part of \ref scroll_layer_set_click_config_onto_window().
! @note This handler is exposed, in case one wants to implement an alternative
! handler for the UP button, as a way to invoke the default behavior.
! @param recognizer The click recognizer for which the handler is called
! @param context A void pointer to the ScrollLayer that is the context of the click event*/
/** scroll_layer_scroll_up_click_handler
  -> Added in 0
gneiss: Vtable Slot 225, Offset 900*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.scroll_layer_scroll_up_click_handler")]
pub unsafe extern "C" fn scroll_layer_scroll_up_click_handler(
	recognizer: ClickRecognizerRef,
	context: *mut ::core::ffi::c_void,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =900", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! The click handlers for the DOWN button that the scroll layer will install as
! part of \ref scroll_layer_set_click_config_onto_window().
! @note This handler is exposed, in case one wants to implement an alternative
! handler for the DOWN button, as a way to invoke the default behavior.
! @param recognizer The click recognizer for which the handler is called
! @param context A void pointer to the ScrollLayer that is the context of the click event*/
/** scroll_layer_scroll_down_click_handler
  -> Added in 0
gneiss: Vtable Slot 224, Offset 896*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.scroll_layer_scroll_down_click_handler")]
pub unsafe extern "C" fn scroll_layer_scroll_down_click_handler(
	recognizer: ClickRecognizerRef,
	context: *mut ::core::ffi::c_void,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =896", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the visibility of the scroll layer shadow.
! If the visibility has changed, \ref layer_mark_dirty() will be called automatically
! on the scroll layer.
! @param scroll_layer The scroll layer for which to set the shadow visibility
! @param hidden Supply `true` to make the shadow hidden, or `false` to make it
! non-hidden.*/
/** scroll_layer_set_shadow_hidden
  -> Added in 0
gneiss: Vtable Slot 232, Offset 928*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.scroll_layer_set_shadow_hidden")]
pub unsafe extern "C" fn scroll_layer_set_shadow_hidden(
	scroll_layer: *mut ScrollLayer,
	hidden: bool,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =928", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the visibility of the scroll layer shadow.
! @param scroll_layer The scroll layer for which to get the visibility
! @return True if the shadow is hidden, false if it is not hidden.*/
/** scroll_layer_get_shadow_hidden
  -> Added in 0
gneiss: Vtable Slot 223, Offset 892*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.scroll_layer_get_shadow_hidden")]
pub unsafe extern "C" fn scroll_layer_get_shadow_hidden(scroll_layer: *const ScrollLayer) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =892", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Enables or disables paging of the ScrollLayer (default: disabled). When enabled, every button
! press will change the scroll offset by the frame's height.
! @param scroll_layer The scroll layer for which to enable or disable paging
! @param paging_enabled True, if paging should be enabled. False to enable.*/
/** scroll_layer_set_paging
  -> Added in 75
gneiss: Vtable Slot 595, Offset 2380*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.scroll_layer_set_paging")]
pub unsafe extern "C" fn scroll_layer_set_paging(
	scroll_layer: *mut ScrollLayer,
	paging_enabled: bool,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2380", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Check whether or not the ScrollLayer uses paging when pressing buttons.
! @param scroll_layer The scroll layer for which to get the paging behavior.
! @return True, if paging is enabled; false otherwise.*/
/** scroll_layer_get_paging
  -> Added in 75
gneiss: Vtable Slot 594, Offset 2376*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.scroll_layer_get_paging")]
pub unsafe extern "C" fn scroll_layer_get_paging(scroll_layer: *mut ScrollLayer) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2376", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
#[repr(C)]
#[derive(Debug)]
pub struct ContentIndicator {
	_unused: [u8; 0],
}
/**! Gets the ContentIndicator for a ScrollLayer.
! @param scroll_layer The ScrollLayer for which to get the ContentIndicator
! @return A pointer to the ContentIndicator, or `NULL` upon failure.*/
/** scroll_layer_get_content_indicator
  -> Added in 71
gneiss: Vtable Slot 577, Offset 2308*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.scroll_layer_get_content_indicator")]
pub unsafe extern "C" fn scroll_layer_get_content_indicator(
	scroll_layer: *mut ScrollLayer,
) -> *mut ContentIndicator {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2308", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///!< The up direction.
pub const ContentIndicatorDirection_ContentIndicatorDirectionUp: ContentIndicatorDirection = 0;
///!< The down direction.
pub const ContentIndicatorDirection_ContentIndicatorDirectionDown: ContentIndicatorDirection = 1;
///!< The number of supported directions.
pub const ContentIndicatorDirection_NumContentIndicatorDirections: ContentIndicatorDirection = 2;
/**! Value to describe directions for \ref ContentIndicator.
! @see \ref content_indicator_configure_direction
! @see \ref content_indicator_set_content_available*/
pub type ContentIndicatorDirection = ::core::ffi::c_uchar;
/**! Struct used to configure directions for \ref ContentIndicator.
! @see \ref content_indicator_configure_direction*/
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ContentIndicatorConfig {
	///!< The layer where the arrow indicator will be rendered when content is available.
	pub layer:     *mut Layer,
	///!< Whether the display of the arrow indicator should timeout.
	pub times_out: bool,
	///!< The alignment of the arrow within the provided layer.
	pub alignment: GAlign,
	pub colors:    ContentIndicatorConfig__bindgen_ty_1,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ContentIndicatorConfig__bindgen_ty_1 {
	///!< The color of the arrow.
	pub foreground: GColor,
	///!< The color of the layer behind the arrow.
	pub background: GColor,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of ContentIndicatorConfig__bindgen_ty_1"]
		[::core::mem::size_of::<ContentIndicatorConfig__bindgen_ty_1>() - 2usize];
	["Alignment of ContentIndicatorConfig__bindgen_ty_1"]
		[::core::mem::align_of::<ContentIndicatorConfig__bindgen_ty_1>() - 1usize];
	["Offset of field: ContentIndicatorConfig__bindgen_ty_1::foreground"]
		[::core::mem::offset_of!(ContentIndicatorConfig__bindgen_ty_1, foreground) - 0usize];
	["Offset of field: ContentIndicatorConfig__bindgen_ty_1::background"]
		[::core::mem::offset_of!(ContentIndicatorConfig__bindgen_ty_1, background) - 1usize];
};
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of ContentIndicatorConfig"][::core::mem::size_of::<ContentIndicatorConfig>() - 8usize];
	["Alignment of ContentIndicatorConfig"]
		[::core::mem::align_of::<ContentIndicatorConfig>() - 4usize];
	["Offset of field: ContentIndicatorConfig::layer"]
		[::core::mem::offset_of!(ContentIndicatorConfig, layer) - 0usize];
	["Offset of field: ContentIndicatorConfig::times_out"]
		[::core::mem::offset_of!(ContentIndicatorConfig, times_out) - 4usize];
	["Offset of field: ContentIndicatorConfig::alignment"]
		[::core::mem::offset_of!(ContentIndicatorConfig, alignment) - 5usize];
	["Offset of field: ContentIndicatorConfig::colors"]
		[::core::mem::offset_of!(ContentIndicatorConfig, colors) - 6usize];
};
/**! Creates a ContentIndicator on the heap.
! @return A pointer to the ContentIndicator. `NULL` if the ContentIndicator could not be created.*/
/** content_indicator_create
  -> Added in 71
gneiss: Vtable Slot 573, Offset 2292*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.content_indicator_create")]
pub unsafe extern "C" fn content_indicator_create() -> *mut ContentIndicator {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2292", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Destroys a ContentIndicator previously created using \ref content_indicator_create().
! @param content_indicator The ContentIndicator to destroy.*/
/** content_indicator_destroy
  -> Added in 71
gneiss: Vtable Slot 574, Offset 2296*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.content_indicator_destroy")]
pub unsafe extern "C" fn content_indicator_destroy(content_indicator: *mut ContentIndicator) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2296", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Configures a ContentIndicator for the given direction.
! @param content_indicator The ContentIndicator to configure.
! @param direction The direction for which to configure the ContentIndicator.
! @param config The configuration to use to configure the ContentIndicator. If NULL, the data
! for the specified direction will be reset.
! @return True if the ContentIndicator was successfully configured for the given direction,
! false otherwise.*/
/** content_indicator_configure_direction
  -> Added in 71
gneiss: Vtable Slot 572, Offset 2288*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.content_indicator_configure_direction")]
pub unsafe extern "C" fn content_indicator_configure_direction(
	content_indicator: *mut ContentIndicator,
	direction: ContentIndicatorDirection,
	config: *const ContentIndicatorConfig,
) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2288", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Retrieves the availability status of content in the given direction.
! @param content_indicator The ContentIndicator for which to get the content availability.
! @param direction The direction for which to get the content availability.
! @return True if content is available in the given direction, false otherwise.*/
/** content_indicator_get_content_available
  -> Added in 71
gneiss: Vtable Slot 575, Offset 2300*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.content_indicator_get_content_available")]
pub unsafe extern "C" fn content_indicator_get_content_available(
	content_indicator: *mut ContentIndicator,
	direction: ContentIndicatorDirection,
) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2300", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the availability status of content in the given direction.
! @param content_indicator The ContentIndicator for which to set the content availability.
! @param direction The direction for which to set the content availability.
! @param available Whether or not content is available.
! @note If times_out is enabled, calling this function resets any previously scheduled timeout
! timer for the ContentIndicator.*/
/** content_indicator_set_content_available
  -> Added in 71
gneiss: Vtable Slot 576, Offset 2304*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.content_indicator_set_content_available")]
pub unsafe extern "C" fn content_indicator_set_content_available(
	content_indicator: *mut ContentIndicator,
	direction: ContentIndicatorDirection,
	available: bool,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2304", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Section drawing function to draw a basic section cell with the title, subtitle, and icon of the
! section. Call this function inside the `.draw_row` callback implementation, see \ref
! MenuLayerCallbacks. Note that if the size of `cell_layer` is too small to fit all of the cell
! items specified, not all of them may be drawn.
! @param ctx The destination graphics context
! @param cell_layer The layer of the cell to draw
! @param title If non-null, draws a title in larger text (24 points, bold
! Raster Gothic system font).
! @param subtitle If non-null, draws a subtitle in smaller text (18 points,
! Raster Gothic system font). If `NULL`, the title will be centered vertically
! inside the menu cell.
! @param icon If non-null, draws an icon to the left of the text. If `NULL`,
! the icon will be omitted and the leftover space is used for the title and
! subtitle.*/
/** menu_cell_basic_draw
  -> Added in 0
gneiss: Vtable Slot 165, Offset 660*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.menu_cell_basic_draw")]
pub unsafe extern "C" fn menu_cell_basic_draw(
	ctx: *mut GContext,
	cell_layer: *const Layer,
	title: *const ::core::ffi::c_char,
	subtitle: *const ::core::ffi::c_char,
	icon: *mut GBitmap,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =660", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Cell drawing function to draw a basic menu cell layout with title, subtitle
! Cell drawing function to draw a menu cell layout with only one big title.
! Call this function inside the `.draw_row` callback implementation, see
! \ref MenuLayerCallbacks.
! @param ctx The destination graphics context
! @param cell_layer The layer of the cell to draw
! @param title If non-null, draws a title in larger text (28 points, bold
! Raster Gothic system font).*/
/** menu_cell_title_draw
  -> Added in 0
gneiss: Vtable Slot 167, Offset 668*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.menu_cell_title_draw")]
pub unsafe extern "C" fn menu_cell_title_draw(
	ctx: *mut GContext,
	cell_layer: *const Layer,
	title: *const ::core::ffi::c_char,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =668", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Section header drawing function to draw a basic section header cell layout
! with the title of the section.
! Call this function inside the `.draw_header` callback implementation, see
! \ref MenuLayerCallbacks.
! @param ctx The destination graphics context
! @param cell_layer The layer of the cell to draw
! @param title If non-null, draws the title in small text (14 points, bold
! Raster Gothic system font).*/
/** menu_cell_basic_header_draw
  -> Added in 0
gneiss: Vtable Slot 166, Offset 664*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.menu_cell_basic_header_draw")]
pub unsafe extern "C" fn menu_cell_basic_header_draw(
	ctx: *mut GContext,
	cell_layer: *const Layer,
	title: *const ::core::ffi::c_char,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =664", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Data structure to represent an menu item's position in a menu, by specifying
! the section index and the row index within that section.*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MenuIndex {
	///! The index of the section
	pub section: u16,
	///! The index of the row within the section with index `.section`
	pub row:     u16,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of MenuIndex"][::core::mem::size_of::<MenuIndex>() - 4usize];
	["Alignment of MenuIndex"][::core::mem::align_of::<MenuIndex>() - 2usize];
	["Offset of field: MenuIndex::section"][::core::mem::offset_of!(MenuIndex, section) - 0usize];
	["Offset of field: MenuIndex::row"][::core::mem::offset_of!(MenuIndex, row) - 2usize];
};
/**! Comparator function to determine the order of two MenuIndex values.
! @param a Pointer to the menu index of the first item
! @param b Pointer to the menu index of the second item
! @return 0 if A and B are equal, 1 if A has a higher section & row
! combination than B or else -1*/
/** menu_index_compare
  -> Added in 0
gneiss: Vtable Slot 168, Offset 672*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.menu_index_compare")]
pub unsafe extern "C" fn menu_index_compare(a: *const MenuIndex, b: *const MenuIndex) -> i16 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =672", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MenuCellSpan {
	pub y:     i16,
	pub h:     i16,
	pub sep:   i16,
	pub index: MenuIndex,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of MenuCellSpan"][::core::mem::size_of::<MenuCellSpan>() - 10usize];
	["Alignment of MenuCellSpan"][::core::mem::align_of::<MenuCellSpan>() - 2usize];
	["Offset of field: MenuCellSpan::y"][::core::mem::offset_of!(MenuCellSpan, y) - 0usize];
	["Offset of field: MenuCellSpan::h"][::core::mem::offset_of!(MenuCellSpan, h) - 2usize];
	["Offset of field: MenuCellSpan::sep"][::core::mem::offset_of!(MenuCellSpan, sep) - 4usize];
	["Offset of field: MenuCellSpan::index"][::core::mem::offset_of!(MenuCellSpan, index) - 6usize];
};
#[repr(C)]
#[derive(Debug)]
pub struct MenuLayer {
	_unused: [u8; 0],
}
/**! Function signature for the callback to get the number of sections in a menu.
! @param menu_layer The \ref MenuLayer for which the data is requested
! @param callback_context The callback context
! @return The number of sections in the menu
! @see \ref menu_layer_set_callbacks()
! @see \ref MenuLayerCallbacks*/
pub type MenuLayerGetNumberOfSectionsCallback = ::core::option::Option<
	unsafe extern "C" fn(
		menu_layer: *mut MenuLayer,
		callback_context: *mut ::core::ffi::c_void,
	) -> u16,
>;
/**! Function signature for the callback to get the number of rows in a
! given section in a menu.
! @param menu_layer The \ref MenuLayer for which the data is requested
! @param section_index The index of the section of the menu for which the
! number of items it contains is requested
! @param callback_context The callback context
! @return The number of rows in the given section in the menu
! @see \ref menu_layer_set_callbacks()
! @see \ref MenuLayerCallbacks*/
pub type MenuLayerGetNumberOfRowsInSectionsCallback = ::core::option::Option<
	unsafe extern "C" fn(
		menu_layer: *mut MenuLayer,
		section_index: u16,
		callback_context: *mut ::core::ffi::c_void,
	) -> u16,
>;
/**! Function signature for the callback to get the height of the menu cell
! at a given index.
! @param menu_layer The \ref MenuLayer for which the data is requested
! @param cell_index The MenuIndex for which the cell height is requested
! @param callback_context The callback context
! @return The height of the cell at the given MenuIndex
! @see \ref menu_layer_set_callbacks()
! @see \ref MenuLayerCallbacks*/
pub type MenuLayerGetCellHeightCallback = ::core::option::Option<
	unsafe extern "C" fn(
		menu_layer: *mut MenuLayer,
		cell_index: *mut MenuIndex,
		callback_context: *mut ::core::ffi::c_void,
	) -> i16,
>;
/**! Function signature for the callback to get the height of the section header
! at a given section index.
! @param menu_layer The \ref MenuLayer for which the data is requested
! @param section_index The index of the section for which the header height is
! requested
! @param callback_context The callback context
! @return The height of the section header at the given section index
! @see \ref menu_layer_set_callbacks()
! @see \ref MenuLayerCallbacks*/
pub type MenuLayerGetHeaderHeightCallback = ::core::option::Option<
	unsafe extern "C" fn(
		menu_layer: *mut MenuLayer,
		section_index: u16,
		callback_context: *mut ::core::ffi::c_void,
	) -> i16,
>;
/**! Function signature for the callback to get the height of the separator
! at a given index.
! @param menu_layer The \ref MenuLayer for which the data is requested
! @param cell_index The MenuIndex for which the cell height is requested
! @param callback_context The callback context
! @return The height of the separator at the given MenuIndex
! @see \ref menu_layer_set_callbacks()
! @see \ref MenuLayerCallbacks*/
pub type MenuLayerGetSeparatorHeightCallback = ::core::option::Option<
	unsafe extern "C" fn(
		menu_layer: *mut MenuLayer,
		cell_index: *mut MenuIndex,
		callback_context: *mut ::core::ffi::c_void,
	) -> i16,
>;
/**! Function signature for the callback to render the menu cell at a given
! MenuIndex.
! @param ctx The destination graphics context to draw into
! @param cell_layer The cell's layer, containing the geometry of the cell
! @param cell_index The MenuIndex of the cell that needs to be drawn
! @param callback_context The callback context
! @note The `cell_layer` argument is provided to make it easy to re-use an
! `.update_proc` implementation in this callback. Only the bounds and frame
! of the `cell_layer` are actually valid and other properties should be
! ignored.
! @see \ref menu_layer_set_callbacks()
! @see \ref MenuLayerCallbacks*/
pub type MenuLayerDrawRowCallback = ::core::option::Option<
	unsafe extern "C" fn(
		ctx: *mut GContext,
		cell_layer: *const Layer,
		cell_index: *mut MenuIndex,
		callback_context: *mut ::core::ffi::c_void,
	),
>;
/**! Function signature for the callback to render the section header at a given
! section index.
! @param ctx The destination graphics context to draw into
! @param cell_layer The header cell's layer, containing the geometry of the
! header cell
! @param section_index The section index of the section header that needs to
! be drawn
! @param callback_context The callback context
! @note The `cell_layer` argument is provided to make it easy to re-use an
! `.update_proc` implementation in this callback. Only the bounds and frame
! of the `cell_layer` are actually valid and other properties should be
! ignored.
! @see \ref menu_layer_set_callbacks()
! @see \ref MenuLayerCallbacks*/
pub type MenuLayerDrawHeaderCallback = ::core::option::Option<
	unsafe extern "C" fn(
		ctx: *mut GContext,
		cell_layer: *const Layer,
		section_index: u16,
		callback_context: *mut ::core::ffi::c_void,
	),
>;
/**! Function signature for the callback to render the separator at a given
! MenuIndex.
! @param ctx The destination graphics context to draw into
! @param cell_layer The cell's layer, containing the geometry of the cell
! @param cell_index The MenuIndex of the separator that needs to be drawn
! @param callback_context The callback context
! @note The `cell_layer` argument is provided to make it easy to re-use an
! `.update_proc` implementation in this callback. Only the bounds and frame
! of the `cell_layer` are actually valid and other properties should be
! ignored.
! @see \ref menu_layer_set_callbacks()
! @see \ref MenuLayerCallbacks*/
pub type MenuLayerDrawSeparatorCallback = ::core::option::Option<
	unsafe extern "C" fn(
		ctx: *mut GContext,
		cell_layer: *const Layer,
		cell_index: *mut MenuIndex,
		callback_context: *mut ::core::ffi::c_void,
	),
>;
/**! Function signature for the callback to handle the event that a user hits
! the SELECT button.
! @param menu_layer The \ref MenuLayer for which the selection event occured
! @param cell_index The MenuIndex of the cell that is selected
! @param callback_context The callback context
! @see \ref menu_layer_set_callbacks()
! @see \ref MenuLayerCallbacks*/
pub type MenuLayerSelectCallback = ::core::option::Option<
	unsafe extern "C" fn(
		menu_layer: *mut MenuLayer,
		cell_index: *mut MenuIndex,
		callback_context: *mut ::core::ffi::c_void,
	),
>;
/**! Function signature for the callback to handle a change in the current
! selected item in the menu.
! @param menu_layer The \ref MenuLayer for which the selection event occured
! @param new_index The MenuIndex of the new item that is selected now
! @param old_index The MenuIndex of the old item that was selected before
! @param callback_context The callback context
! @see \ref menu_layer_set_callbacks()
! @see \ref MenuLayerCallbacks*/
pub type MenuLayerSelectionChangedCallback = ::core::option::Option<
	unsafe extern "C" fn(
		menu_layer: *mut MenuLayer,
		new_index: MenuIndex,
		old_index: MenuIndex,
		callback_context: *mut ::core::ffi::c_void,
	),
>;
/**! Function signature for the callback which allows or changes selection behavior of the menu.
! In order to change the cell that should be selected, modify the passed in new_index.
! Preventing the selection from changing, new_index can be assigned the value of old_index.
! @param menu_layer The \ref MenuLayer for which the selection event that occured
! @param new_index Pointer to the index that the MenuLayer is going to change selection to.
! @param old_index The index that is being unselected.
! @param callback_context The callback context
! @note \ref menu_layer_set_selected_index will not trigger this callback when
! the selection changes, but \ref menu_layer_set_selected_next will.*/
pub type MenuLayerSelectionWillChangeCallback = ::core::option::Option<
	unsafe extern "C" fn(
		menu_layer: *mut MenuLayer,
		new_index: *mut MenuIndex,
		old_index: MenuIndex,
		callback_context: *mut ::core::ffi::c_void,
	),
>;
/**! Function signature for the callback which draws the menu's background.
! The background is underneath the cells of the menu, and is visible in the
! padding below the bottom cell, or if a cell's background color is set to \ref GColorClear.
! @param ctx The destination graphics context to draw into.
! @param bg_layer The background's layer, containing the geometry of the background.
! @param highlight Whether this should be rendered as highlighted or not. Highlight style
! should match the highlight style of cells, since this color can be used for animating selection.*/
pub type MenuLayerDrawBackgroundCallback = ::core::option::Option<
	unsafe extern "C" fn(
		ctx: *mut GContext,
		bg_layer: *const Layer,
		highlight: bool,
		callback_context: *mut ::core::ffi::c_void,
	),
>;
///! Data structure containing all the callbacks of a \ref MenuLayer.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MenuLayerCallbacks {
	/**! Callback that gets called to get the number of sections in the menu.
	! This can get called at various moments throughout the life of a menu.
	! @note When `NULL`, the number of sections defaults to 1.*/
	pub get_num_sections:      MenuLayerGetNumberOfSectionsCallback,
	/**! Callback that gets called to get the number of rows in a section. This
	! can get called at various moments throughout the life of a menu.
	! @note Must be set to a valid callback; `NULL` causes undefined behavior.*/
	pub get_num_rows:          MenuLayerGetNumberOfRowsInSectionsCallback,
	/**! Callback that gets called to get the height of a cell.
	! This can get called at various moments throughout the life of a menu.
	! @note When `NULL`, the default height of @c MENU_CELL_BASIC_CELL_HEIGHT pixels is used.
	! Developers may wish to use \ref MENU_CELL_ROUND_FOCUSED_SHORT_CELL_HEIGHT
	! and \ref MENU_CELL_ROUND_UNFOCUSED_SHORT_CELL_HEIGHT on a round display
	! to respect the system aesthetic.*/
	pub get_cell_height:       MenuLayerGetCellHeightCallback,
	/**! Callback that gets called to get the height of a section header.
	! This can get called at various moments throughout the life of a menu.
	! @note When `NULL`, the default height of 0 pixels is used. This disables
	! section headers.*/
	pub get_header_height:     MenuLayerGetHeaderHeightCallback,
	/**! Callback that gets called to render a menu item.
	! This gets called for each menu item, every time it needs to be
	! re-rendered.
	! @note Must be set to a valid callback; `NULL` causes undefined behavior.*/
	pub draw_row:              MenuLayerDrawRowCallback,
	/**! Callback that gets called to render a section header.
	! This gets called for each section header, every time it needs to be
	! re-rendered.
	! @note Must be set to a valid callback, unless `.get_header_height` is
	! `NULL`. Causes undefined behavior otherwise.*/
	pub draw_header:           MenuLayerDrawHeaderCallback,
	/**! Callback that gets called when the user triggers a click with the SELECT
	! button.
	! @note When `NULL`, click events for the SELECT button are ignored.*/
	pub select_click:          MenuLayerSelectCallback,
	/**! Callback that gets called when the user triggers a long click with the
	! SELECT button.
	! @note When `NULL`, long click events for the SELECT button are ignored.*/
	pub select_long_click:     MenuLayerSelectCallback,
	/**! Callback that gets called whenever the selection changes.
	! @note When `NULL`, selection change events are ignored.*/
	pub selection_changed:     MenuLayerSelectionChangedCallback,
	/**! Callback that gets called to get the height of a separator
	! This can get called at various moments throughout the life of a menu.
	! @note When `NULL`, the default height of 0 is used.*/
	pub get_separator_height:  MenuLayerGetSeparatorHeightCallback,
	/**! Callback that gets called to render a separator.
	! This gets called for each separator, every time it needs to be
	! re-rendered.
	! @note Must be set to a valid callback, unless `.get_separator_height` is
	! `NULL`. Causes undefined behavior otherwise.*/
	pub draw_separator:        MenuLayerDrawSeparatorCallback,
	/**! Callback that gets called before the selected cell changes.
	! This gets called before the selected item in the MenuLayer is changed,
	! and will allow for the selected cell to be overridden.
	! This allows for skipping cells in the menu, locking selection onto a given item,*/
	pub selection_will_change: MenuLayerSelectionWillChangeCallback,
	/**! Callback that gets called before any cells are drawn.
	! This supports two states, either highlighted or not highlighted.
	! If highlighted is specified, it is expected to be colored in the same
	! style as the menu's cells are.
	! If this callback is not specified, it will default to the colors set with
	! \ref menu_layer_set_normal_colors and \ref menu_layer_set_highlight_colors.*/
	pub draw_background:       MenuLayerDrawBackgroundCallback,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of MenuLayerCallbacks"][::core::mem::size_of::<MenuLayerCallbacks>() - 52usize];
	["Alignment of MenuLayerCallbacks"][::core::mem::align_of::<MenuLayerCallbacks>() - 4usize];
	["Offset of field: MenuLayerCallbacks::get_num_sections"]
		[::core::mem::offset_of!(MenuLayerCallbacks, get_num_sections) - 0usize];
	["Offset of field: MenuLayerCallbacks::get_num_rows"]
		[::core::mem::offset_of!(MenuLayerCallbacks, get_num_rows) - 4usize];
	["Offset of field: MenuLayerCallbacks::get_cell_height"]
		[::core::mem::offset_of!(MenuLayerCallbacks, get_cell_height) - 8usize];
	["Offset of field: MenuLayerCallbacks::get_header_height"]
		[::core::mem::offset_of!(MenuLayerCallbacks, get_header_height) - 12usize];
	["Offset of field: MenuLayerCallbacks::draw_row"]
		[::core::mem::offset_of!(MenuLayerCallbacks, draw_row) - 16usize];
	["Offset of field: MenuLayerCallbacks::draw_header"]
		[::core::mem::offset_of!(MenuLayerCallbacks, draw_header) - 20usize];
	["Offset of field: MenuLayerCallbacks::select_click"]
		[::core::mem::offset_of!(MenuLayerCallbacks, select_click) - 24usize];
	["Offset of field: MenuLayerCallbacks::select_long_click"]
		[::core::mem::offset_of!(MenuLayerCallbacks, select_long_click) - 28usize];
	["Offset of field: MenuLayerCallbacks::selection_changed"]
		[::core::mem::offset_of!(MenuLayerCallbacks, selection_changed) - 32usize];
	["Offset of field: MenuLayerCallbacks::get_separator_height"]
		[::core::mem::offset_of!(MenuLayerCallbacks, get_separator_height) - 36usize];
	["Offset of field: MenuLayerCallbacks::draw_separator"]
		[::core::mem::offset_of!(MenuLayerCallbacks, draw_separator) - 40usize];
	["Offset of field: MenuLayerCallbacks::selection_will_change"]
		[::core::mem::offset_of!(MenuLayerCallbacks, selection_will_change) - 44usize];
	["Offset of field: MenuLayerCallbacks::draw_background"]
		[::core::mem::offset_of!(MenuLayerCallbacks, draw_background) - 48usize];
};
/**! Creates a new \ref MenuLayer on the heap and initalizes it with the default values.
!
! * Clips: `true`
! * Hidden: `false`
! * Content size: `frame.size`
! * Content offset: \ref GPointZero
! * Callbacks: None (`NULL` for each one)
! * Callback context: `NULL`
! * After the relevant callbacks are called to populate the menu, the item at MenuIndex(0, 0)
!   will be selected initially.
! @return A pointer to the \ref MenuLayer. `NULL` if the \ref MenuLayer could not
! be created*/
/** menu_layer_create
  -> Added in 39
gneiss: Vtable Slot 439, Offset 1756*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.menu_layer_create")]
pub unsafe extern "C" fn menu_layer_create(frame: GRect) -> *mut MenuLayer {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1756", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Destroys a \ref MenuLayer previously created by menu_layer_create.
/** menu_layer_destroy
  -> Added in 0
gneiss: Vtable Slot 170, Offset 680*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.menu_layer_destroy")]
pub unsafe extern "C" fn menu_layer_destroy(menu_layer: *mut MenuLayer) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =680", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the "root" Layer of the \ref MenuLayer, which is the parent for the sub-
! layers used for its implementation.
! @param menu_layer Pointer to the MenuLayer for which to get the "root" Layer
! @return The "root" Layer of the \ref MenuLayer.*/
/** menu_layer_get_layer
  -> Added in 0
gneiss: Vtable Slot 171, Offset 684*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.menu_layer_get_layer")]
pub unsafe extern "C" fn menu_layer_get_layer(menu_layer: *const MenuLayer) -> *mut Layer {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =684", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the ScrollLayer of the \ref MenuLayer, which is the layer responsible for
! the scrolling of the \ref MenuLayer.
! @param menu_layer Pointer to the \ref MenuLayer for which to get the ScrollLayer
! @return The ScrollLayer of the \ref MenuLayer.*/
/** menu_layer_get_scroll_layer
  -> Added in 0
gneiss: Vtable Slot 172, Offset 688*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.menu_layer_get_scroll_layer")]
pub unsafe extern "C" fn menu_layer_get_scroll_layer(
	menu_layer: *const MenuLayer,
) -> *mut ScrollLayer {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =688", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the callbacks for the MenuLayer.
! @param menu_layer Pointer to the \ref MenuLayer for which to set the callbacks
! and callback context.
! @param callback_context The new callback context. This is passed into each
! of the callbacks and can be set to point to application provided data.
! @param callbacks The new callbacks for the \ref MenuLayer. The storage for this
! data structure must be long lived. Therefore, it cannot be stack-allocated.
! @see MenuLayerCallbacks*/
/** menu_layer_set_callbacks
  -> Added in 54
gneiss: Vtable Slot 522, Offset 2088*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.menu_layer_set_callbacks")]
pub unsafe extern "C" fn menu_layer_set_callbacks(
	menu_layer: *mut MenuLayer,
	callback_context: *mut ::core::ffi::c_void,
	callbacks: MenuLayerCallbacks,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2088", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Convenience function to set the \ref ClickConfigProvider callback on the
! given window to the \ref MenuLayer internal click config provider. This internal
! click configuration provider, will set up the default UP & DOWN
! scrolling / menu item selection behavior.
! This function calls \ref scroll_layer_set_click_config_onto_window to
! accomplish this.
!
! Click and long click events for the SELECT button can be handled by
! installing the appropriate callbacks using \ref menu_layer_set_callbacks().
! This is a deviation from the usual click configuration provider pattern.
! @param menu_layer The \ref MenuLayer that needs to receive click events.
! @param window The window for which to set the click configuration.
! @see \ref Clicks
! @see \ref window_set_click_config_provider_with_context()
! @see \ref scroll_layer_set_click_config_onto_window()*/
/** menu_layer_set_click_config_onto_window
  -> Added in 0
gneiss: Vtable Slot 176, Offset 704*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.menu_layer_set_click_config_onto_window")]
pub unsafe extern "C" fn menu_layer_set_click_config_onto_window(
	menu_layer: *mut MenuLayer,
	window: *mut Window,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =704", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Don't align or update the scroll offset of the \ref MenuLayer.
pub const MenuRowAlign_MenuRowAlignNone: MenuRowAlign = 0;
/**! Scroll the contents of the \ref MenuLayer in such way that the selected row
! is centered relative to the visible area.*/
pub const MenuRowAlign_MenuRowAlignCenter: MenuRowAlign = 1;
/**! Scroll the contents of the \ref MenuLayer in such way that the selected row
! is at the top of the visible area.*/
pub const MenuRowAlign_MenuRowAlignTop: MenuRowAlign = 2;
/**! Scroll the contents of the \ref MenuLayer in such way that the selected row
! is at the bottom of the visible area.*/
pub const MenuRowAlign_MenuRowAlignBottom: MenuRowAlign = 3;
/**! Values to specify how a (selected) row should be aligned relative to the
! visible area of the \ref MenuLayer.*/
pub type MenuRowAlign = ::core::ffi::c_uchar;
/**! Selects the next or previous item, relative to the current selection.
! @param menu_layer The \ref MenuLayer for which to select the next item
! @param up Supply `false` to select the next item in the list (downwards),
! or `true` to select the previous item in the list (upwards).
! @param scroll_align The alignment of the new selection
! @param animated Supply `true` to animate changing the selection, or `false`
! to change the selection instantly.
! @note If there is no next/previous item, this function is a no-op.*/
/** menu_layer_set_selected_next
  -> Added in 0
gneiss: Vtable Slot 178, Offset 712*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.menu_layer_set_selected_next")]
pub unsafe extern "C" fn menu_layer_set_selected_next(
	menu_layer: *mut MenuLayer,
	up: bool,
	scroll_align: MenuRowAlign,
	animated: bool,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =712", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Selects the item with given \ref MenuIndex.
! @param menu_layer The \ref MenuLayer for which to change the selection
! @param index The index of the item to select
! @param scroll_align The alignment of the new selection
! @param animated Supply `true` to animate changing the selection, or `false`
! to change the selection instantly.
! @note If the section and/or row index exceeds the avaible number of sections
! or resp. rows, the exceeding index/indices will be capped, effectively
! selecting the last section and/or row, resp.*/
/** menu_layer_set_selected_index
  -> Added in 0
gneiss: Vtable Slot 177, Offset 708*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.menu_layer_set_selected_index")]
pub unsafe extern "C" fn menu_layer_set_selected_index(
	menu_layer: *mut MenuLayer,
	index: MenuIndex,
	scroll_align: MenuRowAlign,
	animated: bool,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =708", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the MenuIndex of the currently selected menu item.
! @param menu_layer The \ref MenuLayer for which to get the current selected index.
! @see menu_cell_layer_is_highlighted
! @note This function should not be used to determine whether a cell should be
! highlighted or not. See \ref menu_cell_layer_is_highlighted for more
! information.*/
/** menu_layer_get_selected_index
  -> Added in 0
gneiss: Vtable Slot 173, Offset 692*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.menu_layer_get_selected_index")]
pub unsafe extern "C" fn menu_layer_get_selected_index(menu_layer: *const MenuLayer) -> MenuIndex {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =692", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Reloads the data of the menu. This causes the menu to re-request the menu
! item data, by calling the relevant callbacks.
! The current selection and scroll position will not be changed. See the
! note with \ref menu_layer_set_selected_index() for the behavior if the
! old selection is no longer valid.
! @param menu_layer The \ref MenuLayer for which to reload the data.*/
/** menu_layer_reload_data
  -> Added in 0
gneiss: Vtable Slot 174, Offset 696*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.menu_layer_reload_data")]
pub unsafe extern "C" fn menu_layer_reload_data(menu_layer: *mut MenuLayer) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =696", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Returns whether or not the given cell layer is highlighted.
! Using this for determining highlight behaviour is preferable to using
! \ref menu_layer_get_selected_index. Row drawing callbacks may be invoked multiple
! times with a different highlight status on the same cell in order to handle partially
! highlighted cells during animation.
! @param cell_layer The \ref Layer for the cell to check highlight status.
! @return true if the given cell layer is highlighted in the menu.*/
/** menu_cell_layer_is_highlighted
  -> Added in 44
gneiss: Vtable Slot 459, Offset 1836*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.menu_cell_layer_is_highlighted")]
pub unsafe extern "C" fn menu_cell_layer_is_highlighted(cell_layer: *const Layer) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1836", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set the default colors to be used for cells when it is in a normal state (not highlighted).
! The GContext's text and fill colors will be set appropriately prior to calling the `.draw_row`
! callback.
! If this function is not explicitly called on a \ref MenuLayer, it will default to white
! background with black foreground.
! @param menu_layer The \ref MenuLayer for which to set the colors.
! @param background The color to be used for the background of the cell.
! @param foreground The color to be used for the foreground and text of the cell.
! @see \ref menu_layer_set_highlight_colors*/
/** menu_layer_set_normal_colors
  -> Added in 53
gneiss: Vtable Slot 521, Offset 2084*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.menu_layer_set_normal_colors")]
pub unsafe extern "C" fn menu_layer_set_normal_colors(
	menu_layer: *mut MenuLayer,
	background: GColor,
	foreground: GColor,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2084", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set the default colors to be used for cells when it is in a highlighted state.
! The GContext's text and fill colors will be set appropriately prior to calling the `.draw_row`
! callback.
! If this function is not explicitly called on a \ref MenuLayer, it will default to black
! background with white foreground.
! @param menu_layer The \ref MenuLayer for which to set the colors.
! @param background The color to be used for the background of the cell.
! @param foreground The color to be used for the foreground and text of the cell.
! @see \ref menu_layer_set_normal_colors*/
/** menu_layer_set_highlight_colors
  -> Added in 53
gneiss: Vtable Slot 520, Offset 2080*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.menu_layer_set_highlight_colors")]
pub unsafe extern "C" fn menu_layer_set_highlight_colors(
	menu_layer: *mut MenuLayer,
	background: GColor,
	foreground: GColor,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2080", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! This enables or disables padding at the bottom of the \ref MenuLayer.
! Padding at the bottom of the layer keeps the bottom item from being at the very bottom of the
! screen.
! Padding is turned on by default for all MenuLayers.
! The color of the padded area will be the background color set using
! \ref menu_layer_set_normal_colors(). To color the padding a different color, use
! \ref MenuLayerDrawBackgroundCallback.
! @param menu_layer The menu layer for which to enable or disable the padding.
! @param enable True = enable padding, False = disable padding.*/
/** menu_layer_pad_bottom_enable
  -> Added in 56
gneiss: Vtable Slot 523, Offset 2092*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.menu_layer_pad_bottom_enable")]
pub unsafe extern "C" fn menu_layer_pad_bottom_enable(menu_layer: *mut MenuLayer, enable: bool) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2092", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! True, if the \ref MenuLayer generally scrolls such that the selected row is in the center.
! @see \ref menu_layer_set_center_focused*/
/** menu_layer_get_center_focused
  -> Added in 72
gneiss: Vtable Slot 578, Offset 2312*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.menu_layer_get_center_focused")]
pub unsafe extern "C" fn menu_layer_get_center_focused(menu_layer: *mut MenuLayer) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2312", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Controls if the \ref MenuLayer generally scrolls such that the selected row is in the center.
! For platforms with a round display (PBL_ROUND) the default is true,
! otherwise false is the default
! @param menu_layer The menu layer for which to enable or disable the behavior.
! @param center_focused true = enable the mode, false = disable it.
! @see \ref menu_layer_get_center_focused*/
/** menu_layer_set_center_focused
  -> Added in 72
gneiss: Vtable Slot 579, Offset 2316*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.menu_layer_set_center_focused")]
pub unsafe extern "C" fn menu_layer_set_center_focused(
	menu_layer: *mut MenuLayer,
	center_focused: bool,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2316", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Returns whether or not the specified cell index is currently selected.
! @param menu_layer The \ref MenuLayer to use when determining if the index is selected.
! @param index The \ref MenuIndex of the cell to check for selection.
! @note This function should not be used to determine whether a cell is highlighted or not.
! See \ref menu_cell_layer_is_highlighted for more information.*/
/** menu_layer_is_index_selected
  -> Added in 76
gneiss: Vtable Slot 598, Offset 2392*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.menu_layer_is_index_selected")]
pub unsafe extern "C" fn menu_layer_is_index_selected(
	menu_layer: *const MenuLayer,
	index: *mut MenuIndex,
) -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2392", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @addtogroup SimpleMenuLayer
! \brief Wrapper around \ref MenuLayer, that uses static data to display a
! list menu.
!
! ![](simple_menu_layer.png)
! @{*/
#[repr(C)]
#[derive(Debug)]
pub struct SimpleMenuLayer {
	_unused: [u8; 0],
}
/**! Function signature for the callback to handle the event that a user hits
! the SELECT button.
! @param index The row index of the item
! @param context The callback context*/
pub type SimpleMenuLayerSelectCallback = ::core::option::Option<
	unsafe extern "C" fn(index: ::core::ffi::c_int, context: *mut ::core::ffi::c_void),
>;
///! Data structure containing the information of a menu item.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SimpleMenuItem {
	///! The title of the menu item. Required.
	pub title:    *const ::core::ffi::c_char,
	///! The subtitle of the menu item. Optional, leave `NULL` if unused.
	pub subtitle: *const ::core::ffi::c_char,
	///! The icon of the menu item. Optional, leave `NULL` if unused.
	pub icon:     *mut GBitmap,
	/**! The callback that needs to be called upon a click on the SELECT button.
	! Optional, leave `NULL` if unused.*/
	pub callback: SimpleMenuLayerSelectCallback,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of SimpleMenuItem"][::core::mem::size_of::<SimpleMenuItem>() - 16usize];
	["Alignment of SimpleMenuItem"][::core::mem::align_of::<SimpleMenuItem>() - 4usize];
	["Offset of field: SimpleMenuItem::title"]
		[::core::mem::offset_of!(SimpleMenuItem, title) - 0usize];
	["Offset of field: SimpleMenuItem::subtitle"]
		[::core::mem::offset_of!(SimpleMenuItem, subtitle) - 4usize];
	["Offset of field: SimpleMenuItem::icon"]
		[::core::mem::offset_of!(SimpleMenuItem, icon) - 8usize];
	["Offset of field: SimpleMenuItem::callback"]
		[::core::mem::offset_of!(SimpleMenuItem, callback) - 12usize];
};
///! Data structure containing the information of a menu section.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SimpleMenuSection {
	///! Title of the section. Optional, leave `NULL` if unused.
	pub title:     *const ::core::ffi::c_char,
	///! Array of items in the section.
	pub items:     *const SimpleMenuItem,
	///! Number of items in the `.items` array.
	pub num_items: u32,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of SimpleMenuSection"][::core::mem::size_of::<SimpleMenuSection>() - 12usize];
	["Alignment of SimpleMenuSection"][::core::mem::align_of::<SimpleMenuSection>() - 4usize];
	["Offset of field: SimpleMenuSection::title"]
		[::core::mem::offset_of!(SimpleMenuSection, title) - 0usize];
	["Offset of field: SimpleMenuSection::items"]
		[::core::mem::offset_of!(SimpleMenuSection, items) - 4usize];
	["Offset of field: SimpleMenuSection::num_items"]
		[::core::mem::offset_of!(SimpleMenuSection, num_items) - 8usize];
};
/**! Creates a new SimpleMenuLayer on the heap and initializes it.
! It also sets the internal click configuration provider onto given window.
! @param frame The frame at which to initialize the menu
! @param window The window onto which to set the click configuration provider
! @param sections Array with sections that need to be displayed in the menu
! @param num_sections The number of sections in the `sections` array.
! @param callback_context Pointer to application specific data, that is passed
! into the callbacks.
! @note The `sections` array is not deep-copied and can therefore not be stack
! allocated, but needs to be backed by long-lived storage.
! @note This function does not add the menu's layer to the window.
! @return A pointer to the SimpleMenuLayer. `NULL` if the SimpleMenuLayer could not
! be created*/
/** simple_menu_layer_create
  -> Added in 0
gneiss: Vtable Slot 233, Offset 932*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.simple_menu_layer_create")]
pub unsafe extern "C" fn simple_menu_layer_create(
	frame: GRect,
	window: *mut Window,
	sections: *const SimpleMenuSection,
	num_sections: i32,
	callback_context: *mut ::core::ffi::c_void,
) -> *mut SimpleMenuLayer {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =932", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Destroys a SimpleMenuLayer previously created by simple_menu_layer_create.
/** simple_menu_layer_destroy
  -> Added in 0
gneiss: Vtable Slot 234, Offset 936*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.simple_menu_layer_destroy")]
pub unsafe extern "C" fn simple_menu_layer_destroy(menu_layer: *mut SimpleMenuLayer) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =936", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the "root" Layer of the simple menu layer, which is the parent for the
! sub-layers used for its implementation.
! @param simple_menu Pointer to the SimpleMenuLayer for which to get the
! "root" Layer
! @return The "root" Layer of the menu layer.*/
/** simple_menu_layer_get_layer
  -> Added in 0
gneiss: Vtable Slot 235, Offset 940*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.simple_menu_layer_get_layer")]
pub unsafe extern "C" fn simple_menu_layer_get_layer(
	simple_menu: *const SimpleMenuLayer,
) -> *mut Layer {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =940", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the row index of the currently selection menu item.
! @param simple_menu The SimpleMenuLayer for which to get the current
! selected row index.*/
/** simple_menu_layer_get_selected_index
  -> Added in 0
gneiss: Vtable Slot 236, Offset 944*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.simple_menu_layer_get_selected_index")]
pub unsafe extern "C" fn simple_menu_layer_get_selected_index(
	simple_menu: *const SimpleMenuLayer,
) -> ::core::ffi::c_int {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =944", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Selects the item in the first section at given row index.
! @param simple_menu The SimpleMenuLayer for which to change the selection
! @param index The row index of the item to select
! @param animated Supply `true` to animate changing the selection, or `false`
! to change the selection instantly.*/
/** simple_menu_layer_set_selected_index
  -> Added in 0
gneiss: Vtable Slot 237, Offset 948*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.simple_menu_layer_set_selected_index")]
pub unsafe extern "C" fn simple_menu_layer_set_selected_index(
	simple_menu: *mut SimpleMenuLayer,
	index: i32,
	animated: bool,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =948", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @param simple_menu The \ref SimpleMenuLayer to get the \ref MenuLayer from.
! @return The \ref MenuLayer.*/
/** simple_menu_layer_get_menu_layer
  -> Added in 8
gneiss: Vtable Slot 316, Offset 1264*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.simple_menu_layer_get_menu_layer")]
pub unsafe extern "C" fn simple_menu_layer_get_menu_layer(
	simple_menu: *mut SimpleMenuLayer,
) -> *mut MenuLayer {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1264", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
pub const ActionBarLayerIconPressAnimation_ActionBarLayerIconPressAnimationNone:
	ActionBarLayerIconPressAnimation = 0;
pub const ActionBarLayerIconPressAnimation_ActionBarLayerIconPressAnimationMoveLeft:
	ActionBarLayerIconPressAnimation = 1;
pub const ActionBarLayerIconPressAnimation_ActionBarLayerIconPressAnimationMoveUp:
	ActionBarLayerIconPressAnimation = 2;
pub const ActionBarLayerIconPressAnimation_ActionBarLayerIconPressAnimationMoveRight:
	ActionBarLayerIconPressAnimation = 3;
pub const ActionBarLayerIconPressAnimation_ActionBarLayerIconPressAnimationMoveDown:
	ActionBarLayerIconPressAnimation = 4;
pub type ActionBarLayerIconPressAnimation = ::core::ffi::c_uchar;
#[repr(C)]
#[derive(Debug)]
pub struct ActionBarLayer {
	_unused: [u8; 0],
}
/**! Creates a new ActionBarLayer on the heap and initalizes it with the default values.
! * Background color: \ref GColorBlack
! * No click configuration provider (`NULL`)
! * No icons
! * Not added to / associated with any window, thus not catching any button input yet.
! @return A pointer to the ActionBarLayer. `NULL` if the ActionBarLayer could not
! be created*/
/** action_bar_layer_create
  -> Added in 42
gneiss: Vtable Slot 448, Offset 1792*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_bar_layer_create")]
pub unsafe extern "C" fn action_bar_layer_create() -> *mut ActionBarLayer {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1792", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Destroys a ActionBarLayer previously created by action_bar_layer_create
/** action_bar_layer_destroy
  -> Added in 42
gneiss: Vtable Slot 449, Offset 1796*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_bar_layer_destroy")]
pub unsafe extern "C" fn action_bar_layer_destroy(action_bar_layer: *mut ActionBarLayer) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1796", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the "root" Layer of the action bar layer, which is the parent for the sub-
! layers used for its implementation.
! @param action_bar_layer Pointer to the ActionBarLayer for which to get the "root" Layer
! @return The "root" Layer of the action bar layer.*/
/** action_bar_layer_get_layer
  -> Added in 42
gneiss: Vtable Slot 450, Offset 1800*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_bar_layer_get_layer")]
pub unsafe extern "C" fn action_bar_layer_get_layer(
	action_bar_layer: *mut ActionBarLayer,
) -> *mut Layer {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1800", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the context parameter, which will be passed in to \ref ClickHandler
! callbacks and the \ref ClickConfigProvider callback of the action bar.
! @note By default, a pointer to the action bar itself is passed in, if the
! context has not been set or if it has been set to `NULL`.
! @param action_bar The action bar for which to assign the new context
! @param context The new context
! @see action_bar_layer_set_click_config_provider()
! @see \ref Clicks*/
/** action_bar_layer_set_context
  -> Added in 42
gneiss: Vtable Slot 454, Offset 1816*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_bar_layer_set_context")]
pub unsafe extern "C" fn action_bar_layer_set_context(
	action_bar: *mut ActionBarLayer,
	context: *mut ::core::ffi::c_void,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1816", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the click configuration provider callback of the action bar.
! In this callback your application can associate handlers to the different
! types of click events for each of the buttons, see \ref Clicks.
! @note If the action bar had already been added to a window and the window
! is currently on-screen, the click configuration provider will be called
! before this function returns. Otherwise, it will be called by the system
! when the window becomes on-screen.
! @note The `.raw` handlers cannot be used without breaking the automatic
! highlighting of the segment of the action bar that for which a button is
! @see action_bar_layer_set_icon()
! @param action_bar The action bar for which to assign a new click
! configuration provider
! @param click_config_provider The new click configuration provider*/
/** action_bar_layer_set_click_config_provider
  -> Added in 42
gneiss: Vtable Slot 453, Offset 1812*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_bar_layer_set_click_config_provider")]
pub unsafe extern "C" fn action_bar_layer_set_click_config_provider(
	action_bar: *mut ActionBarLayer,
	click_config_provider: ClickConfigProvider,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1812", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets an action bar icon onto one of the 3 slots as identified by `button_id`.
! Only \ref BUTTON_ID_UP, \ref BUTTON_ID_SELECT and \ref BUTTON_ID_DOWN can be
! used. The transition will not be animated.
! Whenever an icon is set, the click configuration provider will be
! called, to give the application the opportunity to reconfigure the button
! interaction.
! @param action_bar The action bar for which to set the new icon
! @param button_id The identifier of the button for which to set the icon
! @param icon Pointer to the \ref GBitmap icon
! @see action_bar_layer_set_icon_animated()
! @see action_bar_layer_set_icon_press_animation()
! @see action_bar_layer_set_click_config_provider()*/
/** action_bar_layer_set_icon
  -> Added in 42
gneiss: Vtable Slot 455, Offset 1820*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_bar_layer_set_icon")]
pub unsafe extern "C" fn action_bar_layer_set_icon(
	action_bar: *mut ActionBarLayer,
	button_id: ButtonId,
	icon: *const GBitmap,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1820", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Convenience function to clear out an existing icon.
! All it does is call `action_bar_layer_set_icon(action_bar, button_id, NULL)`
! @param action_bar The action bar for which to clear an icon
! @param button_id The identifier of the button for which to clear the icon
! @see action_bar_layer_set_icon()*/
/** action_bar_layer_clear_icon
  -> Added in 42
gneiss: Vtable Slot 447, Offset 1788*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_bar_layer_clear_icon")]
pub unsafe extern "C" fn action_bar_layer_clear_icon(
	action_bar: *mut ActionBarLayer,
	button_id: ButtonId,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1788", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Adds the action bar's layer on top of the window's root layer. It also
! adjusts the layout of the action bar to match the geometry of the window it
! gets added to.
! Lastly, it calls \ref window_set_click_config_provider_with_context() on
! the window to set it up to work with the internal callback and raw click
! handlers of the action bar, to enable the highlighting of the section of the
! action bar when the user presses a button.
! @note After this call, do not use
! \ref window_set_click_config_provider_with_context() with the window that
! the action bar has been added to (this would de-associate the action bar's
! click config provider and context). Instead use
! \ref action_bar_layer_set_click_config_provider() and
! \ref action_bar_layer_set_context() to register the click configuration
! provider to configure the buttons actions.
! @note It is advised to call this is in the window's `.load` or `.appear`
! handler. Make sure to call \ref action_bar_layer_remove_from_window() in the
! window's `.unload` or `.disappear` handler.
! @note Adding additional layers to the window's root layer after this calll
! can occlude the action bar.
! @param action_bar The action bar to associate with the window
! @param window The window with which the action bar is to be associated*/
/** action_bar_layer_add_to_window
  -> Added in 42
gneiss: Vtable Slot 446, Offset 1784*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_bar_layer_add_to_window")]
pub unsafe extern "C" fn action_bar_layer_add_to_window(
	action_bar: *mut ActionBarLayer,
	window: *mut Window,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1784", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Removes the action bar from the window and unconfigures the window's
! click configuration provider. `NULL` is set as the window's new click config
! provider and also as its callback context. If it has not been added to a
! window before, this function is a no-op.
! @param action_bar The action bar to de-associate from its current window*/
/** action_bar_layer_remove_from_window
  -> Added in 42
gneiss: Vtable Slot 451, Offset 1804*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_bar_layer_remove_from_window")]
pub unsafe extern "C" fn action_bar_layer_remove_from_window(action_bar: *mut ActionBarLayer) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1804", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the background color of the action bar. Defaults to \ref GColorBlack.
! The action bar's layer is automatically marked dirty.
! @param action_bar The action bar of which to set the background color
! @param background_color The new background color*/
/** action_bar_layer_set_background_color
  -> Added in 42
gneiss: Vtable Slot 452, Offset 1808*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_bar_layer_set_background_color")]
pub unsafe extern "C" fn action_bar_layer_set_background_color(
	action_bar: *mut ActionBarLayer,
	background_color: GColor,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1808", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets an action bar icon onto one of the 3 slots as identified by `button_id`.
! Only \ref BUTTON_ID_UP, \ref BUTTON_ID_SELECT and \ref BUTTON_ID_DOWN can be
! used. Optionally, if `animated` is `true`, the transition will be animated.
! Whenever an icon is set, the click configuration provider will be called,
! to give the application the opportunity to reconfigure the button interaction.
! @param action_bar The action bar for which to set the new icon
! @param button_id The identifier of the button for which to set the icon
! @param icon Pointer to the \ref GBitmap icon
! @param animated True = animate the transition, False = do not animate the transition
! @see action_bar_layer_set_icon()
! @see action_bar_layer_set_icon_press_animation()
! @see action_bar_layer_set_click_config_provider()*/
/** action_bar_layer_set_icon_animated
  -> Added in 42
gneiss: Vtable Slot 456, Offset 1824*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_bar_layer_set_icon_animated")]
pub unsafe extern "C" fn action_bar_layer_set_icon_animated(
	action_bar: *mut ActionBarLayer,
	button_id: ButtonId,
	icon: *const GBitmap,
	animated: bool,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1824", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the animation to use while a button is pressed on an ActionBarLayer.
! By default we use ActionBarLayerIconPressAnimationMoveLeft
! @param action_bar The action bar for which to set the press animation
! @param button_id The button for which to set the press animation
! @param animation The animation to use.
! @see action_bar_layer_set_icon_animated()
! @see action_bar_layer_set_click_config_provider()*/
/** action_bar_layer_set_icon_press_animation
  -> Added in 46
gneiss: Vtable Slot 461, Offset 1844*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_bar_layer_set_icon_press_animation")]
pub unsafe extern "C" fn action_bar_layer_set_icon_press_animation(
	action_bar: *mut ActionBarLayer,
	button_id: ButtonId,
	animation: ActionBarLayerIconPressAnimation,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1844", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @addtogroup StatusBarLayer
! \brief Layer that serves as a configurable status bar.
! @{*/
#[repr(C)]
#[derive(Debug)]
pub struct StatusBarLayer {
	_unused: [u8; 0],
}
///! The default mode. No separator will be shown.
pub const StatusBarLayerSeparatorMode_StatusBarLayerSeparatorModeNone: StatusBarLayerSeparatorMode =
	0;
///! A dotted separator at the bottom of the status bar.
pub const StatusBarLayerSeparatorMode_StatusBarLayerSeparatorModeDotted:
	StatusBarLayerSeparatorMode = 1;
///! Values that are used to indicate the different status bar separator modes.
pub type StatusBarLayerSeparatorMode = ::core::ffi::c_uchar;
/**! Creates a new StatusBarLayer on the heap and initializes it with the default values.
!
! * Text color: \ref GColorBlack
! * Background color: \ref GColorWhite
! * Frame: `GRect(0, 0, screen_width, STATUS_BAR_LAYER_HEIGHT)`
! The status bar is automatically marked dirty after this operation.
! You can call \ref layer_set_frame() to create a StatusBarLayer of a different width.
!
! \code{.c}
! // Change the status bar width to make space for the action bar
! int16_t width = layer_get_bounds(root_layer).size.w - ACTION_BAR_WIDTH;
! GRect frame = GRect(0, 0, width, STATUS_BAR_LAYER_HEIGHT);
! layer_set_frame(status_bar_layer_get_layer(status_bar), frame);
! layer_add_child(root_layer, status_bar_layer_get_layer(status_bar));
! \endcode
! @return A pointer to the StatusBarLayer, which will be allocated to the heap,
! `NULL` if the StatusBarLayer could not be created*/
/** status_bar_layer_create
  -> Added in 57
gneiss: Vtable Slot 524, Offset 2096*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.status_bar_layer_create")]
pub unsafe extern "C" fn status_bar_layer_create() -> *mut StatusBarLayer {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2096", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Destroys a StatusBarLayer previously created by status_bar_layer_create.
! @param status_bar_layer The StatusBarLayer to destroy*/
/** status_bar_layer_destroy
  -> Added in 57
gneiss: Vtable Slot 525, Offset 2100*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.status_bar_layer_destroy")]
pub unsafe extern "C" fn status_bar_layer_destroy(status_bar_layer: *mut StatusBarLayer) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2100", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the "root" Layer of the status bar, which is the parent for the sub-
! layers used for its implementation.
! @param status_bar_layer Pointer to the StatusBarLayer for which to get the "root" Layer
! @return The "root" Layer of the status bar.
! @note The result is always equal to `(Layer *) status_bar_layer`.*/
/** status_bar_layer_get_layer
  -> Added in 57
gneiss: Vtable Slot 528, Offset 2112*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.status_bar_layer_get_layer")]
pub unsafe extern "C" fn status_bar_layer_get_layer(
	status_bar_layer: *mut StatusBarLayer,
) -> *mut Layer {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2112", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets background color of StatusBarLayer
! @param status_bar_layer The StatusBarLayer of which to get the color
! @return GColor of background color property*/
/** status_bar_layer_get_background_color
  -> Added in 57
gneiss: Vtable Slot 526, Offset 2104*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.status_bar_layer_get_background_color")]
pub unsafe extern "C" fn status_bar_layer_get_background_color(
	status_bar_layer: *const StatusBarLayer,
) -> GColor {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2104", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets foreground color of StatusBarLayer
! @param status_bar_layer The StatusBarLayer of which to get the color
! @return GColor of foreground color property*/
/** status_bar_layer_get_foreground_color
  -> Added in 57
gneiss: Vtable Slot 527, Offset 2108*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.status_bar_layer_get_foreground_color")]
pub unsafe extern "C" fn status_bar_layer_get_foreground_color(
	status_bar_layer: *const StatusBarLayer,
) -> GColor {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2108", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the background and foreground colors of StatusBarLayer
! @param status_bar_layer The StatusBarLayer of which to set the colors
! @param background The new \ref GColor to set for background
! @param foreground The new \ref GColor to set for text and other foreground elements*/
/** status_bar_layer_set_colors
  -> Added in 57
gneiss: Vtable Slot 529, Offset 2116*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.status_bar_layer_set_colors")]
pub unsafe extern "C" fn status_bar_layer_set_colors(
	status_bar_layer: *mut StatusBarLayer,
	background: GColor,
	foreground: GColor,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2116", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the mode of the StatusBarLayer separator, to help divide it from content
! @param status_bar_layer The StatusBarLayer of which to set the separator mode
! @param mode Determines the separator mode*/
/** status_bar_layer_set_separator_mode
  -> Added in 57
gneiss: Vtable Slot 530, Offset 2120*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.status_bar_layer_set_separator_mode")]
pub unsafe extern "C" fn status_bar_layer_set_separator_mode(
	status_bar_layer: *mut StatusBarLayer,
	mode: StatusBarLayerSeparatorMode,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2120", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @addtogroup BitmapLayer
! \brief Layer that displays a bitmap image.
!
! ![](bitmap_layer.png)
! BitmapLayer is a Layer subtype that draws a GBitmap within its frame. It uses an alignment property
! to specify how to position the bitmap image within its frame. Optionally, when the
! background color is not GColorClear, it draws a solid background color behind the
! bitmap image, filling areas of the frame that are not covered by the bitmap image.
! Lastly, using the compositing mode property of the BitmapLayer, determines the way the
! bitmap image is drawn on top of what is underneath it (either the background color, or
! the layers beneath it).
!
! <h3>Inside the Implementation</h3>
! The implementation of BitmapLayer is fairly straightforward and relies heavily on the
! functionality as exposed by the core drawing functions (see \ref Drawing).
! \ref BitmapLayer's drawing callback uses \ref graphics_draw_bitmap_in_rect()
! to perform the actual drawing of the \ref GBitmap. It uses \ref grect_align() to perform
! the layout of the image and it uses \ref graphics_fill_rect() to draw the background plane.
! @{*/
#[repr(C)]
#[derive(Debug)]
pub struct BitmapLayer {
	_unused: [u8; 0],
}
/**! Creates a new bitmap layer on the heap and initalizes it the default values.
!
! * Bitmap: `NULL` (none)
! * Background color: \ref GColorClear
! * Compositing mode: \ref GCompOpAssign
! * Clips: `true`
! @return A pointer to the BitmapLayer. `NULL` if the BitmapLayer could not
! be created*/
/** bitmap_layer_create
  -> Added in 0
gneiss: Vtable Slot 56, Offset 224*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.bitmap_layer_create")]
pub unsafe extern "C" fn bitmap_layer_create(frame: GRect) -> *mut BitmapLayer {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =224", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Destroys a window previously created by bitmap_layer_create
/** bitmap_layer_destroy
  -> Added in 0
gneiss: Vtable Slot 57, Offset 228*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.bitmap_layer_destroy")]
pub unsafe extern "C" fn bitmap_layer_destroy(bitmap_layer: *mut BitmapLayer) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =228", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the "root" Layer of the bitmap layer, which is the parent for the sub-
! layers used for its implementation.
! @param bitmap_layer Pointer to the BitmapLayer for which to get the "root" Layer
! @return The "root" Layer of the bitmap layer.*/
/** bitmap_layer_get_layer
  -> Added in 0
gneiss: Vtable Slot 58, Offset 232*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.bitmap_layer_get_layer")]
pub unsafe extern "C" fn bitmap_layer_get_layer(bitmap_layer: *const BitmapLayer) -> *mut Layer {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =232", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the pointer to the bitmap image that the BitmapLayer is using.
!
! @param bitmap_layer The BitmapLayer for which to get the bitmap image
! @return A pointer to the bitmap image that the BitmapLayer is using*/
/** bitmap_layer_get_bitmap
  -> Added in 10
gneiss: Vtable Slot 319, Offset 1276*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.bitmap_layer_get_bitmap")]
pub unsafe extern "C" fn bitmap_layer_get_bitmap(bitmap_layer: *mut BitmapLayer) -> *const GBitmap {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1276", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the bitmap onto the BitmapLayer. The bitmap is set by reference (no deep
! copy), thus the caller of this function has to make sure the bitmap is kept
! in memory.
!
! The bitmap layer is automatically marked dirty after this operation.
! @param bitmap_layer The BitmapLayer for which to set the bitmap image
! @param bitmap The new \ref GBitmap to set onto the BitmapLayer*/
/** bitmap_layer_set_bitmap
  -> Added in 0
gneiss: Vtable Slot 61, Offset 244*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.bitmap_layer_set_bitmap")]
pub unsafe extern "C" fn bitmap_layer_set_bitmap(
	bitmap_layer: *mut BitmapLayer,
	bitmap: *const GBitmap,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =244", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the alignment of the image to draw with in frame of the BitmapLayer.
! The aligment parameter specifies which edges of the bitmap should overlap
! with the frame of the BitmapLayer.
! If the bitmap is smaller than the frame of the BitmapLayer, the background
! is filled with the background color.
!
! The bitmap layer is automatically marked dirty after this operation.
! @param bitmap_layer The BitmapLayer for which to set the aligment
! @param alignment The new alignment for the image inside the BitmapLayer*/
/** bitmap_layer_set_alignment
  -> Added in 0
gneiss: Vtable Slot 59, Offset 236*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.bitmap_layer_set_alignment")]
pub unsafe extern "C" fn bitmap_layer_set_alignment(
	bitmap_layer: *mut BitmapLayer,
	alignment: GAlign,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =236", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the background color of bounding box that will be drawn behind the image
! of the BitmapLayer.
!
! The bitmap layer is automatically marked dirty after this operation.
! @param bitmap_layer The BitmapLayer for which to set the background color
! @param color The new \ref GColor to set the background to
! @see \ref bitmap_layer_set_compositing_mode for enabling transparency*/
/** bitmap_layer_set_background_color
  -> Added in 30
gneiss: Vtable Slot 370, Offset 1480*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.bitmap_layer_set_background_color")]
pub unsafe extern "C" fn bitmap_layer_set_background_color(
	bitmap_layer: *mut BitmapLayer,
	color: GColor,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1480", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the compositing mode of how the bitmap image is composited onto the
! BitmapLayer's background plane, or how it is composited onto what has been
! drawn beneath the BitmapLayer.
!
! The compositing mode only affects the drawing of the bitmap and not the
! drawing of the background color.
!
! For black&white platforms, there is no notion of "transparency" in the graphics system.
! However, the effect of transparency can be created by masking and using compositing modes.
!
! For color platforms, when drawing \ref GBitmap images, \ref GCompOpSet is
! required to apply any transparency.
!
! The bitmap layer is automatically marked dirty after this operation.
! @param bitmap_layer The BitmapLayer for which to set the compositing mode
! @param mode The compositing mode to set
! @see See \ref GCompOp for visual examples of the different compositing modes.*/
/** bitmap_layer_set_compositing_mode
  -> Added in 0
gneiss: Vtable Slot 62, Offset 248*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.bitmap_layer_set_compositing_mode")]
pub unsafe extern "C" fn bitmap_layer_set_compositing_mode(
	bitmap_layer: *mut BitmapLayer,
	mode: GCompOp,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =248", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @addtogroup RotBitmapLayer
! \brief Layer that displays a rotated bitmap image.
!
! A RotBitmapLayer is like a \ref BitmapLayer but has the ability to be rotated (by default, around its center). The amount of rotation
! is specified using \ref rot_bitmap_layer_set_angle() or \ref rot_bitmap_layer_increment_angle(). The rotation argument
! to those functions is specified as an amount of clockwise rotation, where the value 0x10000 represents a full 360 degree
! rotation and 0 represent no rotation, and it scales linearly between those values, just like \ref sin_lookup.
!
! The center of rotation in the source bitmap is always placed at the center of the RotBitmapLayer and the size of the RotBitmapLayer
! is automatically calculated so that the entire Bitmap can fit in at all rotation angles.
!
! For example, if the image is 10px wide and high, the RotBitmapLayer will be 14px wide ( sqrt(10^2+10^2) ).
!
! By default, the center of rotation in the source bitmap is the center of the bitmap but you can call \ref rot_bitmap_set_src_ic() to change the
! center of rotation.
!
! @note RotBitmapLayer has performance limitations that can degrade user
! experience (see \ref graphics_draw_rotated_bitmap). Use sparingly.
! @{*/
#[repr(C)]
#[derive(Debug)]
pub struct RotBitmapLayer {
	_unused: [u8; 0],
}
/**! Creates a new RotBitmapLayer on the heap and initializes it with the default values:
!  * Angle: 0
!  * Compositing mode: \ref GCompOpAssign
!  * Corner clip color: \ref GColorClear
!
! @param bitmap The bitmap to display in this RotBitmapLayer
! @return A pointer to the RotBitmapLayer. `NULL` if the RotBitmapLayer could not
! be created*/
/** rot_bitmap_layer_create
  -> Added in 0
gneiss: Vtable Slot 210, Offset 840*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.rot_bitmap_layer_create")]
pub unsafe extern "C" fn rot_bitmap_layer_create(bitmap: *mut GBitmap) -> *mut RotBitmapLayer {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =840", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Destroys a RotBitmapLayer and frees all associated memory.
! @note It is the developer responsibility to free the \ref GBitmap.
! @param bitmap The RotBitmapLayer to destroy.*/
/** rot_bitmap_layer_destroy
  -> Added in 0
gneiss: Vtable Slot 211, Offset 844*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.rot_bitmap_layer_destroy")]
pub unsafe extern "C" fn rot_bitmap_layer_destroy(bitmap: *mut RotBitmapLayer) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =844", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Defines what color to use in areas that are not covered by the source bitmap.
! By default this is \ref GColorClear.
! @param bitmap The RotBitmapLayer on which to change the corner clip color
! @param color The corner clip color*/
/** rot_bitmap_layer_set_corner_clip_color
  -> Added in 30
gneiss: Vtable Slot 374, Offset 1496*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.rot_bitmap_layer_set_corner_clip_color")]
pub unsafe extern "C" fn rot_bitmap_layer_set_corner_clip_color(
	bitmap: *mut RotBitmapLayer,
	color: GColor,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1496", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the rotation angle of this RotBitmapLayer
! @param bitmap The RotBitmapLayer on which to change the rotation
! @param angle Rotation is an integer between 0 (no rotation) and 0x10000 (360 degree rotation). @see sin_lookup()*/
/** rot_bitmap_layer_set_angle
  -> Added in 0
gneiss: Vtable Slot 213, Offset 852*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.rot_bitmap_layer_set_angle")]
pub unsafe extern "C" fn rot_bitmap_layer_set_angle(bitmap: *mut RotBitmapLayer, angle: i32) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =852", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Change the rotation angle of this RotBitmapLayer
! @param bitmap The RotBitmapLayer on which to change the rotation
! @param angle_change The rotation angle change*/
/** rot_bitmap_layer_increment_angle
  -> Added in 0
gneiss: Vtable Slot 212, Offset 848*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.rot_bitmap_layer_increment_angle")]
pub unsafe extern "C" fn rot_bitmap_layer_increment_angle(
	bitmap: *mut RotBitmapLayer,
	angle_change: i32,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =848", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Defines the only point that will not be affected by the rotation in the source bitmap.
!
! For example, if you pass GPoint(0, 0), the image will rotate around the top-left corner.
!
! This point is always projected at the center of the RotBitmapLayer. Calling this function
! automatically adjusts the width and height of the RotBitmapLayer so that
! the entire bitmap can fit inside the layer at all rotation angles.
!
! @param bitmap The RotBitmapLayer on which to change the rotation
! @param ic The only point in the original image that will not be affected by the rotation.*/
/** rot_bitmap_set_src_ic
  -> Added in 0
gneiss: Vtable Slot 216, Offset 864*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.rot_bitmap_set_src_ic")]
pub unsafe extern "C" fn rot_bitmap_set_src_ic(bitmap: *mut RotBitmapLayer, ic: GPoint) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =864", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the compositing mode of how the bitmap image is composited onto what has been drawn beneath the
! RotBitmapLayer.
! By default this is \ref GCompOpAssign, i.e. transparency disabled.
! The RotBitmapLayer is automatically marked dirty after this operation.
! @param bitmap The RotBitmapLayer on which to change the rotation
! @param mode The compositing mode to set
! @see \ref GCompOp for visual examples of the different compositing modes.*/
/** rot_bitmap_set_compositing_mode
  -> Added in 0
gneiss: Vtable Slot 215, Offset 860*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.rot_bitmap_set_compositing_mode")]
pub unsafe extern "C" fn rot_bitmap_set_compositing_mode(
	bitmap: *mut RotBitmapLayer,
	mode: GCompOp,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =860", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @addtogroup NumberWindow
! \brief A ready-made Window prompting the user to pick a number
!
! ![](number_window.png)
! @{*/
#[repr(C)]
#[derive(Debug)]
pub struct NumberWindow {
	_unused: [u8; 0],
}
///! Function signature for NumberWindow callbacks.
pub type NumberWindowCallback = ::core::option::Option<
	unsafe extern "C" fn(number_window: *mut NumberWindow, context: *mut ::core::ffi::c_void),
>;
///! Data structure containing all the callbacks for a NumberWindow.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct NumberWindowCallbacks {
	/**! Callback that gets called as the value is incremented.
	! Optional, leave `NULL` if unused.*/
	pub incremented: NumberWindowCallback,
	/**! Callback that gets called as the value is decremented.
	! Optional, leave `NULL` if unused.*/
	pub decremented: NumberWindowCallback,
	/**! Callback that gets called as the value is confirmed, in other words the
	! SELECT button is clicked.
	! Optional, leave `NULL` if unused.*/
	pub selected:    NumberWindowCallback,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of NumberWindowCallbacks"][::core::mem::size_of::<NumberWindowCallbacks>() - 12usize];
	["Alignment of NumberWindowCallbacks"]
		[::core::mem::align_of::<NumberWindowCallbacks>() - 4usize];
	["Offset of field: NumberWindowCallbacks::incremented"]
		[::core::mem::offset_of!(NumberWindowCallbacks, incremented) - 0usize];
	["Offset of field: NumberWindowCallbacks::decremented"]
		[::core::mem::offset_of!(NumberWindowCallbacks, decremented) - 4usize];
	["Offset of field: NumberWindowCallbacks::selected"]
		[::core::mem::offset_of!(NumberWindowCallbacks, selected) - 8usize];
};
/**! Creates a new NumberWindow on the heap and initalizes it with the default values.
!
! @param label The title or prompt to display in the NumberWindow. Must be long-lived and cannot be stack-allocated.
! @param callbacks The callbacks
! @param callback_context Pointer to application specific data that is passed
! @note The number window is not pushed to the window stack. Use \ref window_stack_push() to do this.
! @return A pointer to the NumberWindow. `NULL` if the NumberWindow could not
! be created*/
/** number_window_create
  -> Added in 0
gneiss: Vtable Slot 179, Offset 716*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.number_window_create")]
pub unsafe extern "C" fn number_window_create(
	label: *const ::core::ffi::c_char,
	callbacks: NumberWindowCallbacks,
	callback_context: *mut ::core::ffi::c_void,
) -> *mut NumberWindow {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =716", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Destroys a NumberWindow previously created by number_window_create.
/** number_window_destroy
  -> Added in 0
gneiss: Vtable Slot 180, Offset 720*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.number_window_destroy")]
pub unsafe extern "C" fn number_window_destroy(number_window: *mut NumberWindow) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =720", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the text of the title or prompt label.
! @param numberwindow Pointer to the NumberWindow for which to set the label
! text
! @param label The new label text. Must be long-lived and cannot be
! stack-allocated.*/
/** number_window_set_label
  -> Added in 0
gneiss: Vtable Slot 182, Offset 728*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.number_window_set_label")]
pub unsafe extern "C" fn number_window_set_label(
	numberwindow: *mut NumberWindow,
	label: *const ::core::ffi::c_char,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =728", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the maximum value this field can hold
! @param numberwindow Pointer to the NumberWindow for which to set the maximum
! value
! @param max The maximum value*/
/** number_window_set_max
  -> Added in 0
gneiss: Vtable Slot 183, Offset 732*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.number_window_set_max")]
pub unsafe extern "C" fn number_window_set_max(numberwindow: *mut NumberWindow, max: i32) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =732", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the minimum value this field can hold
! @param numberwindow Pointer to the NumberWindow for which to set the minimum
! value
! @param min The minimum value*/
/** number_window_set_min
  -> Added in 0
gneiss: Vtable Slot 184, Offset 736*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.number_window_set_min")]
pub unsafe extern "C" fn number_window_set_min(numberwindow: *mut NumberWindow, min: i32) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =736", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the current value of the field
! @param numberwindow Pointer to the NumberWindow for which to set the current
! value
! @param value The new current value*/
/** number_window_set_value
  -> Added in 0
gneiss: Vtable Slot 186, Offset 744*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.number_window_set_value")]
pub unsafe extern "C" fn number_window_set_value(numberwindow: *mut NumberWindow, value: i32) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =744", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Sets the amount by which to increment/decrement by on a button click
! @param numberwindow Pointer to the NumberWindow for which to set the step
! increment
! @param step The new step increment*/
/** number_window_set_step_size
  -> Added in 0
gneiss: Vtable Slot 185, Offset 740*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.number_window_set_step_size")]
pub unsafe extern "C" fn number_window_set_step_size(numberwindow: *mut NumberWindow, step: i32) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =740", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the current value
! @param numberwindow Pointer to the NumberWindow for which to get the current
! value
! @return The current value*/
/** number_window_get_value
  -> Added in 0
gneiss: Vtable Slot 181, Offset 724*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.number_window_get_value")]
pub unsafe extern "C" fn number_window_get_value(numberwindow: *const NumberWindow) -> i32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =724", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Gets the "root" Window of the number window
! @param numberwindow Pointer to the NumberWindow for which to get the "root" Window
! @return The "root" Window of the number window.*/
/** number_window_get_window
  -> Added in 12
gneiss: Vtable Slot 322, Offset 1288*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.number_window_get_window")]
pub unsafe extern "C" fn number_window_get_window(numberwindow: *mut NumberWindow) -> *mut Window {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1288", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! @addtogroup ActionMenu
! @{*/
#[repr(C)]
#[derive(Debug)]
pub struct ActionMenuItem {
	_unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct ActionMenuLevel {
	_unused: [u8; 0],
}
pub const ActionMenuAlign_ActionMenuAlignTop: ActionMenuAlign = 0;
pub const ActionMenuAlign_ActionMenuAlignCenter: ActionMenuAlign = 1;
pub type ActionMenuAlign = ::core::ffi::c_uchar;
#[repr(C)]
#[derive(Debug)]
pub struct ActionMenu {
	_unused: [u8; 0],
}
/**! Callback executed after the ActionMenu has closed, so memory may be freed.
! @param menu the ActionMenu
! @param performed_action the ActionMenuItem for the action that was performed,
! NULL if the ActionMenu is closing without an action being selected by the user
! @param context the context passed to the ActionMenu*/
pub type ActionMenuDidCloseCb = ::core::option::Option<
	unsafe extern "C" fn(
		menu: *mut ActionMenu,
		performed_action: *const ActionMenuItem,
		context: *mut ::core::ffi::c_void,
	),
>;
///!< Each item gets its own row
pub const ActionMenuLevelDisplayMode_ActionMenuLevelDisplayModeWide: ActionMenuLevelDisplayMode = 0;
///!< Grid view: multiple items per row
pub const ActionMenuLevelDisplayMode_ActionMenuLevelDisplayModeThin: ActionMenuLevelDisplayMode = 1;
/**! enum value that controls whether menu items are displayed in a grid
! (similarly to the emoji replies) or in a single column (reminiscent of \ref MenuLayer)*/
pub type ActionMenuLevelDisplayMode = ::core::ffi::c_uchar;
/**! Callback executed when a given action is selected
! @param action_menu the action menu currently on screen
! @param action the action that was triggered
! @param context the context passed to the action menu
! @note the action menu is closed immediately after an action is performed,
! unless it is frozen in the ActionMenuPerformActionCb*/
pub type ActionMenuPerformActionCb = ::core::option::Option<
	unsafe extern "C" fn(
		action_menu: *mut ActionMenu,
		action: *const ActionMenuItem,
		context: *mut ::core::ffi::c_void,
	),
>;
/**! Callback invoked for each item in an action menu hierarchy.
! @param item the current action menu item
! @param context a caller-provided callback context*/
pub type ActionMenuEachItemCb = ::core::option::Option<
	unsafe extern "C" fn(item: *const ActionMenuItem, context: *mut ::core::ffi::c_void),
>;
///! Configuration struct for the ActionMenu
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ActionMenuConfig {
	///!< the root level of the ActionMenu
	pub root_level: *const ActionMenuLevel,
	///!< a context pointer which will be accessbile when actions are performed
	pub context:    *mut ::core::ffi::c_void,
	pub colors:     ActionMenuConfig__bindgen_ty_1,
	///!< Called immediately before the ActionMenu closes
	pub will_close: ActionMenuDidCloseCb,
	///!< a callback used to cleanup memory after the menu has closed
	pub did_close:  ActionMenuDidCloseCb,
	pub align:      ActionMenuAlign,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ActionMenuConfig__bindgen_ty_1 {
	///!< the color of the left column of the ActionMenu
	pub background: GColor,
	///!< the color of the individual "crumbs" that indicate menu depth
	pub foreground: GColor,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of ActionMenuConfig__bindgen_ty_1"]
		[::core::mem::size_of::<ActionMenuConfig__bindgen_ty_1>() - 2usize];
	["Alignment of ActionMenuConfig__bindgen_ty_1"]
		[::core::mem::align_of::<ActionMenuConfig__bindgen_ty_1>() - 1usize];
	["Offset of field: ActionMenuConfig__bindgen_ty_1::background"]
		[::core::mem::offset_of!(ActionMenuConfig__bindgen_ty_1, background) - 0usize];
	["Offset of field: ActionMenuConfig__bindgen_ty_1::foreground"]
		[::core::mem::offset_of!(ActionMenuConfig__bindgen_ty_1, foreground) - 1usize];
};
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of ActionMenuConfig"][::core::mem::size_of::<ActionMenuConfig>() - 24usize];
	["Alignment of ActionMenuConfig"][::core::mem::align_of::<ActionMenuConfig>() - 4usize];
	["Offset of field: ActionMenuConfig::root_level"]
		[::core::mem::offset_of!(ActionMenuConfig, root_level) - 0usize];
	["Offset of field: ActionMenuConfig::context"]
		[::core::mem::offset_of!(ActionMenuConfig, context) - 4usize];
	["Offset of field: ActionMenuConfig::colors"]
		[::core::mem::offset_of!(ActionMenuConfig, colors) - 8usize];
	["Offset of field: ActionMenuConfig::will_close"]
		[::core::mem::offset_of!(ActionMenuConfig, will_close) - 12usize];
	["Offset of field: ActionMenuConfig::did_close"]
		[::core::mem::offset_of!(ActionMenuConfig, did_close) - 16usize];
	["Offset of field: ActionMenuConfig::align"]
		[::core::mem::offset_of!(ActionMenuConfig, align) - 20usize];
};
/**! Getter for the label of a given \ref ActionMenuItem
! @param item the \ref ActionMenuItem of interest
! @return a pointer to the string label. NULL if invalid.*/
/** action_menu_item_get_label
  -> Added in 63
gneiss: Vtable Slot 542, Offset 2168*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_menu_item_get_label")]
pub unsafe extern "C" fn action_menu_item_get_label(
	item: *const ActionMenuItem,
) -> *mut ::core::ffi::c_char {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2168", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Getter for the action_data pointer of a given \ref ActionMenuItem.
! @see action_menu_level_add_action
! @param item the \ref ActionMenuItem of interest
! @return a pointer to the data. NULL if invalid.*/
/** action_menu_item_get_action_data
  -> Added in 63
gneiss: Vtable Slot 541, Offset 2164*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_menu_item_get_action_data")]
pub unsafe extern "C" fn action_menu_item_get_action_data(
	item: *const ActionMenuItem,
) -> *mut ::core::ffi::c_void {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2164", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Create a new action menu level with storage allocated for a given number of items
! @param max_items the max number of items that will be displayed at that level
! @note levels are freed alongside the whole hierarchy so no destroy API is provided.
! @note by default, levels are using ActionMenuLevelDisplayModeWide.
! Use \ref action_menu_level_set_display_mode to change it.
! @see action_menu_hierarchy_destroy*/
/** action_menu_level_create
  -> Added in 63
gneiss: Vtable Slot 545, Offset 2180*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_menu_level_create")]
pub unsafe extern "C" fn action_menu_level_create(max_items: u16) -> *mut ActionMenuLevel {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2180", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set the action menu display mode
! @param level The ActionMenuLevel whose display mode you want to change
! @param display_mode The display mode for the action menu (3 vs. 1 item per row)*/
/** action_menu_level_set_display_mode
  -> Added in 63
gneiss: Vtable Slot 546, Offset 2184*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_menu_level_set_display_mode")]
pub unsafe extern "C" fn action_menu_level_set_display_mode(
	level: *mut ActionMenuLevel,
	display_mode: ActionMenuLevelDisplayMode,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2184", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Add an action to an ActionLevel
! @param level the level to add the action to
! @param label the text to display for the action in the menu
! @param cb the callback that will be triggered when this action is actuated
! @param action_data data to pass to the callback for this action
! @return a reference to the new \ref ActionMenuItem on success, NULL if the level is full*/
/** action_menu_level_add_action
  -> Added in 63
gneiss: Vtable Slot 543, Offset 2172*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_menu_level_add_action")]
pub unsafe extern "C" fn action_menu_level_add_action(
	level: *mut ActionMenuLevel,
	label: *const ::core::ffi::c_char,
	cb: ActionMenuPerformActionCb,
	action_data: *mut ::core::ffi::c_void,
) -> *mut ActionMenuItem {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2172", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Add a child to this ActionMenuLevel
! @param level the parent level
! @param child the child level
! @param label the text to display in the action menu for this level
! @return a reference to the new \ref ActionMenuItem on success, NULL if the level is full*/
/** action_menu_level_add_child
  -> Added in 63
gneiss: Vtable Slot 544, Offset 2176*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_menu_level_add_child")]
pub unsafe extern "C" fn action_menu_level_add_child(
	level: *mut ActionMenuLevel,
	child: *mut ActionMenuLevel,
	label: *const ::core::ffi::c_char,
) -> *mut ActionMenuItem {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2176", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Destroy a hierarchy of ActionMenuLevels
! @param root the root level in the hierarchy
! @param each_cb a callback to call on every \ref ActionMenuItem in every level
! @param context a context pointer to pass to each_cb on invocation
! @note Typical implementations will cleanup memory allocated for the item label/data
!       associated with each item in the callback
! @note Hierarchy is traversed in post-order.
!       In other words, all children items are freed before their parent is freed.*/
/** action_menu_hierarchy_destroy
  -> Added in 63
gneiss: Vtable Slot 540, Offset 2160*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_menu_hierarchy_destroy")]
pub unsafe extern "C" fn action_menu_hierarchy_destroy(
	root: *const ActionMenuLevel,
	each_cb: ActionMenuEachItemCb,
	context: *mut ::core::ffi::c_void,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2160", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the context pointer this ActionMenu was created with
! @param action_menu A pointer to an ActionMenu
! @return the context pointer initially provided in the \ref ActionMenuConfig.
! NULL if none exists.*/
/** action_menu_get_context
  -> Added in 63
gneiss: Vtable Slot 538, Offset 2152*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_menu_get_context")]
pub unsafe extern "C" fn action_menu_get_context(
	action_menu: *mut ActionMenu,
) -> *mut ::core::ffi::c_void {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2152", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the root level of an ActionMenu
! @param action_menu the ActionMenu you want to know about
! @return a pointer to the root ActionMenuLevel for the given ActionMenu, NULL if invalid*/
/** action_menu_get_root_level
  -> Added in 63
gneiss: Vtable Slot 539, Offset 2156*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_menu_get_root_level")]
pub unsafe extern "C" fn action_menu_get_root_level(
	action_menu: *mut ActionMenu,
) -> *mut ActionMenuLevel {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2156", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Open a new ActionMenu.
! The ActionMenu acts much like a window. It fills the whole screen and handles clicks.
! @param config the configuration info for this new ActionMenu
! @return the new ActionMenu*/
/** action_menu_open
  -> Added in 63
gneiss: Vtable Slot 547, Offset 2188*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_menu_open")]
pub unsafe extern "C" fn action_menu_open(config: *mut ActionMenuConfig) -> *mut ActionMenu {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2188", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Freeze the ActionMenu. The ActionMenu will no longer respond to user input.
! @note this API should be used when waiting for asynchronous operation.
! @param action_menu the ActionMenu*/
/** action_menu_freeze
  -> Added in 63
gneiss: Vtable Slot 537, Offset 2148*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_menu_freeze")]
pub unsafe extern "C" fn action_menu_freeze(action_menu: *mut ActionMenu) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2148", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Unfreeze the ActionMenu previously frozen with \ref action_menu_freeze
! @param action_menu the ActionMenu to unfreeze*/
/** action_menu_unfreeze
  -> Added in 63
gneiss: Vtable Slot 549, Offset 2196*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_menu_unfreeze")]
pub unsafe extern "C" fn action_menu_unfreeze(action_menu: *mut ActionMenu) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2196", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Set the result window for an ActionMenu. The result window will be
! shown when the ActionMenu closes
! @param action_menu the ActionMenu
! @param result_window the window to insert, pass NULL to remove the current result window
! @note repeated call will result in only the last call to be applied, i.e. only
! one result window is ever set*/
/** action_menu_set_result_window
  -> Added in 63
gneiss: Vtable Slot 548, Offset 2192*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_menu_set_result_window")]
pub unsafe extern "C" fn action_menu_set_result_window(
	action_menu: *mut ActionMenu,
	result_window: *mut Window,
) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2192", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Close the ActionMenu, whether it is frozen or not.
! @note this API can be used on a frozen ActionMenu once the data required to
! build the result window has been received and the result window has been set
! @param action_menu the ActionMenu to close
! @param animated whether or not show a close animation*/
/** action_menu_close
  -> Added in 63
gneiss: Vtable Slot 536, Offset 2144*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.action_menu_close")]
pub unsafe extern "C" fn action_menu_close(action_menu: *mut ActionMenu, animated: bool) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2144", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/** Data structure describing a vibration pattern.
A pattern consists of at least 1 vibe-on duration, optionally followed by
alternating vibe-off + vibe-on durations. Each segment may have a different duration.

Example code:
\code{.c}
// Vibe pattern: ON for 200ms, OFF for 100ms, ON for 400ms:
static const uint32_t segments[] = { 200, 100, 400 };
VibePattern pat = {
.durations = segments,
.num_segments = ARRAY_LENGTH(segments),
};
vibes_enqueue_custom_pattern(pat);
\endcode
@see vibes_enqueue_custom_pattern*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VibePattern {
	/**Pointer to an array of segment durations, measured in milli-seconds.
	The maximum allowed duration is 10000ms.*/
	pub durations:    *const u32,
	///The length of the array of durations.
	pub num_segments: u32,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of VibePattern"][::core::mem::size_of::<VibePattern>() - 8usize];
	["Alignment of VibePattern"][::core::mem::align_of::<VibePattern>() - 4usize];
	["Offset of field: VibePattern::durations"]
		[::core::mem::offset_of!(VibePattern, durations) - 0usize];
	["Offset of field: VibePattern::num_segments"]
		[::core::mem::offset_of!(VibePattern, num_segments) - 4usize];
};
/**! Cancel any in-flight vibe patterns; this is a no-op if there is no
! on-going vibe.*/
/** vibes_cancel
  -> Added in 0
gneiss: Vtable Slot 266, Offset 1064*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.vibes_cancel")]
pub unsafe extern "C" fn vibes_cancel() {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1064", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Makes the watch emit one short vibration.
/** vibes_short_pulse
  -> Added in 0
gneiss: Vtable Slot 270, Offset 1080*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.vibes_short_pulse")]
pub unsafe extern "C" fn vibes_short_pulse() {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1080", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
///! Makes the watch emit one long vibration.
/** vibes_long_pulse
  -> Added in 0
gneiss: Vtable Slot 269, Offset 1076*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.vibes_long_pulse")]
pub unsafe extern "C" fn vibes_long_pulse() {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1076", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Makes the watch emit two brief vibrations.
!*/
/** vibes_double_pulse
  -> Added in 0
gneiss: Vtable Slot 267, Offset 1068*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.vibes_double_pulse")]
pub unsafe extern "C" fn vibes_double_pulse() {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1068", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Makes the watch emit a 'custom' vibration pattern.
! @param pattern An arbitrary vibration pattern
! @see VibePattern*/
/** vibes_enqueue_custom_pattern
  -> Added in 0
gneiss: Vtable Slot 268, Offset 1072*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.vibes_enqueue_custom_pattern")]
pub unsafe extern "C" fn vibes_enqueue_custom_pattern(pattern: VibePattern) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1072", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
pub const SpeakerWaveform_SpeakerWaveformSine: SpeakerWaveform = 0;
pub const SpeakerWaveform_SpeakerWaveformSquare: SpeakerWaveform = 1;
pub const SpeakerWaveform_SpeakerWaveformTriangle: SpeakerWaveform = 2;
pub const SpeakerWaveform_SpeakerWaveformSawtooth: SpeakerWaveform = 3;
pub const SpeakerWaveform_SpeakerWaveformCount: SpeakerWaveform = 4;
/**! @addtogroup Speaker
! \brief Controlling the speaker
!
! The Speaker API provides calls that let you play sounds through the watch's speaker.
! You can play simple note sequences (melodies), single tones, or stream raw PCM audio.
!
! Note sequences are compact representations of melodies using MIDI-like note definitions,
! supporting 4 basic waveforms: sine, square, triangle, and sawtooth.
!
! Raw PCM streaming allows apps to generate arbitrary audio in configurable formats.
! @{*/
pub type SpeakerWaveform = ::core::ffi::c_uchar;
/**! A single note in a sequence.
! midi_note: MIDI note number (0-127, 60=C4). 0 = rest (silence).
! waveform: SpeakerWaveform value.
! duration_ms: Note duration in ms (max 10000).
! velocity: Volume 0-127 (0 = use global volume).*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PACKED {
	pub midi_note:   u8,
	pub waveform:    u8,
	pub duration_ms: u16,
	pub velocity:    u8,
	pub reserved:    u8,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of PACKED"][::core::mem::size_of::<PACKED>() - 6usize];
	["Alignment of PACKED"][::core::mem::align_of::<PACKED>() - 2usize];
	["Offset of field: PACKED::midi_note"][::core::mem::offset_of!(PACKED, midi_note) - 0usize];
	["Offset of field: PACKED::waveform"][::core::mem::offset_of!(PACKED, waveform) - 1usize];
	["Offset of field: PACKED::duration_ms"][::core::mem::offset_of!(PACKED, duration_ms) - 2usize];
	["Offset of field: PACKED::velocity"][::core::mem::offset_of!(PACKED, velocity) - 4usize];
	["Offset of field: PACKED::reserved"][::core::mem::offset_of!(PACKED, reserved) - 5usize];
};
/**! A single note in a sequence.
! midi_note: MIDI note number (0-127, 60=C4). 0 = rest (silence).
! waveform: SpeakerWaveform value.
! duration_ms: Note duration in ms (max 10000).
! velocity: Volume 0-127 (0 = use global volume).*/
pub type SpeakerNote = PACKED;
///!< 8kHz 8-bit signed (1 byte/sample)
pub const SpeakerPcmFormat_SpeakerPcmFormat_8kHz_8bit: SpeakerPcmFormat = 0;
///!< 16kHz 8-bit signed (1 byte/sample)
pub const SpeakerPcmFormat_SpeakerPcmFormat_16kHz_8bit: SpeakerPcmFormat = 1;
///!< 8kHz 16-bit signed little-endian (2 bytes/sample)
pub const SpeakerPcmFormat_SpeakerPcmFormat_8kHz_16bit: SpeakerPcmFormat = 2;
///!< 16kHz 16-bit signed little-endian (2 bytes/sample)
pub const SpeakerPcmFormat_SpeakerPcmFormat_16kHz_16bit: SpeakerPcmFormat = 3;
pub const SpeakerPcmFormat_SpeakerPcmFormatCount: SpeakerPcmFormat = 4;
/**! PCM audio format for speaker streaming.
! Bit layout: bit0 = sample rate (0=8kHz, 1=16kHz), bit1 = bit depth (0=8-bit, 1=16-bit).
! All formats are mono signed PCM (8-bit samples are signed [-128,127], not unsigned).*/
pub type SpeakerPcmFormat = ::core::ffi::c_uchar;
pub const SpeakerStatus_SpeakerStatusIdle: SpeakerStatus = 0;
pub const SpeakerStatus_SpeakerStatusPlaying: SpeakerStatus = 1;
pub const SpeakerStatus_SpeakerStatusDraining: SpeakerStatus = 2;
///! Speaker status
pub type SpeakerStatus = ::core::ffi::c_uchar;
///!< Playback completed naturally
pub const SpeakerFinishReason_SpeakerFinishReasonDone: SpeakerFinishReason = 0;
///!< Playback was stopped by the app
pub const SpeakerFinishReason_SpeakerFinishReasonStopped: SpeakerFinishReason = 1;
///!< Preempted by higher priority source
pub const SpeakerFinishReason_SpeakerFinishReasonPreempted: SpeakerFinishReason = 2;
///!< An error occurred
pub const SpeakerFinishReason_SpeakerFinishReasonError: SpeakerFinishReason = 3;
///! Reason reported when speaker playback ends.
pub type SpeakerFinishReason = ::core::ffi::c_uchar;
/**! Callback invoked when playback finishes.
! @param reason Why playback ended
! @param ctx User context*/
pub type SpeakerFinishedCallback = ::core::option::Option<
	unsafe extern "C" fn(reason: SpeakerFinishReason, ctx: *mut ::core::ffi::c_void),
>;
/**! A raw PCM sample that can be pitch-shifted when played by a track.
! data: mono signed PCM in the given format.
! num_bytes: size of data in bytes.
! format: sample rate + bit depth (see SpeakerPcmFormat).
! base_midi_note: the MIDI note at which the sample plays unshifted (e.g. 60 = C4).
!                 Notes above/below this value are produced by resampling.
! loop: if true, the sample restarts from the beginning each time it runs out,
!       and keeps playing until the owning note's duration elapses.*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SpeakerSample {
	pub data:           *const ::core::ffi::c_void,
	pub num_bytes:      u32,
	pub format:         SpeakerPcmFormat,
	pub base_midi_note: u8,
	pub loop_:          bool,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of SpeakerSample"][::core::mem::size_of::<SpeakerSample>() - 12usize];
	["Alignment of SpeakerSample"][::core::mem::align_of::<SpeakerSample>() - 4usize];
	["Offset of field: SpeakerSample::data"][::core::mem::offset_of!(SpeakerSample, data) - 0usize];
	["Offset of field: SpeakerSample::num_bytes"]
		[::core::mem::offset_of!(SpeakerSample, num_bytes) - 4usize];
	["Offset of field: SpeakerSample::format"]
		[::core::mem::offset_of!(SpeakerSample, format) - 8usize];
	["Offset of field: SpeakerSample::base_midi_note"]
		[::core::mem::offset_of!(SpeakerSample, base_midi_note) - 9usize];
	["Offset of field: SpeakerSample::loop_"]
		[::core::mem::offset_of!(SpeakerSample, loop_) - 10usize];
};
/**! A single monophonic voice. Multiple tracks are mixed together by
! speaker_play_tracks() to produce polyphony.
! notes: array of notes to play sequentially.
! num_notes: length of the notes array.
! sample: if non-NULL, notes are played by pitch-shifting this sample;
!         note.waveform is ignored. If NULL, notes use their waveform field.*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SpeakerTrack {
	pub notes:     *const SpeakerNote,
	pub num_notes: u32,
	pub sample:    *const SpeakerSample,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of SpeakerTrack"][::core::mem::size_of::<SpeakerTrack>() - 12usize];
	["Alignment of SpeakerTrack"][::core::mem::align_of::<SpeakerTrack>() - 4usize];
	["Offset of field: SpeakerTrack::notes"][::core::mem::offset_of!(SpeakerTrack, notes) - 0usize];
	["Offset of field: SpeakerTrack::num_notes"]
		[::core::mem::offset_of!(SpeakerTrack, num_notes) - 4usize];
	["Offset of field: SpeakerTrack::sample"]
		[::core::mem::offset_of!(SpeakerTrack, sample) - 8usize];
};
/**! Trigger the backlight and schedule a timer to automatically disable the backlight
! after a short delay. This is the preferred method of interacting with the backlight.*/
/** light_enable_interaction
  -> Added in 0
gneiss: Vtable Slot 159, Offset 636*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.light_enable_interaction")]
pub unsafe extern "C" fn light_enable_interaction() {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =636", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Turn the watch's backlight on or put it back into automatic control.
! Developers should take care when calling this function, keeping Pebble's backlight on for long periods of time
! will rapidly deplete the battery.
! @param enable Turn the backlight on if `true`, otherwise `false` to put it back into automatic control.*/
/** light_enable
  -> Added in 0
gneiss: Vtable Slot 158, Offset 632*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.light_enable")]
pub unsafe extern "C" fn light_enable(enable: bool) {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =632", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Get the recommended amount of milliseconds a result window should be visible before it should
! automatically close.
! @note It is the application developer's responsibility to automatically close a result window.
! @return The recommended result window timeout duration in milliseconds*/
/** preferred_result_display_duration
  -> Added in 83
gneiss: Vtable Slot 623, Offset 2492*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.preferred_result_display_duration")]
pub unsafe extern "C" fn preferred_result_display_duration() -> u32 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2492", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
pub const PreferredContentSize_PreferredContentSizeSmall: PreferredContentSize = 0;
pub const PreferredContentSize_PreferredContentSizeMedium: PreferredContentSize = 1;
pub const PreferredContentSize_PreferredContentSizeLarge: PreferredContentSize = 2;
pub const PreferredContentSize_PreferredContentSizeExtraLarge: PreferredContentSize = 3;
pub const PreferredContentSize_NumPreferredContentSizes: PreferredContentSize = 4;
/**! PreferredContentSize represents the display scale of all the app's UI components. The enum
! contains all sizes that all platforms as a whole are capable of displaying, but each individual
! platform may not be able to display all sizes.
! @note As of version 4.1, platforms other than Emery cannot display extra large and Emery itself
! cannot display small.*/
pub type PreferredContentSize = ::core::ffi::c_uchar;
/**! Returns the user's preferred content size representing the scale of all the app's UI components
! should use for display.
! @return The user's \ref PreferredContentSize setting.*/
/** preferred_content_size
  -> Added in 88
gneiss: Vtable Slot 630, Offset 2520*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.preferred_content_size")]
pub unsafe extern "C" fn preferred_content_size() -> PreferredContentSize {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2520", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Users can toggle Quiet Time manually or on schedule. Watchfaces and apps should respect this
! choice and avoid disturbing actions such as vibration if quiet time is active.
! @return True, if Quiet Time is currently active.*/
/** quiet_time_is_active
  -> Added in 89
gneiss: Vtable Slot 631, Offset 2524*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.quiet_time_is_active")]
pub unsafe extern "C" fn quiet_time_is_active() -> bool {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2524", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! structure containing broken-down time for expressing calendar time
! (ie. Year, Month, Day of Month, Hour of Day) and timezone information*/
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct tm {
	///< Seconds. [0-60] (1 leap second)
	pub tm_sec:    ::core::ffi::c_int,
	///< Minutes. [0-59]
	pub tm_min:    ::core::ffi::c_int,
	///< Hours.  [0-23]
	pub tm_hour:   ::core::ffi::c_int,
	///< Day. [1-31]
	pub tm_mday:   ::core::ffi::c_int,
	///< Month. [0-11]
	pub tm_mon:    ::core::ffi::c_int,
	///< Years since 1900
	pub tm_year:   ::core::ffi::c_int,
	///< Day of week. [0-6]
	pub tm_wday:   ::core::ffi::c_int,
	///< Days in year.[0-365]
	pub tm_yday:   ::core::ffi::c_int,
	///< DST. [-1/0/1]
	pub tm_isdst:  ::core::ffi::c_int,
	/**< Total seconds east of UTC, DST included. tm_isdst is an indicator only --
	never add it to this value.*/
	pub tm_gmtoff: ::core::ffi::c_int,
	///< Timezone abbreviation
	pub tm_zone:   [::core::ffi::c_char; 6usize],
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
	["Size of tm"][::core::mem::size_of::<tm>() - 48usize];
	["Alignment of tm"][::core::mem::align_of::<tm>() - 4usize];
	["Offset of field: tm::tm_sec"][::core::mem::offset_of!(tm, tm_sec) - 0usize];
	["Offset of field: tm::tm_min"][::core::mem::offset_of!(tm, tm_min) - 4usize];
	["Offset of field: tm::tm_hour"][::core::mem::offset_of!(tm, tm_hour) - 8usize];
	["Offset of field: tm::tm_mday"][::core::mem::offset_of!(tm, tm_mday) - 12usize];
	["Offset of field: tm::tm_mon"][::core::mem::offset_of!(tm, tm_mon) - 16usize];
	["Offset of field: tm::tm_year"][::core::mem::offset_of!(tm, tm_year) - 20usize];
	["Offset of field: tm::tm_wday"][::core::mem::offset_of!(tm, tm_wday) - 24usize];
	["Offset of field: tm::tm_yday"][::core::mem::offset_of!(tm, tm_yday) - 28usize];
	["Offset of field: tm::tm_isdst"][::core::mem::offset_of!(tm, tm_isdst) - 32usize];
	["Offset of field: tm::tm_gmtoff"][::core::mem::offset_of!(tm, tm_gmtoff) - 36usize];
	["Offset of field: tm::tm_zone"][::core::mem::offset_of!(tm, tm_zone) - 40usize];
};
/**! Format the time value at tm according to fmt and place the result in a buffer s of size max
! @param s A preallocation char array of size max
! @param maxsize the size of the array s
! @param format a formatting string
! @param tm_p A pointer to a struct tm containing a broken out time value
! @return The number of bytes placed in the array s, not including the null byte,
!   0 if the value does not fit.*/
/** strftime
  -> Added in 0
gneiss: Vtable Slot 244, Offset 976*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.strftime")]
pub unsafe extern "C" fn strftime(
	s: *mut ::core::ffi::c_char,
	maxsize: usize,
	format: *const ::core::ffi::c_char,
	tm_p: *const tm,
) -> usize {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =976", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! convert the time value pointed at by clock to a struct tm which contains the time
! adjusted for the local timezone
! @param timep A pointer to an object of type time_t that contains a time value
! @return A pointer to a struct tm containing the broken out time value adjusted
!   for the local timezone*/
/** localtime
  -> Added in 31
gneiss: Vtable Slot 379, Offset 1516*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.localtime")]
pub unsafe extern "C" fn localtime(timep: *const time_t) -> *mut tm {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1516", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! convert the time value pointed at by clock to a struct tm
!   which contains the time expressed in Coordinated Universal Time (UTC)
! @param timep A pointer to an object of type time_t that contains a time value
! @return A pointer to a struct tm containing Coordinated Universal Time (UTC)*/
/** gmtime
  -> Added in 0
gneiss: Vtable Slot 104, Offset 416*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.gmtime")]
pub unsafe extern "C" fn gmtime(timep: *const time_t) -> *mut tm {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =416", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! convert the broken-down time structure to a timestamp
!   expressed in Coordinated Universal Time (UTC)
! @param tb A pointer to an object of type tm that contains broken-down time
! @return The number of seconds since epoch, January 1st 1970*/
/** mktime
  -> Added in 29
gneiss: Vtable Slot 363, Offset 1452*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.mktime")]
pub unsafe extern "C" fn mktime(tb: *mut tm) -> time_t {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =1452", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Obtain the number of seconds since epoch.
! Note that the epoch is not adjusted for Timezones and Daylight Savings.
! @param tloc Optionally points to an address of a time_t variable to store the time in.
!     If you only want to use the return value, you may pass NULL into tloc instead
! @return The number of seconds since epoch, January 1st 1970*/
/** time
  -> Added in 51
gneiss: Vtable Slot 519, Offset 2076*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.time")]
pub unsafe extern "C" fn time(tloc: *mut time_t) -> time_t {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2076", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Obtain the number of seconds elapsed between beginning and end represented as a double.
! @param end A time_t variable representing some number of seconds since epoch, January 1st 1970
! @param beginning A time_t variable representing some number of seconds since epoch,
!     January 1st 1970. Note that end should be greater than beginning, but this is not enforced.
! @return The number of seconds elapsed between beginning and end.
! @note Pebble uses software floating point emulation.  Including this function which returns a
!     double will significantly increase the size of your binary.  We recommend directly
!     subtracting both timestamps to calculate a time difference.
!     \code{.c}
!     int difference = ts1 - ts2;
!     \endcode*/
/** difftime
  -> Added in 58
gneiss: Vtable Slot 531, Offset 2124*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.difftime")]
pub unsafe extern "C" fn difftime(end: time_t, beginning: time_t) -> f64 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2124", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Obtain the number of seconds and milliseconds part since the epoch.
!   This is a non-standard C function provided for convenience.
! @param tloc if provided receives the current UTC Unix Time seconds portion
! @param out_ms if provided receives the current Unix Time milliseconds portion
! @return Current Unix Time milliseconds portion*/
/** time_ms
  -> Added in 59
gneiss: Vtable Slot 532, Offset 2128*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.time_ms")]
pub unsafe extern "C" fn time_ms(tloc: *mut time_t, out_ms: *mut u16) -> u16 {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2128", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
/**! Return the UTC time that corresponds to the start of today (midnight).
! @return the UTC time corresponding to the start of today (midnight)*/
/** time_start_of_today
  -> Added in 78
gneiss: Vtable Slot 608, Offset 2432*/
#[unsafe(naked)]
#[unsafe(link_section = ".text.time_start_of_today")]
pub unsafe extern "C" fn time_start_of_today() -> time_t {
	core::arch::naked_asm!(
		"push  {{r0-r3}}", "ldr   r1, =2432", "b     {resolver}", resolver = sym crate
		::shim::jump_to_pbl_function,
	)
}
