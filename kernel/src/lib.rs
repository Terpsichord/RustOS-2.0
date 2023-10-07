#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(alloc_error_handler)]
#![feature(custom_test_frameworks)]
#![feature(decl_macro)]
#![feature(never_type)]
#![cfg_attr(test, no_main)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::style)]
#![warn(rust_2018_idioms)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::new_without_default)]
#![deny(unsafe_op_in_unsafe_fn)]

extern crate alloc;

use crate::{
    apic::lapic,
    mem::{frame_allocator, heap},
    task::{keyboard, Executor},
};
use bootloader_api::BootInfo;
use frame_allocator::PageFrameAllocator;
pub use writer::{print, println};
use x86_64::{
    instructions::{hlt, interrupts},
    VirtAddr,
};
#[cfg(test)]
use {bootloader_api::entry_point, core::panic::PanicInfo};

mod acpi;
mod apic;
mod gdt;
mod idt;
mod mem;
mod serial;
pub mod task;
pub mod testing;
mod writer;

/// Initialise the kernel.
///
/// # Panics
/// This function panics if the `boot_info` is missing any required fields.
pub fn init(boot_info: &'static mut BootInfo) -> Executor {
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

    let frame_allocator = unsafe { PageFrameAllocator::init(&boot_info.memory_regions) };
    unsafe { mem::init(phys_mem_offset, frame_allocator) }

    gdt::init();
    idt::init();

    heap::init().expect("heap initialization failed");

    let acpi_tables = unsafe { acpi::init(phys_mem_offset.as_u64(), rsdp_addr) };

    let apic_info = acpi::get_apic_info(&acpi_tables);

    apic::init(&apic_info);

    interrupts::enable();

    let (executor, mut spawner) = task::init_executor_and_spawner();
    spawner.spawn(keyboard::print_keypresses());

    log::info!("initialised");

    executor
}

pub fn hlt_loop() -> ! {
    loop {
        hlt();
    }
}

#[cfg(test)]
entry_point!(main, config = &testing::BOOTLOADER_CONFIG);

#[cfg(test)]
pub fn main(_boot_info: &'static mut BootInfo) -> ! {
    test_main();

    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! { testing::test_panic_handler(info); }
