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

static SCANCODE_QUEUE: Lazy<ArrayQueue<u8>> = Lazy::new(|| ArrayQueue::new(100));
static WAKER: AtomicWaker = AtomicWaker::new();

pub struct ScancodeStream {
    _priv: (),
}

impl ScancodeStream {
    pub fn new() -> Self { Self { _priv: () } }

    async fn next_char<S, L>(&mut self, keyboard: &mut Keyboard<L, S>) -> Option<char>
    where
        S: ScancodeSet,
        L: KeyboardLayout,
    {
        let scancode = self.next().await?;
        let key_event = keyboard.add_byte(scancode).ok().flatten()?;
        let key = keyboard.process_keyevent(key_event)?;
        match key {
            DecodedKey::Unicode(c) => Some(c),
            _ => None,
        }
    }
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

pub async fn print_keypresses() {
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        Uk105Key,
        HandleControl::MapLettersToUnicode,
    );
    let mut scancodes = ScancodeStream::new();

    loop {
        let Some(character) = scancodes.next_char(&mut keyboard).await else {
            continue;
        };

        print!("{}", character);
    }
}
