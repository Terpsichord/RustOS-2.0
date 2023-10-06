use crate::{hlt_loop, lapic, task::keyboard, threads::scheduler};
use conquer_once::spin::Lazy;
use x86_64::{
    instructions::{interrupts, port::PortReadOnly},
    registers::control::Cr2,
    structures::idt::{
        InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode, SelectorErrorCode,
    },
};

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    unsafe {
        idt.breakpoint
            .set_handler_fn(breakpoint_handler)
            .set_stack_index(IstIndex::General as u16);
        idt.divide_error
            .set_handler_fn(divide_error_handler)
            .set_stack_index(IstIndex::General as u16);
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(IstIndex::General as u16);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault_handler)
            .set_stack_index(IstIndex::General as u16);
        idt.page_fault
            .set_handler_fn(page_fault_handler)
            .set_stack_index(IstIndex::General as u16);
        idt.invalid_tss
            .set_handler_fn(invalid_tss_handler)
            .set_stack_index(IstIndex::General as u16);

        idt[InterruptVector::Timer as usize]
            .set_handler_fn(timer_handler)
            .set_stack_index(IstIndex::Timer as u16);
        idt[InterruptVector::Keyboard as usize]
            .set_handler_fn(keyboard_handler)
            .set_stack_index(IstIndex::General as u16);
    }

    idt
});

const INTERRUPT_VECTOR_OFFSET: u8 = 0x30;

/// The IDT index of each interrupt
#[repr(u8)]
pub enum InterruptVector {
    Timer = INTERRUPT_VECTOR_OFFSET,
    Error,
    Spurious,
    Keyboard,
}

#[repr(u16)]
enum IstIndex {
    General = 0,
    Timer,
}

pub fn init() { IDT.load(); }

pub extern "x86-interrupt" fn timer_handler(stack_frame: InterruptStackFrame) {
    interrupts::without_interrupts(|| unsafe {
        lapic::get().end_of_interrupt();
        scheduler::get().schedule(stack_frame.stack_pointer);
    })
}

extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
    interrupts::without_interrupts(|| {
        log::info!("BREAKPOINT");
    });
}

extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    interrupts::without_interrupts(|| {
        let scancode = unsafe { PortReadOnly::new(0x60).read() };
        keyboard::push_scancode(scancode);

        unsafe {
            lapic::get().end_of_interrupt();
        }
    });
}

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    log::error!("DIVIDE ERROR\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _err_code: u64,
) -> ! {
    log::error!("DOUBLE FAULT\n{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    err_code: u64,
) {
    let selector_error = SelectorErrorCode::new_truncate(err_code);
    log::error!(
        "GENERAL PROTECTION FAULT\nSelector Error: {:?}\n{:#?}",
        selector_error,
        stack_frame
    );
    hlt_loop();
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    err_code: PageFaultErrorCode,
) {
    let address = Cr2::read();
    log::error!(
        "PAGE FAULT ({:?})\nAccessed Address: {:?}\n{:#?}",
        err_code,
        address,
        stack_frame,
    );
    hlt_loop();
}

extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, err_code: u64) {
    let selector_error = SelectorErrorCode::new(err_code).expect("couldn't get selector error");
    log::error!(
        "INVALID TSS\nSelector Error: {:?}\n{:#?}",
        selector_error,
        stack_frame
    );
    hlt_loop();
}
// extern "x86-interrupt" fn (stack_frame: InterruptStackFrame, err_code:
// SelectorErrorCode) {     log::error!("INVALID TSS\nSelector Error:
// {:?}\n{:#?}", err_code, stack_frame);     hlt_loop();
// }
