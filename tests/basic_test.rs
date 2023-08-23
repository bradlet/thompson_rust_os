#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(thompson_rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}
use thompson_rust_os;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    thompson_rust_os::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    thompson_rust_os::println!("test_println output");
}
