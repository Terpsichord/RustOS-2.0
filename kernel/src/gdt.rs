use core::ops::Deref;
use spin::Lazy;
use tss::TaskStateSegment;
use x86_64::{
    instructions::{segmentation::Segment, tables::load_tss},
    registers::segmentation::{CS, SS},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss,
    },
    VirtAddr,
};

static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();

    tss.interrupt_stack_table = [{
        const STACK_SIZE: usize = 4096 * 5;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

        let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
        let stack_end = stack_start + STACK_SIZE;
        stack_end
    }; 7];
    tss
});

static GDT: Lazy<(GlobalDescriptorTable, Selectors)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();

    let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    let data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
    let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

    let selectors = Selectors {
        code_selector,
        data_selector,
        tss_selector,
    };

    (gdt, selectors)
});

struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    let (gdt, selectors) = GDT.deref();
    gdt.load();
    unsafe {
        CS::set_reg(selectors.code_selector);
        SS::set_reg(selectors.data_selector);
        load_tss(selectors.tss_selector);
    }
}
