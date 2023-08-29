//! interrupts.rs
//! Module that handles CPU Exceptions supported by types from
//! the `x86-64` crate.

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::{println, gdt};
use lazy_static::lazy_static;

// Note to self: See [ref](https://doc.rust-lang.org/std/keyword.ref.html) docs
// for a bit more understanding on this macro.
lazy_static! {
	/// The IDT needs to live for the life of the program b/c an exception
	/// can occur at any point. Issue: We need to use runtime logic to
	/// mutate the IDT. So, we use `lazy_static!` to handle what would
	/// otherwise be `unsafe` operations lazily at first access.
	static ref IDT: InterruptDescriptorTable = {
		let mut idt = InterruptDescriptorTable::new();
		idt.breakpoint.set_handler_fn(breakpoint_handler);
		unsafe {
			idt.double_fault
				.set_handler_fn(double_fault_handler)
				.set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX); // unsafe
		}
		idt
	};
}

pub fn init_idt() {
	IDT.load()
}

/// Breakpoint exceptions are solely used to pause a program when the
/// breakpoint instruction `int3` is reached.
extern "x86-interrupt" fn breakpoint_handler(isf: InterruptStackFrame) {
	println!("{:#?}", isf);
}

/// Called when another exception occurs that is not handled.
extern "x86-interrupt" fn double_fault_handler(
	isf: InterruptStackFrame,
	_: u64 // Error code is always 0, not needed.
) -> ! {
	panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", isf);
}

#[cfg(test)]
mod tests {

	#[test_case]
	fn test_breakpoint_exception_handler() {
		// Our breakpoint handler should run and then execution should continue
		x86_64::instructions::interrupts::int3();
	}
}
