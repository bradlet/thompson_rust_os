//! interrupts.rs
//! Module that handles CPU Exceptions supported by types from
//! the `x86-64` crate.

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::println;

/// Create a new IDT instance so that we can register our various
/// exception handlers.
pub fn init_idt() {
    let mut idt = InterruptDescriptorTable::new();
}

/// Breakpoint exceptions are solely used to pause a program when the
/// breakpoint instruction `int3` is reached.
extern "x86-interrupt" fn breakpoint_handler(isf: InterruptStackFrame) {
	println!("{:#?}", isf);
}