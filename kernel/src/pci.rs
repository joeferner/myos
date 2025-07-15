use pci::{PciAddress, PciCommonHeader, PciConfigPort};
use spin::Mutex;
use x86_64::instructions::port::{PortGeneric, ReadWriteAccess};

use crate::println;

const PCI_CONFIG_PORT: Mutex<X86PciConfigPort> = Mutex::new(X86PciConfigPort::new());

pub fn pci_enumerate() {
    let binding = PCI_CONFIG_PORT;
    let port = binding.lock();

    for bus in 0..=255 {
        for device in 0..32 {
            let header = PciCommonHeader::new(PciAddress::new(bus, device, 0, 0));
            if let Some((vendor_id, device_id)) = header.id(&*port) {
                let (has_multiple_functions, header_type) = header.header_type(&*port);
                println!("{bus}:{device}.0 => {vendor_id:x} {device_id:x} {header_type:?}");
                if has_multiple_functions {
                    for function in 1..8 {
                        let header =
                            PciCommonHeader::new(PciAddress::new(bus, device, function, 0));
                        if let Some((vendor_id, device_id)) = header.id(&*port) {
                            println!("  {bus}:{device}.{function} => {vendor_id:x} {device_id:x}");
                        }
                    }
                }
            }
        }
    }
}

const PCI_CONFIG_ADDRESS: u16 = 0xcf8;
const PCI_CONFIG_DATA: u16 = 0xcfc;

struct X86PciConfigPort {
    inner: Mutex<X86PciConfigPortInner>,
}

impl X86PciConfigPort {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(X86PciConfigPortInner::new()),
        }
    }
}

impl PciConfigPort for X86PciConfigPort {
    fn read(&self, address: &PciAddress, offset: u32) -> u32 {
        let mut inner = self.inner.lock();
        let address: u32 = address.address() + offset;
        unsafe {
            inner.address_port.write(address);
        }
        unsafe { inner.data_port.read() }
    }
}

struct X86PciConfigPortInner {
    address_port: PortGeneric<u32, ReadWriteAccess>,
    data_port: PortGeneric<u32, ReadWriteAccess>,
}

impl X86PciConfigPortInner {
    pub const fn new() -> Self {
        Self {
            address_port: PortGeneric::new(PCI_CONFIG_ADDRESS),
            data_port: PortGeneric::new(PCI_CONFIG_DATA),
        }
    }
}
