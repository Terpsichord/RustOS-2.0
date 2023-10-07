use crate::serial;
use core::{any, panic::PanicInfo};
use x86_64::instructions::port::Port;

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

pub fn test_panic_handler(info: &PanicInfo<'_>) {
    serial::println!("FAILED\n");
    log::error!("{}", info);
    exit_qemu(QemuExitCode::Failed);
}

#[repr(u32)]
enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

fn exit_qemu(exit_code: QemuExitCode) { unsafe { Port::new(0xf4).write(exit_code as u32) } }
