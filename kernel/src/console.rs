use common::FrameBuffer;
use conquer_once::TryInitError;
use framebuffer::console::console_print_args;
use framebuffer::{FrameBufferDriver, console::Console};
use pc_screen_font::{Font, FontData, include_font_data};
use serial_port::serial_print_args;

include_font_data!(DEFAULT_8X16, "./resources/Tamsyn8x16r.psf");
include_font_data!(DEFAULT_8X16_BOLD, "./resources/Tamsyn8x16b.psf");

pub fn console_init(framebuffer: FrameBuffer) -> Result<(), TryInitError> {
    let framebuffer = FrameBufferDriver::new(framebuffer);
    let font = Font::new(DEFAULT_8X16);
    let bold_font = Font::new(DEFAULT_8X16_BOLD);
    Console::init(framebuffer, font, bold_font)?;
    Console::clear();
    Ok(())
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    let _ = serial_print_args(args);
    let _ = console_print_args(args);
}
