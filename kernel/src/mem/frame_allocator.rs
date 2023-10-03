use crate::mem::PAGE_SIZE;
use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::{
    structures::paging::{FrameAllocator, PhysFrame, Size4KiB},
    PhysAddr,
};

/// A FrameAllocator that returns usable frames from the bootloader's memory
/// map.
pub struct PageFrameAllocator {
    memory_regions: &'static MemoryRegions,
    next: usize,
}

unsafe impl Send for PageFrameAllocator {}

impl PageFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the
    /// passed memory map is valid. The main requirement is that all frames
    /// that are marked as `USABLE` in it are really unused.
    pub unsafe fn init(memory_region: &'static MemoryRegions) -> Self {
        PageFrameAllocator {
            memory_regions: memory_region,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        self.memory_regions
            .iter()
            .filter(|r| r.kind == MemoryRegionKind::Usable)
            .map(|r| r.start..r.end)
            .flat_map(|r| r.step_by(PAGE_SIZE))
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for PageFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let mut frames = self.usable_frames();
        let frame = frames.nth(self.next);
        self.next += 1;
        frame
    }
}
