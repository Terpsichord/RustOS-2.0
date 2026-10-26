use crate::{acpi::Handler, apic::lapic, hlt_loop, mem, println, serial::serial_println};
use acpi::{
    madt::{Madt, MadtEntry},
    AcpiResult, AcpiTables,
};
use bootloader_api::info::MemoryRegion;
use core::ptr;
use raw_cpuid::CpuId;
use x86_64::{
    structures::paging::{Page, PhysFrame},
    PhysAddr, VirtAddr,
};

// pub unsafe fn example_read_and_write_to_msr() {
//     let mut msr = Msr::new(0x1b);
//
//     unsafe {
//         let msr_val = msr.read();
//         println!("read msr: {:x}", msr_val);
//
//         let to_write = msr_val | (1 << 11);
//         println!("writing {:x}", to_write);
//         msr.write(to_write);
//
//         println!("reread msr: {:x}", msr.read());
//
//         serial_println!("msr value: {:x}", msr_val);
//     }
// }

// pub fn ipi(id: u32, mut num: u32) {
//     unsafe {
//         let lapic = crate::lapic::get();
//         println!("num 0x{:x}", num);
//         let mut id_shifted = id << 24;
//         println!("id {} storing {:b}", id, id_shifted);
//         core::ptr::write_volatile(0xfee00310 as *mut u32, id_shifted);
//         //lapic.icr_high.store(&mut id_shifted as *mut u32,
// Ordering::SeqCst);         //let x = lapic.icr_high.load(Ordering::SeqCst);
//         //println!("stored {:b}", unsafe{*x});
//         println!("sending ipi {:b}", num);
//         //lapic.icr_low.store(&mut num, Ordering::SeqCst);
//         core::ptr::write_volatile(0xfee00300 as *mut u32, num);
//     }
// }

const AP_STARTUP: usize = 0x10000;
const PAGE_SIZE: usize = 1 << 12;
const TRAMPOLINE: usize = AP_STARTUP - PAGE_SIZE;

// pub unsafe fn init() {
//     unsafe {
//         let mut manager = crate::mem::MANAGER.try_get().unwrap().lock();
//         let region: &MemoryRegion = manager
//             .frame_allocator
//             .memory_regions
//             .iter()
//             .find(|region| region.start <= TRAMPOLINE_BASE && TRAMPOLINE_BASE
// <= region.end)             .unwrap();
//         println!("{:?}", region);
//         manager.create_mapping_with_page(
//             PhysFrame::containing_address(PhysAddr::new(TRAMPOLINE_BASE)),
//             Page::containing_address(VirtAddr::new(TRAMPOLINE_BASE)),
//         );
//         // manager.frame_allocator.memory_regions
//         // let offset = manager.offset_page_table.phys_offset().as_u64();
//
//         ptr::copy_nonoverlapping(
//             _ap_trampoline as *const u8,
//             TRAMPOLINE_BASE as *const u8,
//             4096,
//         );
//         println!(
//             "ap_start deref: {:#06x}\n0x78000: {:#06x}",
//             *(ap_start as *const u64),
//             *(TRAMPOLINE_BASE as *const u64),
//         );
//
//         let mut lapic = lapic::get();
//         lapic.send_init_ipi_all();
//         lapic.send_sipi_all((TRAMPOLINE_BASE >> 12) as u8);
//     }
// }

pub fn startup_aps(acpi_tables: &AcpiTables<Handler>) {
    let id = unsafe { lapic::get().id() };
    if let Ok(madt) = acpi_tables.find_table::<Madt>() {
        madt.entries()
            .filter_map(|entry| match entry {
                MadtEntry::LocalApic(lapic) => Some(lapic),
                _ => None,
            })
            .for_each(|lapic| {
                if lapic.flags & 1 == 1 {
                    log::debug!("lapic {} enabled", lapic.apic_id);
                    unsafe {
                        start_ap(lapic.apic_id as u32);
                    }
                }
            });
    }
    log::info!("current apic id: {:?}", id);
}

/// Starts the AP with the given `apic_id`.
///
/// # Panics
/// Panics if the memory manager has not been initialised.
unsafe fn start_ap(apic_id: u32) {
    let mut lapic = lapic::get();
    let mut mm = mem::MANAGER
        .get()
        .expect("failed to get memory manager")
        .lock();
    unsafe {
        serial_println!("creating mapping");
        mm.create_mapping_with_page(
            PhysFrame::from_start_address(PhysAddr::new(TRAMPOLINE as u64)).unwrap_or_else(|e| {
                serial_println!("{:?}", e);
                hlt_loop();
            }),
            Page::from_start_address(VirtAddr::new(TRAMPOLINE as u64)).unwrap_or_else(|e| {
                serial_println!("{:?}", e);
                hlt_loop();
            }),
        );
//         let mut manager = crate::mem::MANAGER.try_get().unwrap().lock();
//         let region: &MemoryRegion = manager
//             .frame_allocator
//             .memory_regions
//             .iter()
//             .find(|region| region.start <= TRAMPOLINE_BASE && TRAMPOLINE_BASE <= region.end)
//             .unwrap();
//         println!("{:?}", region);
//         manager.create_mapping_with_page(
//             PhysFrame::containing_address(PhysAddr::new(TRAMPOLINE_BASE)),
//             Page::containing_address(VirtAddr::new(TRAMPOLINE_BASE)),
//         );
//         // manager.frame_allocator.memory_regions
//         // let offset = manager.offset_page_table.phys_offset().as_u64();
//
//         ptr::copy_nonoverlapping(
//             _ap_trampoline as *const u8,
//             TRAMPOLINE_BASE as *const u8,
//             4096,
//         );
//         println!(
//             "ap_start deref: {:#06x}\n0x78000: {:#06x}",
//             *(ap_start as *const u64),
//             *(TRAMPOLINE_BASE as *const u64),
//         );
        serial_println!("sending init ipi to APIC{apic_id}");
        lapic.send_init_ipi(apic_id);
        serial_println!("sending sipi");
        lapic.send_sipi(0, apic_id);
    }
}

extern "C" fn ap_start() -> ! {
    // let vendor_info = CpuId::new().get_vendor_info();
    println!("hello from ap {:?}", 0 /* vendor_info */,);
    hlt_loop();
}
