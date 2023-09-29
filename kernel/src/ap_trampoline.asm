global ap_trampoline

align 4096
bits 16

ap_trampoline:
    mov al, 0x45
    outb al, 0x80
.loop:
    hlt
    jmp ap_trampoline.loop
