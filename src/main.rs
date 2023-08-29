//! main.rs
//! Author: Bradley Thompson

// Stop implicitly linking the standard library - has OS dependencies.
#![no_std]
// Tell the Rust compiler that we don't want to use the normal entrypoint
// chain with crt0.
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(thompson_rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use thompson_rust_os::println;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	println!("Panic: {}", _info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    thompson_rust_os::test_panic_handler(info)
}

/// - `!` specifies this as a diverging fn; entry point should invoke the `exit` syscall.
/// - Throws linker error by default b/c program depends on C runtime. Build for bare metal to fix.
#[no_mangle]
pub extern "C" fn _start() -> ! {
	println!("Hello {}!", "World");

	// Setup OS lib with Interrupt Descriptor Table registration
	thompson_rust_os::init();

	x86_64::instructions::interrupts::int3(); // Test breakpoint exception

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

	// Uncomment to see what a failing test looks liks:
	// #[test_case]
	// fn failing_test() {
	// 	assert_eq!(1, 2);
	// }
}
