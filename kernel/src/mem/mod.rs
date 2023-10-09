// TODO: Re-write this as your own

use crate::mem::frame_allocator::PageFrameAllocator;
use anyhow::Result;
use conquer_once::spin::OnceCell;
use spin::{mutex::Mutex, MutexGuard};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, OffsetPageTable, Page, PageTable,
        PageTableFlags, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

pub mod frame_allocator;
pub mod heap;

const PAGE_SIZE: usize = 4096;

static MANAGER: OnceCell<Mutex<MemoryManager<PageFrameAllocator>>> = OnceCell::uninit();

struct MemoryManager<A: FrameAllocator<Size4KiB>> {
    offset_page_table: OffsetPageTable<'static>,
    frame_allocator: A,
}

fn memory_manager() -> MutexGuard<'static, MemoryManager<PageFrameAllocator>> {
    let result = MANAGER.try_get();
    let mutex = result.expect("memory manager not initialised");
    let guard = mutex.lock();
    guard
}

/// Initialise a new `OffsetPageTable`.
///
/// # Safety
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn init(physical_memory_offset: VirtAddr, frame_allocator: PageFrameAllocator) {
    let level_4_table = unsafe { active_level_4_table(physical_memory_offset) };
    let offset_page_table = unsafe { OffsetPageTable::new(level_4_table, physical_memory_offset) };

    MANAGER
        .try_init_once(|| {
            Mutex::new(MemoryManager {
                offset_page_table,
                frame_allocator,
            })
        })
        .expect("mem::init should only be called once");
}

pub unsafe fn create_mapping(frame: PhysFrame) -> Page {
    let page = Page::from_start_address(translate(frame.start_address()))
        .expect("couldn't translate frame start address");
    unsafe {
        create_mapping_with_page(frame, page);
    }
    page
}

pub unsafe fn create_mapping_with_page(frame: PhysFrame, page: Page) {
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    let mut guard = memory_manager();
    let manager = &mut *guard;

    let map_to_result = unsafe {
        manager
            .offset_page_table
            .map_to(page, frame, flags, &mut manager.frame_allocator)
    };

    map_to_result.expect("map_to failed").flush();
}

pub unsafe fn allocate_range(
    page_range: impl Iterator<Item = Page>,
) -> Result<(), MapToError<Size4KiB>> {
    for page in page_range {
        let frame = memory_manager()
            .frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        unsafe {
            create_mapping_with_page(frame, page);
        }
    }

    Ok(())
}

pub fn translate(phys_addr: PhysAddr) -> VirtAddr {
    memory_manager().offset_page_table.phys_offset() + phys_addr.as_u64()
}

pub fn allocate_frame() -> Option<PhysFrame> { memory_manager().frame_allocator.allocate_frame() }

/// Returns a mutable reference to the active level 4 table.
///
/// # Safety
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let phys_addr = level_4_table_frame.start_address();
    let virt_addr = physical_memory_offset + phys_addr.as_u64();
    let page_table_ptr = virt_addr.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}
