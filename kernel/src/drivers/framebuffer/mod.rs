use bootloader_api::info::{FrameBuffer, PixelFormat};

pub mod font;

pub use font::Font;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
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
                let pixel_buf = &mut pixel_buffer[byte_offset..];
                FrameBufferDriver::set_pixel_raw(pixel_buf, info.pixel_format, color);
                byte_offset += info.bytes_per_pixel;
            }
        }
    }

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
        FrameBufferDriver::set_pixel_raw(pixel_buffer, info.pixel_format, color);
    }

    pub fn draw_char(&mut self, pos: Position, font: &Font, color: Color) {}

    fn set_pixel_raw(pixel_buffer: &mut [u8], pixel_format: PixelFormat, color: Color) {
        match pixel_format {
            PixelFormat::Rgb => {
                pixel_buffer[0] = color.red;
                pixel_buffer[1] = color.green;
                pixel_buffer[2] = color.blue;
            }
            PixelFormat::Bgr => {
                pixel_buffer[0] = color.blue;
                pixel_buffer[1] = color.green;
                pixel_buffer[2] = color.red;
            }
            PixelFormat::U8 => {
                // use a simple average-based grayscale transform
                let gray = color.red / 3 + color.green / 3 + color.blue / 3;
                pixel_buffer[0] = gray;
            }
            other => panic!("unknown pixel format {other:?}"),
        }
    }
}
