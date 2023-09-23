use bootloader_api::info::FrameBuffer;
use bootloader_x86_64_common::framebuffer::FrameBufferWriter;
use conquer_once::spin::OnceCell;
use core::{fmt, fmt::Write};
use log::{Metadata, Record};
use spin::Mutex;

// FIXME: Write your own frame buffer writer

static WRITER: OnceCell<Mutex<FrameBufferWriter>> = OnceCell::uninit();

/// Initialises the frame buffer writer and logger.
pub fn init(framebuffer: &'static mut FrameBuffer) {
    let info = framebuffer.info();
    WRITER
        .try_init_once(|| Mutex::new(FrameBufferWriter::new(framebuffer.buffer_mut(), info)))
        .expect("writer already initialised");

    log::set_logger(&Logger).expect("logger already set");
    log::set_max_level(log::LevelFilter::Trace);
}

pub macro println($fmt:expr, $($arg:tt)*) {
print!(concat!($fmt, "\n"), $($arg)*)
}

pub macro print($($arg:tt)*) {
_print(format_args!($($arg)*))
}

pub fn _print(args: fmt::Arguments<'_>) {
    let mut writer = WRITER.try_get().expect("writer uninitialised").lock();
    writer.write_fmt(args).unwrap();
}

/// Logger to the frame buffer writer.
pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool { true }

    fn log(&self, record: &Record<'_>) {
        let mut writer = WRITER.try_get().expect("writer uninitialised").lock();
        writeln!(writer, "{:5}: {}", record.level(), record.args()).unwrap();
    }

    fn flush(&self) {}
}
