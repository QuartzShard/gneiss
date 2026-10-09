//! The C `mem*` functions and their AEABI forms, forwarded to the firmware's exports.
//!
//! `compiler_builtins` defines all of these as weak symbols, so these strong definitions replace
//! them in the link. `memcmp` and `bcmp` are not exported by the firmware and stay
//! `compiler_builtins`'.

use core::ffi::{c_int, c_void};

macro_rules! forward {
	($($name:ident($($arg:ident: $ty:ty),*) $(-> $ret:ty)? => [$($asm:literal),*] $target:path;)*) => {$(
		#[unsafe(naked)]
		#[unsafe(no_mangle)]
		pub unsafe extern "C" fn $name($($arg: $ty),*) $(-> $ret)? {
			core::arch::naked_asm!($($asm,)* "b     {target}", target = sym $target)
		}
	)*};
}

forward! {
	memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void => [] crate::memcpy;
	memmove(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void => [] crate::memmove;
	memset(dst: *mut c_void, c: c_int, n: usize) -> *mut c_void => [] crate::memset;

	__aeabi_memcpy(dst: *mut c_void, src: *const c_void, n: usize) => [] crate::memcpy;
	__aeabi_memcpy4(dst: *mut c_void, src: *const c_void, n: usize) => [] crate::memcpy;
	__aeabi_memcpy8(dst: *mut c_void, src: *const c_void, n: usize) => [] crate::memcpy;
	__aeabi_memmove(dst: *mut c_void, src: *const c_void, n: usize) => [] crate::memmove;
	__aeabi_memmove4(dst: *mut c_void, src: *const c_void, n: usize) => [] crate::memmove;
	__aeabi_memmove8(dst: *mut c_void, src: *const c_void, n: usize) => [] crate::memmove;

	// AEABI memset takes (dst, n, c), C memset (dst, c, n)
	__aeabi_memset(dst: *mut c_void, n: usize, c: c_int) => ["mov   r3, r1", "mov   r1, r2", "mov   r2, r3"] crate::memset;
	__aeabi_memset4(dst: *mut c_void, n: usize, c: c_int) => ["mov   r3, r1", "mov   r1, r2", "mov   r2, r3"] crate::memset;
	__aeabi_memset8(dst: *mut c_void, n: usize, c: c_int) => ["mov   r3, r1", "mov   r1, r2", "mov   r2, r3"] crate::memset;
	__aeabi_memclr(dst: *mut c_void, n: usize) => ["mov   r2, r1", "movs  r1, #0"] crate::memset;
	__aeabi_memclr4(dst: *mut c_void, n: usize) => ["mov   r2, r1", "movs  r1, #0"] crate::memset;
	__aeabi_memclr8(dst: *mut c_void, n: usize) => ["mov   r2, r1", "movs  r1, #0"] crate::memset;
}
