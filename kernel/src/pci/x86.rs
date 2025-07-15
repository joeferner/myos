use pci::{PciAddress, PciConfigPort};
use spin::Mutex;
use x86_64::instructions::port::{PortGeneric, ReadWriteAccess};

pub static PCI_CONFIG_PORT: Mutex<X86PciConfigPort> = Mutex::new(X86PciConfigPort::new());

const PCI_CONFIG_ADDRESS: u16 = 0xcf8;
const PCI_CONFIG_DATA: u16 = 0xcfc;

pub struct X86PciConfigPort {
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
