#![no_std]
#![no_main]

use bootloader_api::BootInfo;
use core::panic::PanicInfo;

use crate::drivers::framebuffer::{self, FrameBufferDriver};

mod drivers;

bootloader_api::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let framebuffer = boot_info.framebuffer.take();

    if let Some(framebuffer) = framebuffer {
        let mut framebuffer = FrameBufferDriver::new(framebuffer);

        let color = framebuffer::Color {
            red: 0,
            green: 0,
            blue: 0,
        };
        framebuffer.clear(color);

        let rect = framebuffer::Rect {
            x: 20,
            y: 100,
            width: 200,
            height: 200,
        };
        let color = framebuffer::Color {
            red: 0,
            green: 0,
            blue: 255,
        };
        framebuffer.draw_rect(rect, color);
    }
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
