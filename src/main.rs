//! main.rs
//! Author: Bradley Thompson

// Stop implicitly linking the standard library - has OS dependencies.
#![no_std]
// Tell the Rust compiler that we don't want to use the normal entrypoint
// chain with crt0.
#![no_main]

use core::panic::PanicInfo;

const VGA_BUFFER_ADDRESS: u8 = 0xb8000;
const OS_WELCOME_MSG: &[u8] = b"Hello World!";

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// - `!` specifies this as a diverging fn; entry point should invoke the `exit` syscall.
/// - Throws linker error by default b/c program depends on C runtime. Build for bare metal to fix.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga_buffer = VGA_BUFFER_ADDRESS as *mut u8;

    // Enumerate so we can get the index in the welcome msg byte string so that we can use `offset` to
    // point to the region of memory in the VGA buffer that we want to write our string to.
    for (i, &byte) in OS_WELCOME_MSG.iter().enumerate() {
        // Need to wrap operations on the vga_buffer pointer in `unsafe` because rustc can't prove the raw ptr is valid.
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }

    loop {}
}
