//! # VGA Display Driver
//! Creating a module to wrap unsafe interactions with the VGA Text Buffer

use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

const VGA_BUFFER_ADDRESS: u32 = 0xb8000;
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// Need to use this `lazy_static!` macro b/c Rust's const evaluator can't convert
// the raw Buffer ptr to a ref at compile time (maybe it can with const fn's now?
// unclear...). To make the static Writer mutable, we need to wrap it in simple
// Mutex spinlock.
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Black, Color::White),
        buffer: unsafe { &mut *(VGA_BUFFER_ADDRESS as *mut Buffer) },
    });
}

// C-like enum so we can explicitly match the correct color value
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    White = 15,
}

/// A type used to wrap the foreground and background colors.
/// Because foreground and background colors are only stored in 4 bits,
/// we need to use bitwise operations to shift the inputs into the
/// 4 most significant bits (bg color) and 4 least significant (fg color).
/// - Note: `repr(transparent)` basically gets rid of excess memory storage
///     for the struct; just the size of the u8.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(bg: Color, fg: Color) -> ColorCode {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
}

// Need `repr(C)` b/c by default struct fields are not ordered in Rust;
// in C they are guarunteed to have the same order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

/// Memory representation for our VGA buffer;
/// Wrapped `ScreenChar` objects in Volatile so that more heavily-optimizing
/// compilers don't clear the memory -- from the compiler's perspective, we
/// don't access data, so it seems like we could clear memory at any time.
#[repr(transparent)]
pub struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// The public interface with which to interact with the our VGA driver.
/// - Always writes to last line and shifts up if fill or when encountering '\n'.
/// - Reference to VGA buffer needs to live for the entire program life: `'static`.
pub struct Writer {
    pub column_position: usize,
    pub color_code: ColorCode,
    pub buffer: &'static mut Buffer,
}

impl Writer {
    fn eol(&self) -> bool {
        self.column_position >= BUFFER_WIDTH
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.eol() {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });

                self.column_position += 1;
            }
        }
    }

    /// Print any valid ASCII (technically 'code page 937') character.
    /// Any characters outside of the valid range will have a square.
    /// Any byte within a multi-byte UTF-8 character is not valid ASCII,
    /// so some unicode characters will result in multiple square chars.
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Shift all rows up 1  
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    /// Replace target `row` with all empty space characters
    fn clear_row(&mut self, row: usize) {
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            })
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/*
Create our own declarative print & println macros which use the above `fmt::Write`
impl instead of the Rust IO module's `_print` function. Only real diff is ours
writes to the VGA buffer instead of stdout...
*/

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::Write;

    #[test_case]
    fn test_println_runs() {
        println!("println works...");
    }

    #[test_case]
    fn test_println_supports_many_lines() {
        for _ in 0..100 {
            println!("Multiple println's works");
        }
    }

    #[test_case]
    fn test_println_output() {
        use x86_64::instructions::interrupts;

        let s = "Test string";

        interrupts::without_interrupts(|| {
            let mut writer = WRITER.lock();
            writeln!(writer, "\n{}", s).expect("writeln failed");
            for (i, c) in s.chars().enumerate() {
                // BUFFER_HEIGHT - 2 because after println the newline causes the last line to 'shift up'.
                let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
                assert_eq!(char::from(screen_char.ascii_character), c);
            }
        });
    }

    #[test_case]
    fn test_line_wrapped_at_buffer_width() {
        use x86_64::instructions::interrupts;

        let s = "Test a long string that should be wrapped at BUFFER_WIDTH characters..........80separate line!";

        interrupts::without_interrupts(|| {
            let mut writer = WRITER.lock();
            writeln!(writer, "\n{}", s).expect("writeln failed");
            for (i, c) in s.chars().enumerate() {
                // If we are past col 80, a newline should have been inserted.
                let row = if i < 80 {
                    BUFFER_HEIGHT - 3
                } else {
                    BUFFER_HEIGHT - 2
                };
                let col = if i >= 80 { i - 80 } else { i };
                let screen_char = writer.buffer.chars[row][col].read();
                assert_eq!(char::from(screen_char.ascii_character), c);
            }
        });
    }
}
