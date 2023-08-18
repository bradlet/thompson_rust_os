//! main.rs
//! Author: Bradley Thompson

// Stop implicitly linking the standard library - has OS dependencies.
#![no_std]
// Tell the Rust compiler that we don't want to use the normal entrypoint
// chain with crt0.
#![no_main]

// Enable custom test framework because the default `test` crate requires the std lib.
#![feature(custom_test_frameworks)]
// This generates a main function that calls `test_runner`, but we configured no_main.
#![test_runner(crate::test_runner)]
// This is needed to change the name of the generated main function...
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

mod vga_buffer;

const IOBASE_PORT: u16 = 0xf4;

// Wrap codes sent to QEMU's `isa-debug-exit` device for clarity;
// Using port-mapped I/O to communicate that the kernel should quit when
// we write one of these codes to the IOBASE_PORT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(IOBASE_PORT);
        port.write(exit_code as u32);
    }
}

// `tests` - slice of trait object references pointing at the 
// [Fn()](https://doc.rust-lang.org/std/ops/trait.Fn.html) trait.
// - All functions annotated with `#[test_case]` will have their reference passed here.
#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) -> () {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	println!("Panic: {}", _info);
    loop {}
}

/// - `!` specifies this as a diverging fn; entry point should invoke the `exit` syscall.
/// - Throws linker error by default b/c program depends on C runtime. Build for bare metal to fix.
#[no_mangle]
pub extern "C" fn _start() -> ! {
	println!("Hello {}!", "World");

	// Call the generated test main in test contexts
	#[cfg(test)]
	test_main();

    loop {}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test_case]
	fn a_test() {
		print!("Some assertion...");
		assert_eq!(1, 1);
		println!("OK");
	}

}
