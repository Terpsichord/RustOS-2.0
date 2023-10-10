use crate::{device, mem};
use tinypci::{PciClass, PciDeviceInfo};
use x86_64::{structures::paging::PhysFrame, PhysAddr};

pub fn init(pci_devices: &[PciDeviceInfo]) {
    let hba_info = pci_devices
        .iter()
        .find(|dev| dev.class() == PciClass::MassStorage)
        .expect("couldn't find HBA");

    let hba_mem_addr = PhysAddr::new(hba_info.bars[5] as u64);
    let hba_mem = unsafe {
        mem::create_mapping(
            PhysFrame::from_start_address(hba_mem_addr).expect("unaligned HBA address"),
        )
    };
}

pub struct Driver;

impl device::Driver for Driver {
    fn class(&self) -> PciClass { PciClass::MassStorage }
}
