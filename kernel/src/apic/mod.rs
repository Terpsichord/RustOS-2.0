use acpi::platform::interrupt::Apic;
use alloc::alloc::Global;
use pic8259::ChainedPics;

pub mod ioapic;
pub mod lapic;

const APIC_INTERRUPT_OFFSET: u8 = 0x30;
const PIC_OFFSET: u8 = 0x20;

/// Initialises APIC.
pub fn init(apic_info: &Apic<'_, Global>) {
    lapic::init();
    disable_pic();
    ioapic::init(apic_info);
}

fn disable_pic() {
    unsafe {
        let mut pics = ChainedPics::new_contiguous(PIC_OFFSET);
        pics.initialize();
        pics.disable();
    }
}
