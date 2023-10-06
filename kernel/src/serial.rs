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
pub macro print($($arg:tt)*) {
_print(format_args!($($arg)*));
}

/// Prints to the host through the serial interface, appending a newline.
pub macro println {
() => { print!("\n"); },
($($arg:tt)*) => { print!("{}\n", format_args!($($arg)*)) },
}
