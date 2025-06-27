#![no_std]
#![no_main]

use bootloader_api::BootInfo;
use core::panic::PanicInfo;

use crate::drivers::framebuffer::{self, FrameBufferDriver};
use pc_screen_font::{include_font_data, Font, FontData};

mod drivers;

bootloader_api::entry_point!(kernel_main);

include_font_data!(DEFAULT_8X16, "./resources/Tamsyn8x16r.psf");
include_font_data!(DEFAULT_8X16_BOLD, "./resources/Tamsyn8x16b.psf");

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let framebuffer = boot_info.framebuffer.take();

    if let Some(framebuffer) = framebuffer {
        let mut framebuffer = FrameBufferDriver::new(framebuffer);

        let bg_color = framebuffer::Color {
            red: 0,
            green: 0,
            blue: 0,
        };
        let fg_color = framebuffer::Color {
            red: 200,
            green: 200,
            blue: 200,
        };

        framebuffer.clear(bg_color);

        let font = Font::new(DEFAULT_8X16);
        let font_bold = Font::new(DEFAULT_8X16_BOLD);
        for y in (0..10 * font.height).step_by(font.height) {
            let pos = framebuffer::Position { x: 0, y };
            framebuffer.draw_str("Hello World!", pos, &font, fg_color, bg_color);
        }
        for y in (10 * font.height..20 * font.height).step_by(font.height) {
            let pos = framebuffer::Position { x: 0, y };
            framebuffer.draw_str("Hello World!", pos, &font_bold, fg_color, bg_color);
        }
    }
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
