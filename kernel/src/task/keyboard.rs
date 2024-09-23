use crate::print;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_core::Stream;
use futures_util::{task::AtomicWaker, StreamExt};
use pc_keyboard::{
    layouts::Uk105Key, DecodedKey, HandleControl, Keyboard, KeyboardLayout, ScancodeSet,
    ScancodeSet1,
};
use spin::Lazy;
use vte::ansi;

static SCANCODE_QUEUE: Lazy<ArrayQueue<u8>> = Lazy::new(|| ArrayQueue::new(100));
static WAKER: AtomicWaker = AtomicWaker::new();

pub struct ScancodeStream {
    _priv: (),
}

impl ScancodeStream {
    pub fn new() -> Self { Self { _priv: () } }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(scancode) = SCANCODE_QUEUE.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(cx.waker());
        match SCANCODE_QUEUE.pop() {
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}

pub fn push_scancode(scancode: u8) {
    if SCANCODE_QUEUE.push(scancode).is_ok() {
        WAKER.wake();
    } else {
        log::warn!("scancode queue full; dropping keyboard input");
    }
}

fn print_key(key: DecodedKey) {
    match key {
        DecodedKey::Unicode(character) => print!("{}", character),
        DecodedKey::RawKey(key) => print!("{:?}", key),
    }
}

pub async fn print_keypresses() { process_keypresses(print_key).await }

pub async fn process_keypresses(mut process_key: impl FnMut(DecodedKey)) {
    let mut keyboard = Keyboard::new(ScancodeSet1::new(), Uk105Key, HandleControl::Ignore);
    let mut scancodes = ScancodeStream::new();

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                process_key(key)
            }
        }
    }
}

/// A (not-yet) ANSI-compatible terminal
struct Terminal<S: ScancodeSet, L: KeyboardLayout> {
    keyboard: Keyboard<L, S>,
    // escape_parser: EscapeParser,
}

struct Handler {
    color: Option<ansi::Rgb>,
}

// https://github.com/alacritty/vte/blob/master/src/ansi.rs#L1990
impl ansi::Handler for Handler {
    fn set_color(&mut self, _: usize, color: ansi::Rgb) { self.color = Some(color); }
}
// #[derive(Default)]
// enum EscapeParser {
//     #[default]
//     Ground,
//     Escape,
//     EscapeIntermediate,
//     CsiEntry,
//     CsiParam,
//     CsiIntermediate,
//     CsiIgnore,
// }
//
// impl<S: ScancodeSet, L: KeyboardLayout> Terminal<S, L> {
//     fn new(scancode_set: S, layout: L) -> Self {
//         Self {
//             keyboard: Keyboard::new(scancode_set, layout,
// HandleControl::Ignore),             escape_parser: Default::default(),
//         }
//     }
//
//     fn add_byte(&mut self, byte: u8) {
//         // Reference for escape sequence parsing
//         // https://vt100.net/annarbor/aaa-ug/section13.html
//
//         match self.escape_parser {
//             EscapeParser::Start => {
//                 if byte == b'\x1b' {
//                     self.escape_parser = EscapeParser::Escape;
//                 }
//             }
//             EscapeParser::Escape => {
//                 match byte {
//                     b'[' => self.escape_parser = EscapeParser::Control,
//                     _ => todo!(),
//                 }
//                 if byte == b'[' {
//                     self.escape_parser = EscapeParser::Control;
//                 }
//             }
//             EscapeParser::Control => todo!(),
//         }
//     }
// }
//
// pub async fn use_keypresses() { process_keypresses(use_key).await }
//
// fn use_key(key: DecodedKey) {
//     // static
// }
