use crate::print;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_core::Stream;
use futures_util::{task::AtomicWaker, StreamExt};
use pc_keyboard::{layouts::Uk105Key, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Lazy;

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

        WAKER.register(&cx.waker());
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
    if let Ok(_) = SCANCODE_QUEUE.push(scancode) {
        WAKER.wake();
    } else {
        log::warn!("scancode queue full; dropping keyboard input");
    }
}

pub async fn print_keypresses() {
    let mut keyboard = Keyboard::new(ScancodeSet1::new(), Uk105Key, HandleControl::Ignore);
    let mut scancodes = ScancodeStream::new();

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}