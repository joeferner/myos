use crate::{Color, FrameBuffer, Position};
use crate::{FrameBufferDriver, Rect};
use ansi_escape::{Ansi, AnsiEscapeParser, AnsiEscapeParserError};
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
            Ok(event) => {
                if let Some(ansi) = event {
                    match ansi {
                        Ansi::ResetAllModes => self.reset_all_modes(),
                        Ansi::Char(ch) => {
                            self._write_char(ch);
                        }
                        Ansi::ForegroundColor(color) => {
                            self.fg_color = color;
                        }
                        Ansi::BackgroundColor(color) => {
                            self.bg_color = color;
                        }
                        Ansi::CursorHome => {
                            self.set_cursor_position(0, 0);
                        }
                        Ansi::CursorTo(row, column) => {
                            self.set_cursor_position(row.into(), column.into());
                        }
                        Ansi::CursorUp(val) => {
                            self.set_cursor_position(
                                self.column,
                                self.row.saturating_sub(val.into()),
                            );
                        }
                        Ansi::CursorDown(val) => {
                            self.set_cursor_position(
                                self.column,
                                self.row.saturating_add(val.into()),
                            );
                        }
                        Ansi::CursorRight(val) => {
                            self.set_cursor_position(
                                self.column.saturating_add(val.into()),
                                self.row,
                            );
                        }
                        Ansi::CursorLeft(val) => {
                            self.set_cursor_position(
                                self.column.saturating_sub(val.into()),
                                self.row,
                            );
                        }
                        Ansi::Bold => self.bold = true,
                        Ansi::ResetBold => self.bold = false,
                        Ansi::DefaultForeground => self.fg_color = DEFAULT_FG_COLOR,
                        Ansi::DefaultBackground => self.bg_color = DEFAULT_BG_COLOR,
                    }
                }
            }
            Err(err) => match err {
                AnsiEscapeParserError::InvalidEscapeSequence(seq) => {
                    for ch in seq.chars() {
                        self._write_char(ch);
                    }
                }
            },
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
    use std::{format, fs};
    use zune_core::options::EncoderOptions;
    use zune_ppm::{PPMDecoder, PPMEncoder};

    use super::*;
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

    fn write_framebuffer_to_file<const N: usize>(framebuffer: &MockFrameBuffer<N>, dest: &str) {
        let width = framebuffer.width();
        let height = framebuffer.height();
        let found_ppm = PPMEncoder::new(
            &framebuffer.buffer,
            EncoderOptions::new(
                width,
                height,
                zune_core::colorspace::ColorSpace::RGB,
                zune_core::bit_depth::BitDepth::Eight,
            ),
        )
        .encode()
        .unwrap();
        fs::write(dest, found_ppm).unwrap();
    }

    #[test]
    pub fn hello_world() {
        let hello_world_ppm = include_bytes!("../resources/test/console/hello_world.ppm");

        const WIDTH: usize = 128;
        const HEIGHT: usize = 64;
        let framebuffer = MockFrameBuffer {
            width: WIDTH,
            height: HEIGHT,
            bytes_per_pixel: 3,
            pixel_format: PixelFormat::Rgb,
            stride: 128,
            buffer: [0; 3 * HEIGHT * WIDTH],
        };
        let driver = FrameBufferDriver::new(framebuffer);
        let font = Font::parse(DEFAULT_8X16).unwrap();
        let bold_font = Font::parse(DEFAULT_8X16_BOLD).unwrap();
        let mut console = Console::new(driver, font, bold_font);

        console
            .write_str(&format!(
                "{}{}Hello{}{} World",
                Ansi::ForegroundColor(Color::red()),
                Ansi::BackgroundColor(Color::green()),
                Ansi::DefaultForeground,
                Ansi::DefaultBackground
            ))
            .unwrap();

        write_framebuffer_to_file(&console.driver.framebuffer, "/tmp/hello_world.ppm");

        let data = PPMDecoder::new(hello_world_ppm).decode().unwrap();
        assert_eq!(
            console.driver.framebuffer.buffer.to_vec(),
            data.u8().unwrap()
        );
    }
}
