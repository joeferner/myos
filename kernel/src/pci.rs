use spin::mutex::Mutex;
use x86_64::instructions::port::{PortGeneric, ReadWriteAccess};

use crate::println;

const CONFIG_ADDRESS: u16 = 0xcf8;
const CONFIG_ADDRESS_PORT: Mutex<PortGeneric<u32, ReadWriteAccess>> =
    Mutex::new(PortGeneric::<u32, ReadWriteAccess>::new(CONFIG_ADDRESS));
const CONFIG_DATA: u16 = 0xcfc;
const CONFIG_DATA_PORT: Mutex<PortGeneric<u32, ReadWriteAccess>> =
    Mutex::new(PortGeneric::<u32, ReadWriteAccess>::new(CONFIG_DATA));

#[repr(C, packed)]
struct PciCommonHeader {
    pub vendor_id: u16,
    pub device_id: u16,
    pub command: u16,
    pub status: u16,
    pub revision_id: u8,
    pub prog_if: u8,
    pub subclass: u8,
    pub class_code: u8,
    pub cache_line_size: u8,
    pub latency_timer: u8,
    pub header_type: u8,
    pub bist: u8,
}

pub fn pci_enumerate() {
    for bus in 0..=255 {
        for device in 0..32 {
            if let Some(vendor) = pci_read_vendor(bus, device) {
                let device_id = pci_read_device(bus, device).unwrap();
                println!("{bus}:{device} => {vendor:x} {device_id:x}");
                let header_type = pci_read_header_type(bus, device, 0);
            }
        }
    }
}

fn pci_read_vendor(bus: u8, device: u8) -> Option<u16> {
    let vendor = pic_config_read_word(bus, device, 0, 0);
    if vendor == 0xffff { None } else { Some(vendor) }
}

fn pci_read_device(bus: u8, device: u8) -> Option<u16> {
    let device = pic_config_read_word(bus, device, 0, 2);
    if device == 0xffff { None } else { Some(device) }
}

fn pci_read_header(bus: u8, device: u8) -> Option<PciCommonHeader> {
    let vendor_id = pic_config_read_word(bus, device, 0, 0);
    if vendor_id == 0xffff {
        return None;
    }

    let device_id = pic_config_read_word(bus, device, 0, 2);
    if device_id == 0xffff {
        return None;
    }

    Some(PciCommonHeader {
        vendor_id,
        device_id,
        command,
        status,
        revision_id,
        prog_if,
        subclass,
        class_code,
        cache_line_size,
        latency_timer,
        header_type,
        bist,
    })
}

fn pic_config_read_word(bus: u8, device: u8, func: u8, offset: u8) -> u16 {
    let bus: u32 = bus.into();
    let slot: u32 = device.into();
    let func: u32 = func.into();
    let offset: u32 = offset.into();

    let address: u32 = (bus << 16) | (slot << 11) | (func << 8) | (offset & 0xfc) | 0x80000000;

    unsafe {
        CONFIG_ADDRESS_PORT.lock().write(address);
    }

    let r = unsafe { CONFIG_DATA_PORT.lock().read() };

    return ((r >> ((offset & 2) * 8)) & 0xffff) as u16;
}
