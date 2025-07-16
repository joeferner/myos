#![no_std]
#![no_main]

extern crate alloc;

use ansi_escape::{Ansi, Color};
use bootloader_api::{BootInfo, BootloaderConfig, config::Mapping, info::Optional};

use console::console_init;
use pci::{PciDevice, PciDriver};
use serial_port::serial1_init;
use x86_64::VirtAddr;

use crate::{memory::BootInfoFrameAllocator, pci::pci_enumerate};

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

    let pci = PciDriver::new();
    for pci_device in pci.iterate_devices() {
        print_pci_device(pci_device);
    }

    loop {
        core::hint::spin_loop();
    }
}

fn print_pci_device(device: &PciDevice) {

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


// fn print_device<T: PciConfigPort>(
//     port: &T,
//     header: &PciCommonHeader,
//     bus: u8,
//     device: u8,
//     func: u8,
// ) {
//     if let Some((vendor_id, device_id)) = header.id(port) {
//         let header_type = header.header_type(port);
//         let (class_code, sub_class_code) = header.class_code(port);
//         let prog_if = header.prog_if(port);
//         println!(
//             "{bus}:{device}.{func} => {vendor_id:04x} {device_id:04x} ht:{header_type:?} cc:{class_code:?} scc:{sub_class_code:02x} pif:{prog_if:02x}"
//         );
//     } else {
//         println!("{bus}:{device}.{func} => unavailable");
//     }
// }

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
