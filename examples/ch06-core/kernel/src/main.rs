#![no_std]
#![no_main]

mod console;
mod name;
mod uart;

use console::Console;
use core::arch::{asm, global_asm};
use core::fmt::Write;
use core::panic::PanicInfo;

global_asm!(
    r#"
    .section .text.boot, "ax"
    .global boot
boot:
    ldr x0, =__stack_top
    mov sp, x0
    b kernel_main
"#
);

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    // boot 已设置好栈，当前只有一个核心，尚未使用 .bss 中的静态变量。
    unsafe { clear_bss() };

    uart::init();
    let mut console = Console;
    let name = "World";
    writeln!(console, "Hello {name}!").unwrap();
    writeln!(console, "1 + 2 = {}, {:08x}", 1 + 2, 0x1234_abcd_u32).unwrap();

    let page_size: usize = 0x1000;
    let start: usize = 0x1234;
    let page = start.checked_next_multiple_of(page_size).unwrap();
    writeln!(console, "usize: {} bits", usize::BITS).unwrap();
    writeln!(
        console,
        "{start:#x} -> {page:#x}, aligned: {}",
        page.is_multiple_of(page_size)
    )
    .unwrap();

    let mut field = [0xa5; 12];
    let len = name::copy_name(&mut field, "hello.txt").unwrap();
    writeln!(console, "name field: {field:02x?}").unwrap();

    let stored = core::str::from_utf8(&field[..len]).unwrap();
    writeln!(console, "{stored} == hello.txt: {}", stored == "hello.txt").unwrap();
    writeln!(
        console,
        "{stored} cmp meow.txt: {:?}",
        stored.cmp("meow.txt")
    )
    .unwrap();

    loop {
        // 在内核中等待事件。被唤醒后继续等待。
        unsafe { asm!("wfe", options(nomem, nostack, preserves_flags)) }
    }
}

// 只能在启动时、首次使用静态变量之前调用。
unsafe fn clear_bss() {
    unsafe extern "C" {
        static mut __bss_start: u8;
        static mut __bss_end: u8;
    }

    let start = &raw mut __bss_start as usize;
    let end = &raw mut __bss_end as usize;
    for addr in start..end {
        unsafe { (addr as *mut u8).write_volatile(0) };
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
