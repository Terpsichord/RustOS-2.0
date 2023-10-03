use crate::mem;
use linked_list_allocator::LockedHeap;
use x86_64::{
    structures::paging::{mapper::MapToError, Page, Size4KiB},
    VirtAddr,
};

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

// TODO: Write your own heap allocator

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Initialises the heap allocator.
pub fn init() -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;

        Page::range_inclusive(
            Page::containing_address(heap_start),
            Page::containing_address(heap_end),
        )
    };

    unsafe {
        mem::allocate_range(page_range)?;

        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }

    Ok(())
}
