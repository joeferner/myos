use common::FrameBuffer;
use conquer_once::TryInitError;
use framebuffer::console::console_print_args;
use framebuffer::{console::Console, FrameBufferDriver};
use pc_screen_font::{include_font_data, Font, FontData};
use serial_port::serial_print_args;

include_font_data!(DEFAULT_8X16, "./resources/Tamsyn8x16r.psf");
include_font_data!(DEFAULT_8X16_BOLD, "./resources/Tamsyn8x16b.psf");

pub fn console_init(framebuffer: FrameBuffer) -> Result<(), TryInitError> {
    let framebuffer = FrameBufferDriver::new(framebuffer);
    let font = Font::new(DEFAULT_8X16);
    Console::init(framebuffer, font)?;
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
