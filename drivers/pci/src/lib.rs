#![no_std]

use crate::{
    types::{PciAddress, PciCommonHeader},
    x86::PCI_CONFIG_PORT,
};

mod types;
mod x86;

pub struct PciDriver {}

impl PciDriver {
    pub fn iterate_devices(&self) -> PciDeviceIterator {
        PciDeviceIterator::new()
    }
}

pub struct PciDeviceIterator {
    has_next: bool,
    next_bus: u8,
    next_device: u8,
    next_func: u8,
}

impl PciDeviceIterator {
    fn new() -> Self {
        Self {
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

impl Iterator for PciDeviceIterator {
    type Item = PciDevice;

    fn next(&mut self) -> Option<Self::Item> {
        // check early to avoid taking a lock
        if !self.has_next {
            return None;
        }

        let port = PCI_CONFIG_PORT.lock();
        while self.has_next {
            let addr = PciAddress::new(self.next_bus, self.next_device, self.next_func, 0);
            let header = PciCommonHeader::new(addr);

            if header.has_multiple_functions(&*port) {
                if self.next_func == 7 {
                    self.next_func = 0;
                    self.update_next();
                } else {
                    self.next_func += 1;
                }
            } else {
                self.update_next();
            }

            if header.id(&*port).is_some() {
                return Some(PciDevice::new(addr));
            }
        }
        return None;
    }
}

pub struct PciDevice {
    addr: PciAddress,
}

impl PciDevice {
    fn new(addr: PciAddress) -> Self {
        Self { addr }
    }

    pub fn id(&self) -> PciCommonHeader {

    }
}
