//;    .code16
//;ap_trampoline:
//;    cli
//;    cld
//;    ljmp    $0, $0x8040
//;    .align 16
//;_L8010_GDT_table:
//;    .long 0, 0
//;    .long 0x0000FFFF, 0x00CF9A00    ; flat code
//;    .long 0x0000FFFF, 0x008F9200    ; flat data
//;    .long 0x00000068, 0x00CF8900    ; tss
//;_L8030_GDT_value:
//;    .word _L8030_GDT_value - _L8010_GDT_table - 1
//;    .long 0x8010
//;    .long 0, 0
//;    .align 64
//;_L8040:
//;    xorw    %ax, %ax
//;    movw    %ax, %ds
//;    lgdtl   0x8030
//;    movl    %cr0, %eax
//;    orl     $1, %eax
//;    movl    %eax, %cr0
//;    ljmp    $8, $0x8060
//;    .align 32
//;    .code32
//;_L8060:
//;    movw    $16, %ax
//;    movw    %ax, %ds
//;    movw    %ax, %ss
//;    ; get our Local APIC ID
//;    mov     $1, %eax
//;    cpuid
//;    shrl    $24, %ebx
//;    movl    %ebx, %edi
//;    ; set up 32k stack, one for each core. It is important that all core must have its own stack
//;    shll    $15, %ebx
//;    movl    stack_top, %esp
//;    subl    %ebx, %esp
//;    pushl   %edi
//;    ; spinlock, wait for the BSP to finish
//;1:  pause
//;    cmpb    $0, bspdone
//;    jz      1b
//;    lock    incb aprunning
//;    ; jump into C code (should never return)
//;    ljmp    $8, $ap_startup

.section .smp_trampoline, "awx"
.global _smp_trampoline
.global stack_avail
.global undefined_instruction
.global gdt_64_pointer

.code16
_smp_trampoline:
  # clear the direction flag (e.g. go forward in memory when using
  # instructions like lodsb)
  cld
  # disable interrupts
  cli

  # Set the A20 line
  in al, 0x92
  or al, 2
  out 0x92, al

  # Load 32-bit GDT
  lgdt gdt_64_pointer

  # Enable protected mode
  mov eax, cr0
  or  eax, (1 << 0)
  mov cr0, eax

  ljmp 24, offset protected_mode_setup

.align 4
.code32
protected_mode_setup:
  mov bx, 32
  mov ds, bx
  mov es, bx
  mov fs, bx
  mov gs, bx
  mov ss, bx


# spin loop till stack is available
wait_for_stack:
  xor al, al
  lock xchg byte ptr [stack_avail], al
  test al, al
  jz wait_for_stack
  mov byte ptr [stack_avail], 0

  mov esp, offset STACK_ARRAY
  add esp, STACK_SIZE
  call smp_main

spin:
  jmp spin

undefined_instruction:
  .long 0xffffffff

.align 2
stack_avail: .byte 1

.align 4
gdt_64:
    .quad 0x0000000000000000          # Null Descriptor - should be present.
    .quad 0x00209A0000000000          # 8  | 64-bit code descriptor (exec/read).
    .quad 0x0000920000000000          # 16 | 64-bit data descriptor (read/write).
    .quad 0x00cf9a000000ffff          # 24 | 32-bit code descriptor (exec/read).
    .quad 0x00cf92000000ffff          # 32 | 32-bit data descriptor (read/write).
    .quad 0x00009a004000ffff          # 40 | 16-bit code descriptor
    .quad 0x00009a007c00ffff          # 48 | 16-bit data descriptor

.align 4
gdt_64_pointer:
    .word gdt_64_pointer - gdt_64 - 1    # 16-bit Size (Limit) of GDT.
    .long gdt_64                         # 64-bit Base Address of GDT. (CPU will zero extend to 64-bit)