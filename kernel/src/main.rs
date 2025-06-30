#![no_std]
#![no_main]

use ansi_escape::{Ansi, Color};
use bootloader_api::BootInfo;

use console::console_init;
use serial_port::serial1_init;

mod console;

bootloader_api::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    unsafe { serial1_init() }.expect("serial1 failed to init");
    println!("after serial init");

    let framebuffer = boot_info.framebuffer.take();

    if let Some(framebuffer) = framebuffer {
        console_init(framebuffer).expect("console failed to init");
    }

    for n in 0..80 {
        println!("line {n}");
    }
    println!("Hello World 2!");
    println!(
        "this is a really long string this is a really long string this is a really long string this is a really long string this is a really long string this is a really long string this is a really long string this is a really long string this is a really long string this is a really long string this is a really long string this is a really long string this is a really long string"
    );
    println!(
        "{}{}Hello world!{}{}",
        Ansi::ForegroundColor(Color::white()),
        Ansi::BackgroundColor(Color::red()),
        Ansi::DefaultForeground,
        Ansi::DefaultBackground
    );

    println!(
        "{}{}Hello{} From 10,10",
        Ansi::CursorTo(10, 10),
        Ansi::Bold,
        Ansi::ResetBold
    );

    loop {}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
