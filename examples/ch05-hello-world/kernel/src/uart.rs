const UART_BASE: usize = 0x0900_0000;
const UARTDR: *mut u32 = UART_BASE as *mut u32;
const UARTFR: *const u32 = (UART_BASE + 0x18) as *const u32;
const UARTIBRD: *mut u32 = (UART_BASE + 0x24) as *mut u32;
const UARTFBRD: *mut u32 = (UART_BASE + 0x28) as *mut u32;
const UARTLCR_H: *mut u32 = (UART_BASE + 0x2c) as *mut u32;
const UARTCR: *mut u32 = (UART_BASE + 0x30) as *mut u32;
const TXFF: u32 = 1 << 5;

// 内核启动时调用一次，此时串口还没有待发送的数据。
pub fn init() {
    unsafe {
        UARTCR.write_volatile(0);
        // 24 MHz 时钟，115200 波特：整数分频 13，小数部分 1/64。
        UARTIBRD.write_volatile(13);
        UARTFBRD.write_volatile(1);
        UARTLCR_H.write_volatile((0b11 << 5) | (1 << 4)); // 8 位数据，启用 FIFO
        UARTCR.write_volatile((1 << 8) | 1); // TXE、UARTEN
    }
}

pub fn put_byte(byte: u8) {
    // 当前 virt 板把 PL011 映射在这个地址，两个寄存器均按 32 位访问。
    unsafe {
        while UARTFR.read_volatile() & TXFF != 0 {
            core::hint::spin_loop();
        }
        UARTDR.write_volatile(u32::from(byte));
    }
}
