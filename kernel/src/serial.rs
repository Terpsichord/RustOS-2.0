use bootloader_x86_64_common::serial::SerialPort;
use conquer_once::spin::Lazy;
use core::{fmt, fmt::Write};
use spin::Mutex;
use x86_64::instructions::interrupts;

static SERIAL: Lazy<Mutex<SerialPort>> = Lazy::new(|| unsafe { SerialPort::init() }.into());

#[doc(hidden)]
pub fn _print(args: fmt::Arguments<'_>) {
    interrupts::without_interrupts(|| {
        SERIAL
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });
}

/// Prints to the host through the serial interface.
pub macro serial_print($($arg:tt)*) {
_print(format_args!($($arg)*));
}

/// Prints to the host through the serial interface, appending a newline.
pub macro serial_println($fmt: expr, $($arg:tt)*) {
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
