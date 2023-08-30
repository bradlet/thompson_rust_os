//! should_panic_on_stack_overflow.rs
//! Not super happy w/ this test b/c it only tests the GDT's TSS but
//! not our actual IDT registration.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use lazy_static::lazy_static;
use core::panic::PanicInfo;
use x86_64::structures::idt::{InterruptStackFrame, InterruptDescriptorTable};
use thompson_rust_os::{exit_qemu, serial_print, serial_println, QemuExitCode, gdt};

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

pub fn init_test_idt() {
    TEST_IDT.load();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    gdt::init();
    init_test_idt();

    // trigger a stack overflow
    stack_overflow();

    panic!("Execution continued after stack overflow");
}

// Intentionally blow up the stack - uses this attribute to block
// compiler warnings
#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
	// prevent tail recursion optimizations; rustc does this by default!?
    volatile::Volatile::new(0).read();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}