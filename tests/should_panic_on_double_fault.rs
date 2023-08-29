#![no_std]
#![no_main]

use core::panic::PanicInfo;
use thompson_rust_os::{exit_qemu, serial_println, QemuExitCode};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    thompson_rust_os::init();

    // trigger a page fault
    unsafe {
        *(0xdeadbeef as *mut u8) = 42;
    };

    serial_println!("[failed]");
    exit_qemu(QemuExitCode::Failed);
    loop{}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}