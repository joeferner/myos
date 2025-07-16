#![no_std]

use core::fmt::Debug;

use spin::Mutex;

use crate::{
    types::{DeviceId, PciAddress, PciCommonHeader, PciConfigPort, VendorId},
    x86::{PCI_CONFIG_PORT, X86PciConfigPort},
};

mod types;
mod x86;

pub static PCI_DRIVER: PciDriver<X86PciConfigPort> = PciDriver::new(&PCI_CONFIG_PORT);

pub struct PciDriver<'a, T: PciConfigPort> {
    config_port: &'a Mutex<T>,
}

impl<'a, T: PciConfigPort> PciDriver<'a, T> {
    pub const fn new(config_port: &'a Mutex<T>) -> Self {
        Self { config_port }
    }

    pub fn iterate_devices(&self) -> PciDeviceIterator<'a, T> {
        PciDeviceIterator::new(&self.config_port)
    }
}

pub struct PciDeviceIterator<'a, T: PciConfigPort> {
    config_port: &'a Mutex<T>,
    has_next: bool,
    next_bus: u8,
    next_device: u8,
    next_func: u8,
}

impl<'a, T: PciConfigPort> PciDeviceIterator<'a, T> {
    fn new(config_port: &'a Mutex<T>) -> Self {
        Self {
            config_port,
            has_next: true,
            next_bus: 0,
            next_device: 0,
            next_func: 0,
        }
    }

    fn update_next(&mut self) {
        if self.next_device == 31 {
            self.next_device = 0;
            if self.next_bus == 254 {
                self.has_next = false;
            }
            self.next_bus += 1;
        } else {
            self.next_device += 1;
        }
    }
}

impl<'a, T: PciConfigPort> Iterator for PciDeviceIterator<'a, T> {
    type Item = PciDevice<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        // check early to avoid taking a lock
        if !self.has_next {
            return None;
        }

        let port = PCI_CONFIG_PORT.lock();
        while self.has_next {
            let addr = PciAddress::new(self.next_bus, self.next_device, self.next_func, 0);
            let header = PciCommonHeader::new(addr);

            if header.has_multiple_functions(&*port) || self.next_func > 0 {
                if self.next_func == 7 {
                    self.next_func = 0;
                    self.update_next();
                } else {
                    self.next_func += 1;
                }
            } else {
                self.update_next();
            }

            let id = header.id(&*port);
            if let Some(id) = id {
                return Some(PciDevice::new(self.config_port, addr, id.0, id.1, header));
            }
        }
        return None;
    }
}

pub struct PciDevice<'a, T: PciConfigPort> {
    _config_port: &'a Mutex<T>,
    pub addr: PciAddress,
    pub vendor_id: VendorId,
    pub device_id: DeviceId,
    _header: PciCommonHeader,
}

impl<'a, T: PciConfigPort> PciDevice<'a, T> {
    fn new(
        config_port: &'a Mutex<T>,
        addr: PciAddress,
        vendor_id: VendorId,
        device_id: DeviceId,
        header: PciCommonHeader,
    ) -> Self {
        Self {
            _config_port: config_port,
            addr,
            vendor_id,
            device_id,
            _header: header,
        }
    }
}

impl<'a, T: PciConfigPort> Debug for PciDevice<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PciDevice")
            .field("addr", &self.addr)
            .field("vendor_id", &format_args!("0x{:04x}", self.vendor_id))
            .field("device_id", &format_args!("0x{:04x}", self.device_id))
            .finish()
    }
}
