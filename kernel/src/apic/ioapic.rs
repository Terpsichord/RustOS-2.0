use crate::{
    apic::{lapic, APIC_INTERRUPT_OFFSET},
    idt::InterruptVector,
    mem::MANAGER,
};
use acpi::platform::interrupt::Apic;
use alloc::alloc::Global;
use x2apic::ioapic::{IoApic, IrqFlags, IrqMode};
use x86_64::{structures::paging::PhysFrame, PhysAddr};

const KEYBOARD_IRQ: u8 = 1;

/// Initialises the I/O APIC from the address in `apic_info`.
pub(super) fn init(apic_info: Apic<'_, Global>) {
    let phys_addr = PhysAddr::new(apic_info.io_apics[0].address as u64);
    let mut manager = MANAGER.get().expect("couldn't get memory manager").lock();

    let mut ioapic;
    unsafe {
        let page = manager.create_mapping(PhysFrame::containing_address(phys_addr));
        let virt_addr = page.start_address();

        ioapic = IoApic::new(virt_addr.as_u64());

        ioapic.init(APIC_INTERRUPT_OFFSET);
    }

    add_keyboard_entry(&mut ioapic);

    log::info!("{} io apic(s)", apic_info.io_apics.len());
}

fn add_keyboard_entry(ioapic: &mut IoApic) {
    let mut entry = unsafe { ioapic.table_entry(KEYBOARD_IRQ) };

    let dest_id = unsafe { lapic::get().id() };
    entry.set_dest(dest_id as u8);
    entry.set_vector(InterruptVector::Keyboard as u8);
    entry.set_mode(IrqMode::Fixed);
    entry.set_flags(IrqFlags::LEVEL_TRIGGERED | IrqFlags::LOW_ACTIVE | IrqFlags::MASKED);

    unsafe {
        ioapic.set_table_entry(KEYBOARD_IRQ, entry);
        ioapic.enable_irq(KEYBOARD_IRQ);
    }
}
