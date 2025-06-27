#![no_std]
#![no_main]

use bootloader_api::BootInfo;
use core::panic::PanicInfo;

use crate::console::Console;
use framebuffer::FrameBufferDriver;
use serial_port::serial1_init;

mod console;

bootloader_api::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    unsafe { serial1_init() }.expect("serial1 failed to init");

    let framebuffer = boot_info.framebuffer.take();

    if let Some(framebuffer) = framebuffer {
        let framebuffer = FrameBufferDriver::new(framebuffer);
        Console::init(framebuffer).expect("console failed to init");
        Console::clear();

        println!("Hello World 1!");
        println!("Hello World 2!");
        println!("this is a really long string this is a really long string this is a really long string this is a really long string this is a really long string this is a really long string this is a really long string this is a really long string this is a really long string this is a really long string this is a really long string this is a really long string this is a really long string");
    }
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
