//! serial.rs
//! This module implements interactions with the serial port. This is a simple way
//! to communicate information from QEMU, running our kernel, back to the host
//! stdout (or any other part of its file system).
//! 
//! This module's print macros should be used in test contexts to report state to
//! the host machine.

use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

const SERIAL_PORT: u16 = 0x3F8; // Standard port number for UART's first serial interface

lazy_static! {
    pub static ref SERIAL: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(SERIAL_PORT) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

/* Similar to the VGA Buffer, add macros to make interacting w/ the SerialPort simpler */

// I don't hide this from docs like the tutorial does b/c this is all just for learning/reference
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
	// `SerialPort` implements the `fmt::Write` trait
    SERIAL.lock().write_fmt(args).expect("Printing to serial failed");
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}