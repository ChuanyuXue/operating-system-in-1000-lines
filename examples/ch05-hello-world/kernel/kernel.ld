ENTRY(boot)

SECTIONS {
    . = 0x40400000;
    .text : {
        KEEP(*(.text.boot))
        *(.text .text.*)
    }
    .rodata : { *(.rodata .rodata.*) }
    .data : { *(.data .data.*) }
    .bss (NOLOAD) : {
        __bss_start = .;
        *(.bss .bss.*)
        __bss_end = .;
    }
    .stack (NOLOAD) : ALIGN(16) {
        . += 128K;
        __stack_top = .;
    }
}
