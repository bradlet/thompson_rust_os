//! main.rs
//! Author: Bradley Thompson

// Stop implicitly linking the standard library - has OS dependencies.
#![no_std]
// Tell the Rust compiler that we don't want to use the normal entrypoint
// chain with crt0.
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	loop {}
}

/// - Tell the rust compiler to actually name this `_start` instead of encoding
/// additional information (name mangling) for the sake of unique fn id.
/// - Use `extern "C"` to specify the C calling convention instead of the Rust
/// calling convention.
/// - C calling convention is stack-centric: subroutine params, registers, local vars
/// all placed in memory on the stack.
/// - `!` specifies this as a diverging fn; entry point should invoke the `exit` syscall.
/// - Throws linker error by default b/c program depends on C runtime. Build for bare metal to fix.
#[no_mangle]
pub extern "C" fn _start() -> ! {
	loop {}
}
