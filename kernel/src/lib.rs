#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(alloc_error_handler)]
#![feature(custom_test_frameworks)]
#![feature(decl_macro)]
#![feature(never_type)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![warn(clippy::all)]
#![warn(rust_2018_idioms)]
#![allow(clippy::new_without_default)]
#![deny(unsafe_op_in_unsafe_fn)]

extern crate alloc;

use crate::{
    apic::lapic,
    mem::{frame_allocator::PageFrameAllocator, heap},
    task::{keyboard, Executor},
};
use bootloader_api::BootInfo;
use core::panic::PanicInfo;
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
pub mod task;
mod testing;
mod writer;

#[test_case]
fn addition() {
    assert_eq!(1 + 2, 3);
}

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

    apic::init(apic_info);

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
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();

    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    testing::test_panic_handler(info);
    hlt_loop();
}
