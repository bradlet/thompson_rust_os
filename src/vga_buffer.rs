//! # VGA Display Driver
//! Creating a module to wrap unsafe interactions with the VGA Text Buffer

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// C-like enum for the sake of clarity
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
/// 		for the struct; just the size of the u8.
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

#[repr(transparent)]
pub struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
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
				if self.eol() { self.new_line(); }

				let row  = BUFFER_HEIGHT - 1;
				let col = self.column_position;
				let color_code = self.color_code;

				self.buffer.chars[row][col] = ScreenChar {
					ascii_character: byte,
					color_code
				};

				self.column_position += 1;
			}
		}
	}

	pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }

        }
    }

	fn new_line(&mut self) { }
}