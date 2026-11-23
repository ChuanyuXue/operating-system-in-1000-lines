---
title: Hello World!
---

# Hello World!

上一章，我们对照 PC 和链接映射，确认 CPU 已经进入 `kernel_main`。这次要让内核自己报个到：先把 `Hello World!` 逐字节送到终端，再让它输出带十进制和十六进制数值的一行文字。

跟着上一章动手的读者可以继续使用自己的工程。下面只改串口输出和 QEMU 启动参数，`boot` 设置栈、Rust 清零 `.bss` 的顺序照旧。本章完成时的工程在教程仓库的 `examples/ch05-hello-world/`。[本章完整源码](https://github.com/ChuanyuXue/operating-system-in-1000-lines/tree/main/examples/ch05-hello-world)

## 通过 PL011 串口说“hello”

> **与原书对照**　原书的 `putchar` 执行 `ecall`，请 OpenSBI 代为输出字符。我们的内核运行在 QEMU 的 Arm `virt` 板上，可以直接访问 PL011 串口。两条路径的共同目标很小：先把一个字节送出去。

第 02 章的 `str` 指令把值写到一个地址。在这台虚拟机的物理地址空间里，有些地址对应 RAM，还有一些地址对应设备寄存器。向串口的数据寄存器写入 `0x48`，设备就会发送 ASCII 字符 `H`。这叫**内存映射 I/O**（MMIO）。

先找到这个地址。QEMU 为这台机器生成的设备树中，`pl011@9000000` 的 `reg` 属性给出的基地址是 `0x0900_0000`。这是开发板的地址分配。PL011 手册规定了各寄存器相对基地址的偏移，因此知道基地址后，可以这样找到发送所需的寄存器。[QEMU virt 硬件说明](https://www.qemu.org/docs/master/system/arm/virt.html#hardware-configuration-information-for-bare-metal-programming) · [Arm PL011 手册，第 3 章](https://documentation-service.arm.com/static/5e8e36c2fd977155116a90b5)

| 寄存器 | 偏移 | 本章的用法 |
| --- | --- | --- |
| `UARTDR` | `0x00` | 写入要发送的字节。 |
| `UARTFR` | `0x18` | 读第 5 位 `TXFF`。值为 1 表示发送队列已满。 |
| `UARTIBRD`、`UARTFBRD` | `0x24`、`0x28` | 设置波特率分频值的整数和小数部分。 |
| `UARTLCR_H`、`UARTCR` | `0x2c`、`0x30` | 选择每个字符的数据位数，开启发送队列和发送器。 |

创建 `kernel/src/uart.rs`：

<<< @/../examples/ch05-hello-world/kernel/src/uart.rs

`init()` 在启动时执行一次。这台板子的串口时钟是 24 MHz，选择 115200 波特时，分频值为 `24_000_000 / (16 × 115_200) ≈ 13.0208`。小数寄存器以 1/64 为单位，因此分别写入 13 和 1。随后选择 8 位数据、开启 FIFO，最后启用串口和发送器。

`put_byte()` 才是每个字节都要走的路径。它轮询 `TXFF`，等发送队列有空位，再把字节写入 `UARTDR` 的低 8 位。`read_volatile` 和 `write_volatile` 让编译器保留每次设备访问。这里的 `unsafe` 依赖具体的板级条件：这些地址映射到 PL011，对齐方式和 32 位访问宽度符合设备要求。

### 试一试

在 `kernel/src/main.rs` 的两个 `#![...]` 属性之后添加 `mod uart;`。在 `kernel_main` 中，紧接 `unsafe { clear_bss() };`、进入原有的等待循环之前，加上：

```rust
uart::init();
for &byte in b"Hello World!\r\n" {
    uart::put_byte(byte);
}
```

上一章让 QEMU 用 `-S` 等待调试，并用 `-serial none` 断开了串口。本章让 CPU 直接运行，把第一个串口接到宿主终端。将工程根目录的 `run.sh` 更新为：

<<< @/../examples/ch05-hello-world/run.sh{sh}

在工程目录运行下面的命令。看到输出后，先按 **Ctrl+A**，松开，再按 **X** 退出 QEMU。

```sh
sh run.sh
```

::: details 查看运行结果（客机串口）
```text
Hello World!
```
:::

跟着第一个字符走一遍：Rust 循环取出 `H` 的字节值 `0x48`，`put_byte` 把它写到 `0x0900_0000`，QEMU 的 PL011 模型接收这次写入，再把字节送到宿主终端。末尾的 `\r` 和 `\n` 分别让光标回到行首并移到下一行。

## 格式化输出

有了 `put_byte`，下一步就可以打印计算结果了。原书的 `printf` 自己解析 `%s`、`%d` 和 `%x`。Rust 的 `core::fmt` 已经能格式化字符串和整数，在 `#![no_std]` 内核里也能用。我们只需告诉它：得到一段文本后，怎样把它送到串口？

创建 `kernel/src/console.rs`，为 `Console` 实现 `core::fmt::Write`。这个 trait 要求实现的方法是 `write_str`，参数是已经格式化好的一段 UTF-8 文本。[Rust 文档：core::fmt::Write](https://doc.rust-lang.org/core/fmt/trait.Write.html)

<<< @/../examples/ch05-hello-world/kernel/src/console.rs

`text.bytes()` 依次取出 UTF-8 字节，沿用刚刚验证过的 `put_byte`。换行的处理也放在这里：调用方只写 `\n`，串口收到 `\r\n`。整个过程可以分段发送文本，无需先在堆上构造一个 `String`。

回到 `kernel/src/main.rs`，保留 `mod uart;`，在模块顶层添加：

```rust
mod console;
use console::Console;
use core::fmt::Write;
```

保留 `uart::init();`，把刚才逐字节发送问候语的 `for` 循环替换为下面四行：

```rust
let mut console = Console;
let name = "World";
writeln!(console, "Hello {name}!").unwrap();
writeln!(console, "1 + 2 = {}, {:08x}", 1 + 2, 0x1234_abcd_u32).unwrap();
```

`{name}` 插入字符串，`{}` 按十进制显示加法结果，`{:08x}` 用小写十六进制显示整数，不足八位时在左边补零。`writeln!` 负责在每行末尾追加换行。上面的 `unwrap()` 取出写入结果。这个轮询实现会等到字节可以发送，`write_str` 完成后总是返回 `Ok(())`。

再次启动内核：

```sh
sh run.sh
```

::: details 查看运行结果（客机串口）
```text
Hello World!
1 + 2 = 3, 1234abcd
```
:::

现在要观察内核里的一个地址或整数，只需在需要的位置调用 `writeln!`，就能在同一个终端看到它的值。
