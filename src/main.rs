//! main.rs
//! Author: Bradley Thompson

// Stop implicitly linking the standard library - has OS dependencies.
#![no_std]
// Tell the Rust compiler that we don't want to use the normal entrypoint
// chain with crt0.
#![no_main]

use core::panic::PanicInfo;

mod vga_buffer;
use vga_buffer::{Color, ColorCode, Buffer, Writer};

const VGA_BUFFER_ADDRESS: u32 = 0xb8000;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// - `!` specifies this as a diverging fn; entry point should invoke the `exit` syscall.
/// - Throws linker error by default b/c program depends on C runtime. Build for bare metal to fix.
#[no_mangle]
pub extern "C" fn _start() -> ! {
	let mut writer = Writer {
		column_position: 0,
		color_code: ColorCode::new(Color::Black, Color::White),
		buffer: unsafe { &mut *(VGA_BUFFER_ADDRESS as *mut Buffer) },
	};

	writer.write_str("Hello World!");

	writer.write_byte(b'T');
	writer.write_str("est! ~☺_☺~");

    loop {}
}
