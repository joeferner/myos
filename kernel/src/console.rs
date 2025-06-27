use crate::drivers::framebuffer::FrameBufferDriver;
use crate::framebuffer::{Color, Position};
use conquer_once::{spin::OnceCell, TryInitError};
use pc_screen_font::{include_font_data, Font, FontData};
use spin::Mutex;

include_font_data!(DEFAULT_8X16, "./resources/Tamsyn8x16r.psf");
include_font_data!(DEFAULT_8X16_BOLD, "./resources/Tamsyn8x16b.psf");

static CONSOLE: OnceCell<Mutex<Console>> = OnceCell::uninit();

pub struct Console<'a> {
    driver: FrameBufferDriver,
    fg_color: Color,
    bg_color: Color,
    column: usize,
    row: usize,
    font: Font<'a>,
}

impl<'a> Console<'a> {
    pub fn init(driver: FrameBufferDriver) -> Result<(), TryInitError> {
        let bg_color = Color {
            red: 0,
            green: 0,
            blue: 0,
        };
        let fg_color = Color {
            red: 200,
            green: 200,
            blue: 200,
        };
        let font = Font::new(DEFAULT_8X16);

        CONSOLE.try_init_once(|| {
            spin::Mutex::new(Console {
                driver,
                fg_color,
                bg_color,
                column: 0,
                row: 0,
                font,
            })
        })
    }

    pub fn clear() {
        if let Ok(console) = CONSOLE.try_get() {
            console.lock()._clear();
        }
    }

    fn _clear(&mut self) {
        self.driver.clear(self.bg_color);
    }

    fn _write_char(&mut self, ch: char) {
        let pos = Position {
            x: self.column * self.font.width,
            y: self.row * self.font.height,
        };
        self.driver
            .draw_char(ch, pos, &self.font, self.fg_color, self.bg_color);
        self.column += 1;
        if self.column >= self.driver.framebuffer.info().width / self.font.width {
            self.column = 0;
            self.row += 1;
        }
    }
}

impl<'a> core::fmt::Write for Console<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for ch in s.chars() {
            match ch {
                '\n' => {
                    self.column = 0;
                    self.row += 1;
                }
                ch => self._write_char(ch),
            }
        }
        Ok(())
    }
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
    use core::fmt::Write;
    if let Ok(console) = CONSOLE.try_get() {
        console.lock().write_fmt(args).unwrap();
    }
}
