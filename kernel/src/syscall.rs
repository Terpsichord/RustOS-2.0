use alloc::vec::Vec;
use core::arch::asm;

use x86_64::{
    instructions::{interrupts, tlb},
    PhysAddr,
    registers::{
        control::{Cr3, Cr3Flags, Efer, EferFlags},
        model_specific::Star,
        rflags,
        rflags::RFlags,
    },
    structures::paging::{
        mapper::{MappedFrame, TranslateResult::Mapped},
        Page,
        page_table::PageTableEntry, PageTable, PageTableFlags, PhysFrame,
    }, VirtAddr,
};
use x86_64::structures::paging::OffsetPageTable;

use crate::{
    gdt, memory,
    memory::{active_level_4_table, Manager},
    user,
};

const USER_PROGRAM_BASE: u64 = 0x40_0000;
const STACK_BASE: u64 = 0x80_0000;

pub fn init_msr() {
    // let (_, selectors) = &*gdt::GDT;
    unsafe {
        // Star::write(
        //     selectors.user_code_selector,
        //     selectors.user_data_selector,
        //     selectors.user_code_selector,
        //     selectors.user_data_selector,
        // )
        // .unwrap_or_else(|e| panic!("{e}"));

        Star::write_raw(0x23, 0x8);
        Efer::write(Efer::read() | EferFlags::SYSTEM_CALL_EXTENSIONS);
        log::debug!("star: {:x?}", Star::read());
    }
}

#[allow(clippy::similar_names)]
pub unsafe fn jump_to_usermode(code: VirtAddr, stack_end: VirtAddr) {
    let (cs_idx, ds_idx) = gdt::set_usermode_segs();
    // log::debug!("indexes: {cs_idx:b}, {ds_idx:b}");
    tlb::flush_all();
    interrupts::disable();
    unsafe {
        asm!("\
            push rax  // stack segment
            push rsi  // rsp
            push 0x200 // rflags (only interrupt bit set)
            push rdx  // code segment
            push rdi  // ret to virtual addr
            iretq
            ",
        in("ax") ds_idx,                        // ss
        in("rsi") stack_end.as_u64(),            // rsp
        in("rcx") (rflags::read() | RFlags::INTERRUPT_FLAG).bits(), // rflags
        in("rdi") code.as_u64(),                 // rip
        in("dx") cs_idx,                        // cs
        );
    }
}

fn init_page_table(/*page_table: &mut PageTable*/) -> PageTable {
    let mut page_table = PageTable::new();
    page_table[0].set_addr(
        memory::manager()
            .allocate_frame()
            .expect("couldn't allocate frame")
            .start_address(),
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE,
    );

    let child_pt: &mut PageTable = unsafe { next_page_table(&page_table[0]) };

    let current_pt = unsafe { active_level_4_table(memory::manager().phys_offset()) };
    let current_child_pt = unsafe { next_page_table(&current_pt[0]) };

    for i in 3..=6 {
        child_pt[i] = current_child_pt[i].clone();
    }
    page_table
}

pub fn init() {
    let user_prog_addr = VirtAddr::new(user::userpace_program as *const () as u64);

    let Mapped {
        frame: MappedFrame::Size4KiB(frame),
        offset,
        ..
    } = memory::manager().translate(user_prog_addr)
    else {
        panic!("user program not mapped")
    };
    let userspace_fn_virt = VirtAddr::new(USER_PROGRAM_BASE + offset);

    // let mut pt = PageTable::new();
    let mut page_table = init_page_table();

    {
        let mut manager = Manager::new(&mut page_table);

        log::trace!(
            "mapping program {:x} to {:x}",
            USER_PROGRAM_BASE,
            frame.start_address()
        );
        unsafe {
            manager.create_mapping_with_page(
                frame,
                Page::from_start_address_unchecked(VirtAddr::new(USER_PROGRAM_BASE)),
                PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE,
            );
            log::trace!("mapping stack");
            let offset = 0x1000;
            manager.create_mapping_with_page(
                frame + offset,
                Page::from_start_address_unchecked(VirtAddr::new(USER_PROGRAM_BASE) + offset),
                PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE,
            );
            log::trace!("mapped");
        }

        log::trace!("translating");

        let mut stack_space: Vec<u8> = Vec::with_capacity(0x1000);
        let Mapped {
            frame: MappedFrame::Size4KiB(stack_space_phys),
            ..
        } = memory::manager().translate(VirtAddr::new(stack_space.as_mut_ptr() as u64))
        else {
            panic!("failed to translate stack_space")
        };
        log::trace!("translated");

        unsafe {
            manager.create_mapping_with_page(
                stack_space_phys,
                Page::from_start_address_unchecked(VirtAddr::new(STACK_BASE)),
                PageTableFlags::PRESENT
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::USER_ACCESSIBLE,
            );
        }
        log::trace!("map stack");

        // unsafe {
        //     let frame =
        // PhysFrame::<Size4KiB>::containing_address(pt0entry.addr());
        //     Cr3::write(
        //         frame,
        //         Cr3Flags::empty(),
        //     );
        // }
    }

    // for (index, entry) in page_table.iter().enumerate() {
    //     if !entry.is_unused() {
    //         log::debug!("{index}: {entry:?}");
    //         let old_entry =
    // memory::manager().offset_page_table.level_4_table()[index].clone();
    //         log::debug!("{:?}", old_entry);
    //         unsafe {
    //             for (index, entry) in next_page_table(&entry).iter().enumerate()
    // {                 if !entry.is_unused() {
    //                     log::debug!("{index}: {entry:?}");
    //                     let old_entry = &next_page_table(&old_entry)[index];
    //                     log::debug!("{:?}", old_entry);
    //                 }
    //             }
    //         }
    //     }
    // }
    log::debug!("page table {:?}", page_table_phys(&page_table));
    log::trace!("left page table");

    let table = memory::manager().offset_page_table.level_4_table().clone();
    log::debug!("got table");
    unsafe {
        Cr3::write(
            PhysFrame::from_start_address(page_table_phys(&table))
                .expect("3rd level page table not page aligned"),
            Cr3Flags::empty(),
        );
    }

    log::trace!("msr");

    init_msr();

    log::trace!("userspace_fn {:p}", userspace_fn_virt);

    unsafe {
        jump_to_usermode(userspace_fn_virt, VirtAddr::new(STACK_BASE + 0x1000));
    }
    log::trace!("jump usermode");
}

unsafe fn next_page_table(entry: &PageTableEntry) -> &'static mut PageTable {
    unsafe { &mut *(memory::manager().phys_offset() + entry.addr().as_u64()).as_mut_ptr() }
}

fn page_table_phys(pt: &PageTable) -> PhysAddr {
    let virt = VirtAddr::from_ptr(pt as *const _);
    let Mapped {
        frame: MappedFrame::Size4KiB(frame),
        offset,
        ..
    } = memory::manager().translate(virt)
    else {
        panic!("couldn't translate virt addr");
    };
    frame.start_address() + offset
}

// struct Registers {
//     r11: u64,
//     r10: u64,
//     r9: u64,
//     r8: u64,
//     rsi: u64,
//     rdi: u64,
//     rdx: u64,
//     rcx: u64,
//     rax: u64,
// }
//
// macro_rules! push_context {
//     () => {
//         asm!("\
//             push rax
//             push rcx
//             push rdx
//             push rdi
//             push rsi
//             push r8
//             push r9
//             push r10
//             push r11
//         ")
//     };
// }


// Jump to usermode based on the Process::exec function in MOROS (https://github.com/vinc/moros/blob/trunk/src/sys/process.rs#L426)
pub fn moros_jump_user_mode() {
    let page_table = {
        // sys::process::page_table:
        let mut level_4_table = unsafe { memory::manager().active_level_4_table() };
        let phys_mem_offset = memory::manager().offset_page_table.phys_offset();
        let mapper = unsafe { OffsetPageTable::new(&mut level_4_table, phys_mem_offset) };
        // let frame_allocator
    }
}