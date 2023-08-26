//! should_panic_example.rs
//! This is an example for an integration test that can confirm some panic occurred
//! in the test. We can only run a single test case given this method, so we can
//! disable the test harness and avoid the need for a test runner.
//! 
//! See `Cargo.toml` for test harness disabling config...

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use thompson_rust_os::{exit_qemu, serial_print, serial_println, QemuExitCode};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("[failed]");
    exit_qemu(QemuExitCode::Failed);
    loop{}
}

fn should_fail() {
    serial_print!("should_panic_example::should_fail...\t");
    assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}