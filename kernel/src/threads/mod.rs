pub use context_switch::context_switch;

pub mod context_switch;
pub mod process;
pub mod scheduler;
pub mod thread;

#[repr(C)]
pub struct Context {
    r15: usize,
    r14: usize,
    r13: usize,
    r12: usize,

    rbx: usize,
    rbp: usize,

    rflags: usize,
    rip: usize,
}

pub fn init() {}
