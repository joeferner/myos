#![no_std]
#![no_main]

extern crate alloc;

use core::slice;

use ansi_escape::{Ansi, Color};
use bootloader_api::{BootInfo, BootloaderConfig, config::Mapping, info::Optional};

use console::console_init;
use pci::PCI_DRIVER;
use serial_port::serial1_init;
use x86_64::VirtAddr;

use crate::memory::BootInfoFrameAllocator;

mod allocator;
mod console;
mod memory;

const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::FixedAddress(0x0000_6000_0000_0000));
    config
};

bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    unsafe { serial1_init() }.expect("serial1 failed to init");
    println!("after serial init");

    let framebuffer = boot_info.framebuffer.take();

    if let Some(framebuffer) = framebuffer {
        console_init(framebuffer).expect("console failed to init");
        println_status!("OK", "Console initialized.");
    }

    if let Optional::Some(physical_memory_offset) = boot_info.physical_memory_offset {
        let phys_mem_offset = VirtAddr::new(physical_memory_offset);
        let mut mapper = unsafe { memory::init(phys_mem_offset) };
        let mut frame_allocator =
            unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };
        allocator::init_heap(&mut mapper, &mut frame_allocator)
            .expect("heap initialization failed");
        println_status!("OK", "Allocator initialized.");
    }

    if let Optional::Some(ramdisk_addr) = boot_info.ramdisk_addr {
        println!(
            "ram disk 0x{ramdisk_addr:08x} (size: {})",
            boot_info.ramdisk_len
        );
        let data = unsafe { slice::from_raw_parts(ramdisk_addr as *const u8, boot_info.ramdisk_len as usize) };
        let disk = vsfs::io::Cursor::new(data);
        let vsfs = vsfs::FileSystem::new(&disk, vsfs::FsOptions::new()).unwrap();

        let root_dir = vsfs.root_dir();
        for entry in root_dir.iter() {
            println!("{entry:?}");
        }
    } else {
        println!("ram disk not found");
    }

    for pci_device in PCI_DRIVER.iterate_devices() {
        println!("{pci_device:?}");
    }

    loop {
        core::hint::spin_loop();
    }
}

// pub fn pci_enumerate() {
//     let port = PCI_CONFIG_PORT.lock();

//     for bus in 0..=255 {
//         for device in 0..32 {
//             let header = PciCommonHeader::new(PciAddress::new(bus, device, 0, 0));
//             if header.id(&*port).is_some() {
//                 print_device(&*port, &header, bus, device, 0);
//                 if header.has_multiple_functions(&*port) {
//                     for function in 1..8 {
//                         let header =
//                             PciCommonHeader::new(PciAddress::new(bus, device, function, 0));
//                         if header.id(&*port).is_some() {
//                             print_device(&*port, &header, bus, device, function);
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
