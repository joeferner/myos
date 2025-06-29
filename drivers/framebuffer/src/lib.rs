#![no_std]

pub mod console;

use ansi_escape::Color;
use common::{FrameBuffer, PixelFormat};
use pc_screen_font::Font;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

pub struct FrameBufferDriver {
    framebuffer: FrameBuffer,
}

impl FrameBufferDriver {
    pub fn new(framebuffer: FrameBuffer) -> Self {
        Self { framebuffer }
    }

    pub fn clear(&mut self, color: Color) {
        let info = self.framebuffer.info();
        let rect = Rect {
            x: 0,
            y: 0,
            width: info.width,
            height: info.height,
        };
        self.draw_rect(rect, color);
    }

    pub fn draw_rect(&mut self, rect: Rect, color: Color) {
        let info = self.framebuffer.info();
        let pixel_buffer = self.framebuffer.buffer_mut();

        for y in 0..rect.height {
            let mut byte_offset = (rect.y + y) * info.stride * info.bytes_per_pixel;
            byte_offset += rect.x * info.bytes_per_pixel;
            for _x in 0..rect.width {
                if byte_offset >= pixel_buffer.len() {
                    return;
                }
                let pixel_buf = &mut pixel_buffer[byte_offset..];
                FrameBufferDriver::set_pixel_raw(pixel_buf, info.pixel_format, color);
                byte_offset += info.bytes_per_pixel;
            }
        }
    }

    #[allow(dead_code)]
    pub fn set_pixel(&mut self, position: Position, color: Color) {
        let info = self.framebuffer.info();

        // calculate offset to first byte of pixel
        let byte_offset = {
            // use stride to calculate pixel offset of target line
            let line_offset = position.y * info.stride;
            // add x position to get the absolute pixel offset in buffer
            let pixel_offset = line_offset + position.x;
            // convert to byte offset
            pixel_offset * info.bytes_per_pixel
        };
        // set pixel based on color format
        let pixel_buffer = &mut self.framebuffer.buffer_mut()[byte_offset..];
        if byte_offset >= pixel_buffer.len() {
            return;
        }
        FrameBufferDriver::set_pixel_raw(pixel_buffer, info.pixel_format, color);
    }

    #[allow(dead_code)]
    pub fn draw_str(
        &mut self,
        s: &str,
        position: Position,
        font: &Font,
        fg_color: Color,
        bg_color: Color,
    ) {
        let mut x = 0;
        for ch in s.chars() {
            self.draw_char(
                ch,
                Position {
                    x: position.x + x,
                    y: position.y,
                },
                font,
                fg_color,
                bg_color,
            );
            x += font.width;
        }
    }

    pub fn draw_char(
        &mut self,
        ch: char,
        position: Position,
        font: &Font,
        fg_color: Color,
        bg_color: Color,
    ) {
        let info = self.framebuffer.info();
        let pixel_buffer = &mut self.framebuffer.buffer_mut();
        font.render_char(ch, |x, y, v| {
            let color = if v { fg_color } else { bg_color };
            let byte_offset = {
                // use stride to calculate pixel offset of target line
                let line_offset = (position.y + y) * info.stride;
                // add x position to get the absolute pixel offset in buffer
                let pixel_offset = line_offset + (position.x + x);
                // convert to byte offset
                pixel_offset * info.bytes_per_pixel
            };
            if byte_offset >= pixel_buffer.len() {
                return;
            }
            let p = &mut pixel_buffer[byte_offset..];
            FrameBufferDriver::set_pixel_raw(p, info.pixel_format, color);
        });
    }

    fn set_pixel_raw(pixel_buffer: &mut [u8], pixel_format: PixelFormat, color: Color) {
        match pixel_format {
            PixelFormat::Rgb => {
                if pixel_buffer.len() < 3 {
                    return;
                }
                pixel_buffer[0] = color.red;
                pixel_buffer[1] = color.green;
                pixel_buffer[2] = color.blue;
            }
            PixelFormat::Bgr => {
                if pixel_buffer.len() < 3 {
                    return;
                }
                pixel_buffer[0] = color.blue;
                pixel_buffer[1] = color.green;
                pixel_buffer[2] = color.red;
            }
            PixelFormat::U8 => {
                if pixel_buffer.len() < 1 {
                    return;
                }
                // use a simple average-based grayscale transform
                let gray = color.red / 3 + color.green / 3 + color.blue / 3;
                pixel_buffer[0] = gray;
            }
            other => panic!("unknown pixel format {other:?}"),
        }
    }

    pub fn get_width(&self) -> usize {
        self.framebuffer.info().width
    }

    pub fn get_height(&self) -> usize {
        self.framebuffer.info().height
    }

    fn scroll_y(&mut self, offset: isize) {
        let info = self.framebuffer.info();
        let buffer = self.framebuffer.buffer_mut();

        if offset < 0 {
            let offset: usize = offset.abs() as usize;
            let from_offset = {
                let line_offset = offset * info.stride;
                line_offset * info.bytes_per_pixel
            };
            buffer.copy_within(from_offset..buffer.len(), 0);
        } else {
            let offset: usize = offset as usize;
            let to_offset = {
                let line_offset = offset * info.stride;
                line_offset * info.bytes_per_pixel
            };
            buffer.copy_within(0..buffer.len() - to_offset, to_offset);
        }
    }
}
