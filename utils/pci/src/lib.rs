#![no_std]

use core::fmt::Debug;

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
pub type SubClassCode = u8;
pub type ProgIF = u8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HeaderType {
    Endpoint,
    PciPciBridge,
    CardBusBridge,
    Unknown(u8),
}

impl From<u32> for HeaderType {
    fn from(value: u32) -> Self {
        match value {
            0x00 => HeaderType::Endpoint,
            0x01 => HeaderType::PciPciBridge,
            0x02 => HeaderType::CardBusBridge,
            v => HeaderType::Unknown(v as u8),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClassCode {
    OldDevice,
    MassStorageController,
    NetworkController,
    DisplayController,
    MultimediaDevice,
    MemoryController,
    BridgeDevice,
    SimpleCommunicationsController,
    BaseSystemPeripheral,
    InputDevice,
    DockingStation,
    Processor,
    SerialBusController,
    Reserved(u8),
    Misc,
}

impl From<u32> for ClassCode {
    fn from(value: u32) -> Self {
        match value {
            0x00 => ClassCode::OldDevice,
            0x01 => ClassCode::MassStorageController,
            0x02 => ClassCode::NetworkController,
            0x03 => ClassCode::DisplayController,
            0x04 => ClassCode::MultimediaDevice,
            0x05 => ClassCode::MemoryController,
            0x06 => ClassCode::BridgeDevice,
            0x07 => ClassCode::SimpleCommunicationsController,
            0x08 => ClassCode::BaseSystemPeripheral,
            0x09 => ClassCode::InputDevice,
            0x0a => ClassCode::DockingStation,
            0x0b => ClassCode::Processor,
            0x0c => ClassCode::SerialBusController,
            0xff => ClassCode::Misc,
            v => ClassCode::Reserved(v as u8),
        }
    }
}

/// ```ignore
///       31          24 23         16 15          8 7                0
///       +-------------+-------------+-------------+-----------------+
/// 0x00  |        Device ID          |         Vendor ID             |
/// 0x04  |          Status           |          Command              |
/// 0x08  |  Class Code |  Sub-Class  |   Prog IF   |  Revision ID    |
/// 0x0c  |     0x00    | Header Type |    0x00     | Cache Line Size |
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

    pub fn class_code<T: PciConfigPort>(&self, port: &T) -> (ClassCode, SubClassCode) {
        let data = port.read(&self.0, 0x08);
        let class_code: ClassCode = data.get_bits(24..32).into();
        let sub_class_code: SubClassCode = data.get_bits(16..24) as SubClassCode;
        (class_code, sub_class_code)
    }

    pub fn prog_if<T: PciConfigPort>(&self, port: &T) -> ProgIF {
        let data = port.read(&self.0, 0x08);
        data.get_bits(8..16) as ProgIF
    }

    pub fn header_type<T: PciConfigPort>(&self, port: &T) -> HeaderType {
        let data = port.read(&self.0, 0x0c);
        // don't include high bit, that indicates if device has multiple functions
        data.get_bits(16..23).into()
    }

    pub fn has_multiple_functions<T: PciConfigPort>(&self, port: &T) -> bool {
        let data = port.read(&self.0, 0x0c);
        data.get_bit(23)
    }
}
