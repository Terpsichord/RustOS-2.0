#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(alloc_error_handler)]
#![feature(never_type)]
#![feature(decl_macro)]
#![warn(clippy::all)]
#![warn(rust_2018_idioms)]
#![allow(clippy::new_without_default)]
#![deny(unsafe_op_in_unsafe_fn)]

extern crate alloc;

use crate::{
    apic::lapic,
    mem::{frame_allocator::BootInfoFrameAllocator, heap},
};
use bootloader_api::BootInfo;
pub use writer::{print, println};
use x86_64::{
    instructions::{hlt, interrupts},
    VirtAddr,
};

mod acpi;
mod apic;
mod gdt;
mod idt;
mod mem;
mod serial;
pub mod smp;
mod writer;

pub fn init(boot_info: &'static mut BootInfo) {
    interrupts::disable();

    let phys_mem_offset = VirtAddr::new(
        boot_info
            .physical_memory_offset
            .into_option()
            .expect("couldn't get memory offset"),
    );

    let frame_buffer = boot_info
        .framebuffer
        .as_mut()
        .expect("couldn't get framebuffer");

    let rsdp_addr = boot_info
        .rsdp_addr
        .into_option()
        .expect("couldn't get rsdp address");

    writer::init(frame_buffer);

    let frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };
    unsafe { mem::init(phys_mem_offset, frame_allocator) }

    gdt::init();
    idt::init();

    heap::init().expect("heap initialization failed");

    let acpi_tables = unsafe { acpi::init(phys_mem_offset.as_u64(), rsdp_addr) };

    let apic_info = acpi::get_apic_info(&acpi_tables);

    apic::init(apic_info);

    interrupts::enable();

    log::info!("initialised");
}

pub fn hlt_loop() -> ! {
    loop {
        hlt();
    }
}
