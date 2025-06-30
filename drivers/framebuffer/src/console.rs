use crate::{Color, FrameBuffer, Position};
use crate::{FrameBufferDriver, Rect};
use ansi_escape::AnsiEscapeParser;
use pc_screen_font::Font;

const DEFAULT_BG_COLOR: Color = Color::black();
const DEFAULT_FG_COLOR: Color = Color::rgb(200, 200, 200);

pub struct Console<TFrameBuffer: FrameBuffer> {
    driver: FrameBufferDriver<TFrameBuffer>,
    ansi_parser: AnsiEscapeParser,
    fg_color: Color,
    bg_color: Color,
    column: usize,
    row: usize,
    font: Font<'static>,
    bold_font: Font<'static>,
    bold: bool,
}

impl<TFrameBuffer: FrameBuffer> Console<TFrameBuffer> {
    pub fn new(
        driver: FrameBufferDriver<TFrameBuffer>,
        font: Font<'static>,
        bold_font: Font<'static>,
    ) -> Self {
        Console {
            driver,
            ansi_parser: AnsiEscapeParser::new(),
            fg_color: DEFAULT_FG_COLOR,
            bg_color: DEFAULT_BG_COLOR,
            column: 0,
            row: 0,
            font,
            bold_font,
            bold: false,
        }
    }

    pub fn clear(&mut self) {
        self.driver.clear(self.bg_color);
    }

    pub fn reset_all_modes(&mut self) {
        self.fg_color = DEFAULT_FG_COLOR;
        self.bg_color = DEFAULT_BG_COLOR;
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
            ansi_escape::AnsiEvent::DefaultForeground => self.fg_color = DEFAULT_FG_COLOR,
            ansi_escape::AnsiEvent::DefaultBackground => self.bg_color = DEFAULT_BG_COLOR,
        }
    }
}

impl<TFrameBuffer: FrameBuffer> core::fmt::Write for Console<TFrameBuffer> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for ch in s.chars() {
            self.push_char(ch);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use core::fmt::Write;
    use std::format;
    use zune_ppm::PPMDecoder;

    use super::*;
    use ansi_escape::codes::Ansi;
    use common::PixelFormat;
    use pc_screen_font::Font;

    const DEFAULT_8X16: &[u8] = include_bytes!("../resources/Tamsyn8x16r.psf");
    const DEFAULT_8X16_BOLD: &[u8] = include_bytes!("../resources/Tamsyn8x16b.psf");

    struct MockFrameBuffer<const N: usize> {
        width: usize,
        height: usize,
        stride: usize,
        bytes_per_pixel: usize,
        pixel_format: common::PixelFormat,
        buffer: [u8; N],
    }

    impl<const N: usize> FrameBuffer for MockFrameBuffer<N> {
        fn width(&self) -> usize {
            self.width
        }

        fn height(&self) -> usize {
            self.height
        }

        fn stride(&self) -> usize {
            self.stride
        }

        fn bytes_per_pixel(&self) -> usize {
            self.bytes_per_pixel
        }

        fn pixel_format(&self) -> common::PixelFormat {
            self.pixel_format
        }

        fn buffer_mut(&mut self) -> &mut [u8] {
            &mut self.buffer
        }
    }

    #[test]
    pub fn hello_world() {
        let hello_world_ppm = include_bytes!("../resources/test/console/hello_world.ppm");

        let framebuffer = MockFrameBuffer {
            width: 128,
            height: 64,
            bytes_per_pixel: 3,
            pixel_format: PixelFormat::Rgb,
            stride: 128,
            buffer: [0; 3 * 64 * 128],
        };
        let driver = FrameBufferDriver::new(framebuffer);
        let font = Font::new(DEFAULT_8X16);
        let bold_font = Font::new(DEFAULT_8X16_BOLD);
        let mut console = Console::new(driver, font, bold_font);

        console
            .write_str(&format!(
                "{} World",
                Ansi::fg(Color::red(), &Ansi::bg(Color::green(), "Hello"))
            ))
            .unwrap();

        let data = PPMDecoder::new(hello_world_ppm).decode().unwrap();
        assert_eq!(
            console.driver.framebuffer.buffer.to_vec(),
            data.u8().unwrap()
        );
    }
}
