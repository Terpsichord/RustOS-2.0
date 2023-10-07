use crate::{hlt_loop, serial};
use bootloader_api::{config::Mapping, BootloaderConfig};
use core::{any, panic::PanicInfo};
use x86_64::instructions::port::Port;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

pub trait Test {
    fn run(&self);
}

impl<T> Test for T
where
    T: Fn(),
{
    fn run(&self) {
        serial::print!("test {} ... ", any::type_name::<T>());
        self();
        serial::println!("ok");
    }
}

pub fn test_runner(tests: &[&dyn Test]) {
    serial::println!("TESTttTING");

    serial::println!(
        "running {} test{}",
        tests.len(),
        if tests.len() == 1 { "" } else { "s" }
    );
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo<'_>) -> ! {
    serial::println!("FAILED\n");
    log::error!("{}", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[repr(u32)]
enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

fn exit_qemu(exit_code: QemuExitCode) {
    unsafe {
        Port::new(0xf4).write(exit_code as u32);
    }
}
