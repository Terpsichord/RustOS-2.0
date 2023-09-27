#![no_std]
#![no_main]

use bootloader_api::{config::Mapping, entry_point, BootInfo, BootloaderConfig};
use core::panic::PanicInfo;
use kernel::{
    hlt_loop,
    task::{executor::Executor, keyboard, Task},
};
use x86_64::instructions::interrupts;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(main, config = &BOOTLOADER_CONFIG);

fn main(boot_info: &'static mut BootInfo) -> ! {
    kernel::init(boot_info);

    interrupts::int3();
    log::info!("Hello, World!");

    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{info}");
    hlt_loop();
}
