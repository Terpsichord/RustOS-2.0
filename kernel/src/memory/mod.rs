// TODO: Re-write this as your own

use crate::memory::frame_allocator::PageFrameAllocator;
use anyhow::Result;
use conquer_once::spin::OnceCell;
use spin::{mutex::Mutex, MutexGuard};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        mapper::{MapToError, TranslateResult},
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags as Flags,
        PhysFrame, Size4KiB, Translate,
    },
    VirtAddr,
};

pub mod frame_allocator;
pub mod heap;

const PAGE_SIZE: usize = 4096;

static MANAGER: OnceCell<Mutex<Manager<'_, PageFrameAllocator>>> = OnceCell::uninit();

pub struct Manager<'a, A: FrameAllocator<Size4KiB>> {
    pub(crate) offset_page_table: OffsetPageTable<'a>,
    frame_allocator: A,
}

pub fn manager() -> MutexGuard<'static, Manager<'static, PageFrameAllocator>> {
    let manager = MANAGER
        .try_get()
        .expect("memory manager not initialised")
        .try_lock()
        .expect("couldn't locked manager");
    manager
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
            Mutex::new(Manager {
                offset_page_table,
                frame_allocator,
            })
        })
        .expect("memory::init should only be called once");
}

impl<'a> Manager<'a, PageFrameAllocator> {
    pub fn new(page_table: &'a mut PageTable) -> Self {
        let manager = manager();
        unsafe {
            Manager {
                offset_page_table: OffsetPageTable::new(
                    page_table,
                    manager.offset_page_table.phys_offset(),
                ),
                frame_allocator: manager.frame_allocator.clone(),
            }
        }
    }

    pub unsafe fn create_mapping(&mut self, frame: PhysFrame) -> Page {
        let page = Page::from_start_address(
            self.offset_page_table.phys_offset() + frame.start_address().as_u64(),
        )
        .expect("couldn't translate frame start address");
        unsafe {
            self.create_mapping_with_page(frame, page, Flags::PRESENT | Flags::WRITABLE);
        }
        page
    }

    pub unsafe fn create_mapping_with_page(&mut self, frame: PhysFrame, page: Page, flags: Flags) {
        let map_to_result = unsafe {
            self.offset_page_table
                .map_to(page, frame, flags, &mut self.frame_allocator)
        };

        map_to_result.expect("map_to failed").flush();
    }

    pub unsafe fn allocate_range(
        &mut self,
        page_range: impl Iterator<Item = Page>,
    ) -> Result<(), MapToError<Size4KiB>> {
        for page in page_range {
            let frame = self
                .frame_allocator
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;

            unsafe {
                self.create_mapping_with_page(frame, page, Flags::PRESENT | Flags::WRITABLE);
            }
        }

        Ok(())
    }

    pub fn translate(&mut self, virt_addr: VirtAddr) -> TranslateResult {
        self.offset_page_table.translate(virt_addr)
    }

    pub fn allocate_frame(&mut self) -> Option<PhysFrame> { self.frame_allocator.allocate_frame() }

    pub fn phys_offset(&self) -> VirtAddr { self.offset_page_table.phys_offset() }
}

/// Returns a mutable reference to the active level 4 table.
///
/// # Safety
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let phys_addr = level_4_table_frame.start_address();
    let virt_addr = physical_memory_offset + phys_addr.as_u64();
    let page_table_ptr = virt_addr.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}
