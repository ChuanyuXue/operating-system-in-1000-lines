#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;

global_asm!(
    r#"
    .section .text.boot, "ax"
    .global boot
boot:
    ldr x0, =__stack_top
    mov sp, x0

    ldr x0, =__bss_start
    ldr x1, =__bss_end
1:
    cmp x0, x1
    b.hs 2f
    strb wzr, [x0], #1
    b 1b
2:
    b kernel_main
"#
);

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    loop {
        // 在内核中等待事件。被唤醒后继续等待。
        unsafe { asm!("wfe", options(nomem, nostack, preserves_flags)) }
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
