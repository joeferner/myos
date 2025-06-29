use crate::{Color, Position};
use crate::{FrameBufferDriver, Rect};
use ansi_escape::AnsiEscapeParser;
use conquer_once::{TryInitError, spin::OnceCell};
use pc_screen_font::Font;
use spin::Mutex;

static CONSOLE: OnceCell<Mutex<Console>> = OnceCell::uninit();

const DEFAULT_BG_COLOR: Color = Color {
    red: 0,
    green: 0,
    blue: 0,
};
const DEFAULT_FG_COLOR: Color = Color {
    red: 200,
    green: 200,
    blue: 200,
};

pub struct Console {
    driver: FrameBufferDriver,
    ansi_parser: AnsiEscapeParser,
    fg_color: Color,
    bg_color: Color,
    column: usize,
    row: usize,
    font: Font<'static>,
    bold_font: Font<'static>,
    bold: bool,
}

impl Console {
    pub fn init(
        driver: FrameBufferDriver,
        font: Font<'static>,
        bold_font: Font<'static>,
    ) -> Result<(), TryInitError> {
        CONSOLE.try_init_once(|| {
            spin::Mutex::new(Console {
                driver,
                ansi_parser: AnsiEscapeParser::new(),
                fg_color: DEFAULT_FG_COLOR,
                bg_color: DEFAULT_BG_COLOR,
                column: 0,
                row: 0,
                font,
                bold_font,
                bold: false,
            })
        })
    }

    pub fn clear() {
        if let Ok(console) = CONSOLE.try_get() {
            console.lock()._clear();
        }
    }

    pub fn reset_all_modes(&mut self) {
        self.fg_color = DEFAULT_FG_COLOR;
        self.bg_color = DEFAULT_BG_COLOR;
    }

    fn _clear(&mut self) {
        self.driver.clear(self.bg_color);
    }

    fn get_columns(&self) -> usize {
        self.driver.get_width() / self.font.width
    }

    fn get_rows(&self) -> usize {
        self.driver.get_height() / self.font.height
    }

    fn _write_char(&mut self, ch: char) {
        if ch == '\n' {
            self.next_line();
            return;
        }

        let pos = Position {
            x: self.column * self.font.width,
            y: self.row * self.font.height,
        };

        let font = if self.bold {
            &self.bold_font
        } else {
            &self.font
        };

        self.driver
            .draw_char(ch, pos, &font, self.fg_color, self.bg_color);
        self.column += 1;
        if self.column >= self.get_columns() {
            self.next_line();
        }
    }

    fn next_line(&mut self) {
        self.column = 0;
        self.row += 1;
        if self.row >= self.get_rows() {
            self.row -= 1;
            let iheight: Result<isize, _> = self.font.height.try_into();
            if let Ok(iheight) = iheight {
                self.driver.scroll_y(-iheight);
            }
            self.driver.draw_rect(
                Rect {
                    x: 0,
                    y: self.driver.get_height() - self.font.height,
                    height: self.font.height,
                    width: self.driver.get_width(),
                },
                self.bg_color,
            );
        }
    }

    fn set_cursor_position(&mut self, row: usize, column: usize) {
        self.row = column;
        if self.row >= self.get_rows() {
            self.row = self.get_rows() - 1;
        }
        self.column = row;
        if self.column >= self.get_columns() {
            self.column = self.get_columns() - 1;
        }
    }

    fn push_char(&mut self, ch: char) {
        match self.ansi_parser.push(ch) {
            ansi_escape::AnsiEvent::None => {}
            ansi_escape::AnsiEvent::ResetAllModes => self.reset_all_modes(),
            ansi_escape::AnsiEvent::Char(ch) => {
                self._write_char(ch);
            }
            ansi_escape::AnsiEvent::InvalidEscapeSequence(s) => {
                for ch in s.chars() {
                    self._write_char(ch);
                }
            }
            ansi_escape::AnsiEvent::SetForegroundColor(color) => {
                self.fg_color = color;
            }
            ansi_escape::AnsiEvent::SetBackgroundColor(color) => {
                self.bg_color = color;
            }
            ansi_escape::AnsiEvent::CursorHome => {
                self.set_cursor_position(0, 0);
            }
            ansi_escape::AnsiEvent::CursorTo(row, column) => {
                self.set_cursor_position(row.into(), column.into());
            }
            ansi_escape::AnsiEvent::CursorUp(val) => {
                self.set_cursor_position(self.column, self.row.saturating_sub(val.into()));
            }
            ansi_escape::AnsiEvent::CursorDown(val) => {
                self.set_cursor_position(self.column, self.row.saturating_add(val.into()));
            }
            ansi_escape::AnsiEvent::CursorRight(val) => {
                self.set_cursor_position(self.column.saturating_add(val.into()), self.row);
            }
            ansi_escape::AnsiEvent::CursorLeft(val) => {
                self.set_cursor_position(self.column.saturating_sub(val.into()), self.row);
            }
            ansi_escape::AnsiEvent::SetBoldMode => self.bold = true,
            ansi_escape::AnsiEvent::ResetBoldMode => self.bold = false,
        }
    }
}

impl core::fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for ch in s.chars() {
            self.push_char(ch);
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
