use pci::{PciAddress, PciCommonHeader};

use crate::{pci::x86::PCI_CONFIG_PORT, println};

mod x86;

pub fn pci_enumerate() {
    let port = PCI_CONFIG_PORT.lock();

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
