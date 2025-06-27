use crate::FrameBufferDriver;
use crate::{Color, Position};
use conquer_once::{spin::OnceCell, TryInitError};
use pc_screen_font::Font;
use spin::Mutex;

static CONSOLE: OnceCell<Mutex<Console>> = OnceCell::uninit();

pub struct Console {
    driver: FrameBufferDriver,
    fg_color: Color,
    bg_color: Color,
    column: usize,
    row: usize,
    font: Font<'static>,
}

impl Console {
    pub fn init(driver: FrameBufferDriver, font: Font<'static>) -> Result<(), TryInitError> {
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
        if self.column >= self.driver.get_width() / self.font.width {
            self.column = 0;
            self.row += 1;
        }
    }
}

impl core::fmt::Write for Console {
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

pub fn console_print_args(args: core::fmt::Arguments) -> core::fmt::Result {
    use core::fmt::Write;
    if let Ok(console) = CONSOLE.try_get() {
        console.lock().write_fmt(args)
    } else {
        Ok(())
    }
}
