// TODO: Re-write this as your own

use crate::mem::frame_allocator::BootInfoFrameAllocator;
use conquer_once::spin::OnceCell;
use spin::mutex::Mutex;
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame,
        Size4KiB,
    },
    PhysAddr, VirtAddr,
};

pub mod frame_allocator;
pub mod heap;

pub static MANAGER: OnceCell<Mutex<MemoryManager<BootInfoFrameAllocator>>> = OnceCell::uninit();

pub struct MemoryManager<A: FrameAllocator<Size4KiB>> {
    pub offset_page_table: OffsetPageTable<'static>,
    frame_allocator: A,
}

/// Initialise a new OffsetPageTable.
///
/// # Safety
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn init(physical_memory_offset: VirtAddr, frame_allocator: BootInfoFrameAllocator) {
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

/// Returns a mutable reference to the active level 4 table.
///
/// # Safety
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}

impl<A: FrameAllocator<Size4KiB>> MemoryManager<A> {
    pub fn phys_addr_to_virt(&self, phys_addr: PhysAddr) -> VirtAddr {
        self.offset_page_table.phys_offset() + phys_addr.as_u64()
    }

    pub unsafe fn create_mapping(&mut self, frame: PhysFrame) -> Page {
        let page = Page::from_start_address(self.phys_addr_to_virt(frame.start_address()))
            .expect("couldn't translate frame start address");
        unsafe {
            self.create_mapping_with_page(frame, page);
        }
        page
    }

    pub unsafe fn create_mapping_with_page(&mut self, frame: PhysFrame, page: Page) {
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        let map_to_result = unsafe {
            self.offset_page_table
                .map_to(page, frame, flags, &mut self.frame_allocator)
        };
        map_to_result.expect("map_to failed").flush();
    }

    pub fn allocate_frame(&mut self) -> Option<PhysFrame> { self.frame_allocator.allocate_frame() }
}
