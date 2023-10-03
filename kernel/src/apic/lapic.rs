use crate::{idt::InterruptVector, mem};
use conquer_once::spin::OnceCell;
use spin::{Mutex, MutexGuard};
use x2apic::lapic::{xapic_base, LocalApic, LocalApicBuilder};
use x86_64::{structures::paging::PhysFrame, PhysAddr};

pub static LAPIC: OnceCell<Mutex<LocalApic>> = OnceCell::uninit();

/// Initialise the Local APIC into `LAPIC`.
pub(super) fn init() {
    let phys_addr = PhysAddr::new(unsafe { xapic_base() });
    let page = unsafe { mem::create_mapping(PhysFrame::containing_address(phys_addr)) };
    let virt_addr = page.start_address();

    let mut local_apic = LocalApicBuilder::new()
        .set_xapic_base(virt_addr.as_u64())
        .timer_vector(InterruptVector::Timer as usize)
        .error_vector(InterruptVector::Error as usize)
        .spurious_vector(InterruptVector::Spurious as usize)
        .build()
        .unwrap_or_else(|err| panic!("{err}"));

    unsafe {
        local_apic.enable();
    }

    LAPIC
        .try_init_once(|| Mutex::new(local_apic))
        .expect("lapic::init should only be called once");
}

/// Gets a lock of the `LAPIC` static.
pub fn get() -> MutexGuard<'static, LocalApic> {
    LAPIC.try_get().expect("local apic uninitialised").lock()
}
