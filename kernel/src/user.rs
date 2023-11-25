use core::arch::asm;

#[naked]
pub unsafe extern "C" fn userpace_program() {
    unsafe {
        asm!(
            "\
            nop
            nop
            nop
        ",
            options(noreturn)
        );
    }
}
