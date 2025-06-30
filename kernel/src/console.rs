use conquer_once::{TryInitError, spin::OnceCell};
use framebuffer::{FrameBuffer, FrameBufferDriver, console::Console};
use pc_screen_font::Font;
use serial_port::serial_print_args;
use spin::Mutex;

struct MyFrameBuffer {
    framebuffer: bootloader_api::info::FrameBuffer,
}

impl MyFrameBuffer {
    pub fn new(framebuffer: bootloader_api::info::FrameBuffer) -> Self {
        MyFrameBuffer { framebuffer }
    }
}

impl FrameBuffer for MyFrameBuffer {
    fn width(&self) -> usize {
        self.framebuffer.info().width
    }

    fn height(&self) -> usize {
        self.framebuffer.info().height
    }

    fn stride(&self) -> usize {
        self.framebuffer.info().stride
    }

    fn bytes_per_pixel(&self) -> usize {
        self.framebuffer.info().bytes_per_pixel
    }

    fn pixel_format(&self) -> common::PixelFormat {
        self.framebuffer.info().pixel_format
    }

    fn buffer_mut(&mut self) -> &mut [u8] {
        self.framebuffer.buffer_mut()
    }
}

static CONSOLE: OnceCell<Mutex<Console<MyFrameBuffer>>> = OnceCell::uninit();

const DEFAULT_8X16: &[u8] = include_bytes!("./resources/Tamsyn8x16r.psf");
const DEFAULT_8X16_BOLD: &[u8] = include_bytes!("./resources/Tamsyn8x16b.psf");

pub fn console_init(framebuffer: bootloader_api::info::FrameBuffer) -> Result<(), TryInitError> {
    let framebuffer = FrameBufferDriver::new(MyFrameBuffer::new(framebuffer));
    let font = Font::new(DEFAULT_8X16);
    let bold_font = Font::new(DEFAULT_8X16_BOLD);
    CONSOLE.try_init_once(|| {
        let mut console = Console::new(framebuffer, font, bold_font);
        console.clear();
        Mutex::new(console)
    })
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

pub fn console_print_args(args: core::fmt::Arguments) -> core::fmt::Result {
    use core::fmt::Write;
    if let Ok(console) = CONSOLE.try_get() {
        console.lock().write_fmt(args)
    } else {
        Ok(())
    }
}
