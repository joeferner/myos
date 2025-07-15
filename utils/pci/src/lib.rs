#![no_std]

use bit_field::BitField;

pub trait PciConfigPort {
    fn read(&self, address: &PciAddress, offset: u32) -> u32;
}

/// The address of a PCIe function.
///
/// PCIe supports 65536 segments, each with 256 buses, each with 32 devices, each with 8 possible functions.:
///
/// ```ignore
/// 32                              16               8         3      0
///  +-------------------------------+---------------+---------+------+
///  |            segment            |      bus      | device  | func |
///  +-------------------------------+---------------+---------+------+
/// ```
pub struct PciAddress(u32);

impl PciAddress {
    pub fn new(bus: u8, device: u8, func: u8, offset: u8) -> Self {
        let bus: u32 = bus.into();
        let device: u32 = device.into();
        let func: u32 = func.into();
        let offset: u32 = offset.into();
        let address: u32 =
            (bus << 16) | (device << 11) | (func << 8) | (offset & 0xfc) | 0x80000000;
        Self(address)
    }

    pub fn address(&self) -> u32 {
        self.0
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
pub struct PciCommonHeader(PciAddress);

impl PciCommonHeader {
    pub const fn new(address: PciAddress) -> Self {
        Self(address)
    }

    pub fn id<T: PciConfigPort>(&self, port: &T) -> Option<(VendorId, DeviceId)> {
        let data = port.read(&self.0, 0x00);
        if data == 0xffff_ffff {
            return None;
        }
        let vendor_id = data.get_bits(0..16) as VendorId;
        let device_id = data.get_bits(16..32) as DeviceId;
        Some((vendor_id, device_id))
    }

    pub fn header_type<T: PciConfigPort>(&self, port: &T) -> (HasMultipleFunctions, HeaderType) {
        let data = port.read(&self.0, 0x0c);
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
