use core::ptr::addr_of;
use spin::Lazy;
use tss::TaskStateSegment;
use x86_64::{
    instructions::{segmentation::Segment, tables::load_tss},
    registers::segmentation::{CS, DS, SS},
    structures::{
        gdt::{Descriptor, DescriptorFlags, GlobalDescriptorTable, SegmentSelector},
        tss,
    },
    PrivilegeLevel, VirtAddr,
};

#[allow(clippy::enum_variant_names)]
#[repr(u16)]
pub enum IstIndex {
    DoubleFault = 1,
    PageFault,
    GeneralProtectionFault,
}

const STACK_SIZE: usize = 4096 * 5;

static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();

    tss.interrupt_stack_table[IstIndex::DoubleFault as usize] = create_stack!();
    tss.interrupt_stack_table[IstIndex::PageFault as usize] = create_stack!();
    tss.interrupt_stack_table[IstIndex::GeneralProtectionFault as usize] = create_stack!();
    tss.privilege_stack_table[0] = create_stack!();

    let dfs = tss.interrupt_stack_table[IstIndex::DoubleFault as usize];
    let ps = tss.privilege_stack_table[0];

    debug_assert_ne!(dfs, ps);

    tss
});

macro create_stack() {{
    static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
    let stack_start = VirtAddr::from_ptr(unsafe { addr_of!(STACK) });
    stack_start + STACK_SIZE
}}

pub static GDT: Lazy<(GlobalDescriptorTable, Selectors)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();

    let kernel_data_flags =
        DescriptorFlags::USER_SEGMENT | DescriptorFlags::PRESENT | DescriptorFlags::WRITABLE;
    let kernel_code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    let kernel_data_selector = gdt.add_entry(Descriptor::UserSegment(kernel_data_flags.bits()));

    let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

    let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());
    let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());

    let selectors = Selectors {
        kernel_code_selector,
        kernel_data_selector,
        user_code_selector,
        user_data_selector,
        tss_selector,
    };

    (gdt, selectors)
});

pub struct Selectors {
    pub kernel_code_selector: SegmentSelector,
    pub kernel_data_selector: SegmentSelector,
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
    pub tss_selector: SegmentSelector,
}

pub fn init() {
    let (gdt, selectors) = &*GDT;
    gdt.load();

    // unsafe {
    //     log::debug!(
    //     "Loaded GDT: {:p}\nTSS: {:p}\nStack {:p}\nUser stack {:p}\nCS segment:
    // {:?}\nTSS segment {:?}",     gdt as *const _,
    //     &*TSS as *const _,
    //     &STACK as *const _,
    //     &PRIVILEGE_STACK as *const _,
    //     selectors.kernel_code_selector,
    //     selectors.tss_selector
    // );
    // }

    unsafe {
        CS::set_reg(selectors.kernel_code_selector);
        SS::set_reg(selectors.kernel_data_selector);
        load_tss(selectors.tss_selector);
    }
}

pub fn set_usermode_segs() -> (u16, u16) {
    let (gdt, selectors) = &*GDT;
    // let (mut cs, mut ds) = (selectors.user_code_selector,
    // selectors.user_data_selector);
    let (mut cs, mut ds) = (
        selectors.kernel_code_selector,
        selectors.kernel_data_selector,
    );
    log::debug!("{cs:?} {ds:?}");

    cs.set_rpl(PrivilegeLevel::Ring3);
    ds.set_rpl(PrivilegeLevel::Ring3);

    unsafe {
        DS::set_reg(ds);
    }

    let indexes = (cs.0, ds.0);
    log::debug!("{indexes:?}");
    log::debug!("{:?}", gdt.as_raw_slice()[0]);
    indexes
}
