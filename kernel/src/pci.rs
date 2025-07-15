use bit_field::BitField;
use spin::mutex::Mutex;
use x86_64::instructions::port::{PortGeneric, ReadWriteAccess};

use crate::println;

const CONFIG_ADDRESS: u16 = 0xcf8;
const CONFIG_ADDRESS_PORT: Mutex<PortGeneric<u32, ReadWriteAccess>> =
    Mutex::new(PortGeneric::<u32, ReadWriteAccess>::new(CONFIG_ADDRESS));
const CONFIG_DATA: u16 = 0xcfc;
const CONFIG_DATA_PORT: Mutex<PortGeneric<u32, ReadWriteAccess>> =
    Mutex::new(PortGeneric::<u32, ReadWriteAccess>::new(CONFIG_DATA));

/// The address of a PCIe function.
///
/// PCIe supports 65536 segments, each with 256 buses, each with 32 slots, each with 8 possible functions.:
///
/// ```ignore
/// 32                              16               8         3      0
///  +-------------------------------+---------------+---------+------+
///  |            segment            |      bus      | device  | func |
///  +-------------------------------+---------------+---------+------+
/// ```
struct PciAddress(u32);

impl PciAddress {
    pub fn new(bus: u8, device: u8, func: u8, offset: u8) -> Self {
        let bus: u32 = bus.into();
        let slot: u32 = device.into();
        let func: u32 = func.into();
        let offset: u32 = offset.into();
        let address: u32 = (bus << 16) | (slot << 11) | (func << 8) | (offset & 0xfc) | 0x80000000;
        Self(address)
    }
}

pub type VendorId = u16;
pub type DeviceId = u16;
pub type HasMultipleFunctions = bool;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HeaderType {
    Endpoint,
    PciPciBridge,
    CardBusBridge,
    Unknown(u8),
}

/// ```ignore
///       31          24 23         16 15          8 7                0
///       +-------------+-------------+-------------+-----------------+
/// 0x00  |        Device ID          |         Vendor ID             |
/// 0x04  |          Status           |          Command              |
/// 0x08  |        Class Code                       | Revision ID     |
/// 0x0c  |     0x00    | Header Type |  0x00       | Cache Line Size |
///       +-------------+-------------+-------------+-----------------+
/// ```
struct PciCommonHeader(PciAddress);

impl PciCommonHeader {
    pub const fn new(address: PciAddress) -> Self {
        Self(address)
    }

    pub fn id(&self) -> Option<(VendorId, DeviceId)> {
        let data = pci_read(&self.0, 0x00);
        if data == 0xffff_ffff {
            return None;
        }
        let vendor_id = data.get_bits(0..16) as VendorId;
        let device_id = data.get_bits(16..32) as DeviceId;
        Some((vendor_id, device_id))
    }

    pub fn header_type(&self) -> (HasMultipleFunctions, HeaderType) {
        let data = pci_read(&self.0, 0x0c);
        // high level bit 23 contains
        let header_type = data.get_bits(16..23);
        let header_type = match header_type {
            0x00 => HeaderType::Endpoint,
            0x01 => HeaderType::PciPciBridge,
            0x02 => HeaderType::CardBusBridge,
            v => HeaderType::Unknown(v as u8),
        };
        let has_multiple_functions = data.get_bit(23);
        (has_multiple_functions, header_type)
    }
}

pub fn pci_enumerate() {
    for bus in 0..=255 {
        for device in 0..32 {
            let header = PciCommonHeader::new(PciAddress::new(bus, device, 0, 0));
            if let Some((vendor_id, device_id)) = header.id() {
                let (has_multiple_functions, header_type) = header.header_type();
                println!("{bus}:{device}.0 => {vendor_id:x} {device_id:x} {header_type:?}");
                if has_multiple_functions {
                    for function in 1..8 {
                        let header =
                            PciCommonHeader::new(PciAddress::new(bus, device, function, 0));
                        if let Some((vendor_id, device_id)) = header.id() {
                            println!(
                                "  {bus}:{device}.{function} => {vendor_id:x} {device_id:x}"
                            );
                        }
                    }
                }
            }
        }
    }
}

fn pci_read(address: &PciAddress, offset: u32) -> u32 {
    let address: u32 = address.0 + offset;
    unsafe {
        CONFIG_ADDRESS_PORT.lock().write(address);
    }
    unsafe { CONFIG_DATA_PORT.lock().read() }
}
