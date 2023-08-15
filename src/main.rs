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

// `tests` - slice of trait object references pointing at the 
// [Fn()](https://doc.rust-lang.org/std/ops/trait.Fn.html) trait.
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
