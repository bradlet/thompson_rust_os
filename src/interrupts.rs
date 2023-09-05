//! interrupts.rs
//! Module that handles CPU Exceptions supported by types from
//! the `x86-64` crate, and interrupts sent to the Intel 8259
//! chained Programmable Interrupt Controller interface.

use crate::{gdt, print, println};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

/// Intel 8259 has two PIC's; these need to be at a higher interrupt vector
/// value b/c lower values are used by other interrupts, like CPU exceptions.
pub const PIC_1_OFFSET: u8 = 32;
/// There are two PIC's in Intel 8259, the second feeds into one of PIC_1's
/// input interrupt lines, and then PIC_1 communicates with the CPU.
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
/// Port that we read keyboard scancodes on -- must be read before handling
/// another keyboard interrupt.
pub const PS2_CONTROLLER_IO_PORT: u16 = 0x60;

pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe {
    // ChainedPics::new is unsafe b/c bad offsets can yield undefined behavior.
    ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
});

/// Define the different hardware interrupt values, starting from the PIC_1_OFFSET
/// which is high enough to leave room for CPU exceptions and other types of
/// interrupts.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
	Keyboard, // Defaults to previous + 1
}

impl InterruptIndex {
    fn as_usize(self) -> usize {
        usize::from(self as u8)
    }
}

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
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
		idt[InterruptIndex::Keyboard.as_usize()]
			.set_handler_fn(keyboard_interrupt_handler);
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
    _: u64, // Error code is always 0, not needed.
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", isf);
}

/// Sent every time the Programmable Interval Timer periodically ticks.
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!(".");
    // Need to send an EOI signal to let CPU know the handling of the
    // last interrupts is complete.
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}

/// Handler for any keyboard interrupt.
/// We read the scancode, which uses the scancode set 1, 
/// ["IBM XT"](https://en.wikipedia.org/wiki/IBM_Personal_Computer_XT).
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame){
	use pc_keyboard::{layouts::Us104Key, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
	use spin::Mutex;
	use x86_64::instructions::port::Port;

	lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<Us104Key, ScancodeSet1>> =
            Mutex::new(
				Keyboard::new(
					Us104Key, ScancodeSet1, HandleControl::Ignore
				)
            );
    }

	let mut keyboard = KEYBOARD.lock();
	let mut port = Port::new(PS2_CONTROLLER_IO_PORT);

	let scancode: u8 = unsafe { port.read() };
	if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

	unsafe {
		PICS.lock()
			.notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
	}
}

#[cfg(test)]
mod tests {

    #[test_case]
    fn test_breakpoint_exception_handler() {
        // Our breakpoint handler should run and then execution should continue
        x86_64::instructions::interrupts::int3();
    }
}
