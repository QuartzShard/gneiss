//! PebbleOS `syscall` shim
//! All OS fn calls will have a generated shim which moves their vtable index into the r1 register,
//! before branching into [`jump_to_pbl_fn`].
//! Relies on the shim code being co-located in memory with the address, in that order, no more than
//! 4095 bytes apart. (if within 1020, instruction remains T1 width)
//! ```
//! .pbl_dispatch : ALIGN(4) {
//!     KEEP(*(.pbl_dispatch.code))
//!     . = ALIGN(4);
//!     KEEP(*(.pbl_dispatch.data))
//!  } > APP
//!  ```

#[used]
#[unsafe(export_name = "pbl_table_addr")]
#[unsafe(link_section = ".pbl_dispatch.data")]
/// The dummy address that the PebbleOS loader replaces with
/// a pointer to the vtable at load time
pub(crate) static mut PBL_TABLE_ADDR: u32 = 0xA8A8_A8A8;

#[unsafe(naked)]
#[unsafe(link_section = ".pbl_dispatch.code")]
/// Call a pebble-sys function. Requires the index of the fn ptr to be in r1, which is
/// handled by the per-fn shim. Don't call directly unless you really know what you're doing
pub(crate) unsafe extern "C" fn jump_to_pbl_function() {
    core::arch::naked_asm!(
        "adr   r3, {table}",        // r3  = &PBL_TABLE_ADDR
        "ldr   r0, [r3]",           // r0  = base of OS dispatch table
        "add   r0, r0, r1",         // r0  = base + offset (r1)
        "ldr   r12, [r0]",          // r12 = fn ptr
        "pop   {{r0-r3}}",          // restore AAPCS args
        "bx    r12",                // tail-call into OS, leaves return address at caller, not shim
        table = sym PBL_TABLE_ADDR,
    )
}
