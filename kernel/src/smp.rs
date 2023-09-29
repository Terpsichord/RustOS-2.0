use crate::{apic::lapic, hlt_loop, println};
use bootloader_api::info::MemoryRegion;
use core::ptr;
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

const TRAMPOLINE_BASE: u64 = 0x78000;

pub unsafe fn init() {
    unsafe {
        let mut manager = crate::mem::MANAGER.try_get().unwrap().lock();
        let region: &MemoryRegion = manager
            .frame_allocator
            .memory_regions
            .iter()
            .find(|region| region.start <= TRAMPOLINE_BASE && TRAMPOLINE_BASE <= region.end)
            .unwrap();
        println!("{:?}", region);
        manager.create_mapping_with_page(
            PhysFrame::containing_address(PhysAddr::new(TRAMPOLINE_BASE)),
            Page::containing_address(VirtAddr::new(TRAMPOLINE_BASE)),
        );
        // manager.frame_allocator.memory_regions
        // let offset = manager.offset_page_table.phys_offset().as_u64();

        ptr::copy_nonoverlapping(
            _ap_trampoline as *const u8,
            TRAMPOLINE_BASE as *const u8,
            4096,
        );
        println!(
            "ap_start deref: {:#06x}\n0x78000: {:#06x}",
            *(ap_start as *const u64),
            *(TRAMPOLINE_BASE as *const u64),
        );

        let mut lapic = lapic::get();
        lapic.send_init_ipi_all();
        lapic.send_sipi_all((TRAMPOLINE_BASE >> 12) as u8);
    }
}

extern "C" fn ap_start() -> ! {
    // let vendor_info = CpuId::new().get_vendor_info();
    println!("hello from ap {:?}", 0 /* vendor_info */,);
    hlt_loop();
}
