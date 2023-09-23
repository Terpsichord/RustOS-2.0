use acpi::{platform::interrupt::Apic, AcpiHandler, AcpiTables, InterruptModel, PhysicalMapping};
use alloc::alloc::Global;
use core::ptr::NonNull;

#[derive(Clone, Copy)]
pub struct Handler {
    phys_memory_offset: u64,
}

impl Handler {
    pub fn new(phys_memory_offset: u64) -> Self { Self { phys_memory_offset } }
}

impl AcpiHandler for Handler {
    unsafe fn map_physical_region<T>(
        &self,
        phys_addr: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let virt_addr = self.phys_memory_offset as usize + phys_addr;

        unsafe {
            PhysicalMapping::new(
                phys_addr,
                NonNull::new(virt_addr as *mut T).unwrap(),
                size,
                size,
                *self,
            )
        }
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}

/// Initialises the ACPI tables.
///
/// # Safety
/// This function is unsafe because the caller must ensure that `rsdp_addr` is a
/// valid RSDP address.
pub unsafe fn init<'a>(phys_memory_offset: u64, rsdp_addr: u64) -> AcpiTables<Handler> {
    let handler = Handler::new(phys_memory_offset);
    unsafe { AcpiTables::from_rsdp(handler, rsdp_addr as usize).unwrap() }
}

/// Gets info about the APIC from the ACPI tables.
pub fn get_apic_info(acpi_tables: &AcpiTables<Handler>) -> Apic<'_, Global> {
    let InterruptModel::Apic(apic_info) = acpi_tables.platform_info().unwrap().interrupt_model
    else {
        panic!("unknown interrupt model (not apic)")
    };

    apic_info
}
