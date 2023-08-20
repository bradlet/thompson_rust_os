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
mod serial;

// Keeping all ports used for port-mapped I/O here.
const IOBASE_PORT: u16 = 0xf4;
const SERIAL_PORT: u16 = 0x3F8; // Standard port number for UART's first serial interface

// Wrap codes sent to QEMU's `isa-debug-exit` device for clarity;
// Using port-mapped I/O to communicate that the kernel should quit when
// we write one of these codes to the IOBASE_PORT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
	// Note: doesn't really matter, just shouldn't clash with QEMU's default codes
    Success = 0b1010, 	// 21 after left bitwise shift and bitwise OR 1.
    Failed = 0b1011, 	// 23 after ^
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(IOBASE_PORT);
        port.write(exit_code as u32);
    }
}

// Trait used to make our tests have common behavior (e.g. Test starting and success msging)
pub trait Testable {
    fn run(&self) -> ();
}

impl <T: Fn()> Testable for T {
	fn run(&self) -> () {
		// `type_name` is implemented directly by the compiler
		serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");	
	}
}

// `tests` - slice of trait object references pointing at the 
// [Fn()](https://doc.rust-lang.org/std/ops/trait.Fn.html) trait.
// - All functions annotated with `#[test_case]` will have their reference passed here.
#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) -> () {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
	exit_qemu(QemuExitCode::Success);
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	println!("Panic: {}", _info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	serial_println!("[failed]\n\nError: {}\n", _info);
	exit_qemu(QemuExitCode::Failed);
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

	#[test_case]
	fn a_test() {
		assert_eq!(1, 1);
	}

	#[test_case]
	fn failing_test() {
		// This is left in to demonstrate the panic_handler configured for test contexts
		assert_eq!(1, 2);
	}

}
