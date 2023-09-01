//! lib.rs
//! Home for our custom test framework

#![no_std]
#![cfg_attr(test, no_main)]
// 1. Enable custom test framework because the default `test` crate requires the std lib.
// 2. Enable the x86_interrupt calling convention for exception handling (see interrupts.rs)
#![feature(custom_test_frameworks, abi_x86_interrupt)]
// This generates a main function that calls `test_runner`, but we configured no_main.
#![test_runner(crate::test_runner)]
// This is needed to change the name of the generated main function...
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

pub mod vga_buffer;
pub mod serial;
pub mod interrupts;
pub mod gdt;

const IOBASE_PORT: u16 = 0xf4;

/// Surface `interrupts` mod's IDT initializer for convenience /
/// to obviate the need for consumers of this lib to import the
/// interrupts module.
pub fn init() {
	gdt::init();
	interrupts::init_idt();
	// Initialize our interrupt controllers
	unsafe { interrupts::PICS.lock().initialize() };
	// Enable interrupts in the CPU configuration using the `sti` instruction
	x86_64::instructions::interrupts::enable(); 
}

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
pub fn test_runner(tests: &[&dyn Testable]) -> () {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
	exit_qemu(QemuExitCode::Success);
}


pub fn test_panic_handler(_info: &PanicInfo) -> ! {
	serial_println!("[failed]\n\nError: {}\n", _info);
	exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	test_panic_handler(_info)
}

/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
	init();
    test_main();
    loop {}
}
