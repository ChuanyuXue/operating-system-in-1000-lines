---
title: AArch64 101
---

# AArch64 101

我是一名系统软件工程师，日常工作中经常使用 AArch64。这就是我选择用 AArch64 来移植这本书的原因。

AArch64 是 Arm 架构的 64 位执行状态。在这个状态下，处理器执行 A64 指令。本章沿用上一章的 Cortex-A53，看看 Rust 函数到了这颗 CPU 上是怎样执行的。

## QEMU 虚拟机

计算机是由不同的设备组成的：CPU、内存、网卡、硬盘等等。例如，虽然 iPhone 和树莓派都使用 Arm CPUs，但显而易见它们是不一样的计算机。

本书支持 QEMU `virt` 虚拟机（[文档](https://www.qemu.org/docs/master/system/arm/virt.html)）是因为：

- 哪怕它不是真实存在于这个世界的，它也是简单的并且跟真实设备非常相似。
- 你可以免费使用 QEMU 来模拟而不需要购买物理硬件。
- 当你遇到调试问题，你可以阅读 QEMU 的代码、或者用调试器附加到 QEMU 进程来研究哪里出错了。

## AArch64 汇编入门

上一章用 `file` 看到了目标文件的格式。这次让 Rust 编译器输出汇编，就可以读到它为函数选择的指令。把上一章的 `probe.rs` 改成下面这样。`extern "C"` 让函数采用目标平台的 C 调用约定，`#[unsafe(no_mangle)]` 以原名导出函数，这样在汇编中也能直接找到 `add_one`。使用这个属性时，要确保程序里只有一个同名的导出符号。

```rust [probe.rs]
#![no_std]

#[unsafe(no_mangle)]
pub extern "C" fn add_one(value: u32) -> u32 {
    value.wrapping_add(1)
}
```

打开优化，用 `--emit=asm` 将汇编写入文件，再从函数名读到返回指令：

```sh
rustc +stable --target aarch64-unknown-none-softfloat --crate-type lib -O --emit=asm probe.rs -o probe.s
sed -n '/^add_one:/,/^[[:space:]]*ret/p' probe.s
```

::: details 查看运行结果
```asm
add_one:
    .cfi_startproc
    add w0, w0, #1
    ret
```
:::

### 汇编语言基础

`add_one:` 是标记函数地址的标签。`.cfi_startproc` 是给汇编器的指示。CPU 在这里执行的第一条指令是：

```asm
add w0, w0, #1
```

从左往右读：`add` 做加法，第一个 `w0` 是结果写入的位置，第二个 `w0` 是原来的值，`#1` 是直接写在指令里的常数（立即数）。如果进入函数时 `w0` 是 41，执行后它就是 42。

### 寄存器

上一章在 QEMU 监视器里看到的 `X00`，在汇编中写作 `x0`。`x0` 有 64 位，`w0` 是同一个寄存器的低 32 位。写入 `w0` 时，`x0` 的高 32 位会清零。`add_one` 接收一个 `u32`，编译器用 `w0` 就能完成这次加法。

| 名称 | 本章中的用途 |
| --- | --- |
| `x0` 到 `x7` | 按调用约定传递前几个整数或指针参数。一个整数结果通常通过 `w0` 或 `x0` 返回。 |
| `x30`（`lr`） | 链接寄存器，保存函数调用的返回地址。 |
| `sp` | 栈指针，指向当前栈顶。 |
| `pc` | 程序计数器。在 QEMU 暂停时可以查看下一条要执行的指令位于哪里。 |

### 内存访问

假设 `x1` 里装着一个可以读写的、按 4 字节对齐的 `u32` 的地址。下面三条指令会把内存里的值加一：

```asm
ldr w0, [x1]    // 从 x1 指向的地址读取 4 字节
add w0, w0, #1
str w0, [x1]    // 向同一个地址写回 4 字节
```

方括号表示按地址访问内存。地址用 64 位的 `x1` 保存，这次读写的宽度由 `w0` 决定。执行完 `add` 时只有寄存器的值变了，直到 `str` 执行后，内存里的值才会更新。

## 分支指令

标签也可以标记循环的开头。`b 标签` 会无条件跳到那个位置，`cbnz`（compare and branch on nonzero）则先检查一个寄存器：

```asm
    mov w0, #3
count_down:
    sub w0, w0, #1
    cbnz w0, count_down
```

第一次执行 `cbnz` 时，`w0` 等于 2，于是 CPU 回到 `count_down`。第三次检查时，`w0` 已经减到 0，CPU 继续执行分支下面的指令。

### 函数调用

在调用方，给 `add_one` 传入 41 只需准备一个参数：

```asm
mov w0, #41
bl  add_one
mov w1, w0
```

`bl`（branch with link）把下一条指令的地址写入 `x30`，再跳到 `add_one`。回看前面编译出的两条指令：`add` 将 42 写入 `w0`，`ret` 默认跳回 `x30` 中的地址。于是调用方接着执行 `mov w1, w0`，此时 `w1` 也等于 42。

> **与原书对照**　原书的 RISC-V 示例用 `a0` 传入整数并取回结果，用 `ra` 保存返回地址。在这个 AArch64 示例中，对应的是 `w0` 和 `x30`。寄存器的分工称为调用约定。[Arm 过程调用标准](https://github.com/ARM-software/abi-aa/blob/main/aapcs64/aapcs64.rst)

### 栈

如果一个函数本身还要调用另一个函数，第二次 `bl` 就会覆盖 `x30`。调用前可以先把自己的返回地址保存在栈里，等调用结束再取回来：

```asm
call_add_one:
    sub sp, sp, #16
    str x30, [sp]
    bl  add_one
    ldr x30, [sp]
    add sp, sp, #16
    ret
```

栈向低地址增长，减小 `sp` 就为当前函数腾出了空间。`x30` 只占 8 字节，这里分配 16 字节可以保持栈指针的 16 字节对齐。最后的 `ret` 使用恢复后的地址，回到 `call_add_one` 的调用方。这段代码也解释了为什么引导时要先设置 `sp`：编译器随时可能生成像 `str x30, [sp]` 这样的栈访问。

## CPU 模式

AArch64 用异常级别（Exception Level，简称 EL）表示当前代码拥有的权限。我们后面会分别在内核和用户程序里执行代码：

| 级别 | 常见用途 |
| --- | --- |
| EL0 | 用户程序 |
| EL1 | 操作系统内核 |
| EL2 | 虚拟机监控器 |
| EL3 | 安全监控固件 |

打开上一章的 QEMU 监视器运行结果，找到 `PSTATE` 一行。这次运行显示 `EL1h`，表示 CPU 暂停时处于 EL1，使用该级别的栈指针。

## 特权指令

内核还可以通过系统寄存器查看或改变处理器状态。例如，`CurrentEL` 的第 3、2 位保存当前异常级别。下面几种操作会在内核代码中出现，具体访问权限取决于指令和所访问的寄存器：

| 指令示例 | 作用 |
| --- | --- |
| `mrs x0, CurrentEL` | 把当前异常级别的编码读入通用寄存器。 |
| `msr VBAR_EL1, x0` | 设置 EL1 异常向量表的基址。 |
| `eret` | 从异常返回，恢复相应的程序状态和执行地址。 |

## 内联汇编

在 Rust 函数里，可以用 `core::arch::asm!` 写一小段需要直接交给 CPU 的指令。例如，内核想取得当前异常级别时，可以让汇编读取系统寄存器，再用普通的 Rust 表达式取出对应的位。

### 如何编写内联汇编代码

`asm!` 的第一个参数是指令模板。模板里的 `{raw}` 是占位符，后面的 `raw = out(reg) value` 要求编译器选一个通用寄存器，把指令写入该寄存器的结果交给 Rust 变量 `value`。相反，`in(reg)` 可以把 Rust 的值作为指令输入。[Rust 内联汇编参考](https://doc.rust-lang.org/reference/inline-assembly.html)

### 示例

把这个函数追加到 `probe.rs`：

```rust
#[unsafe(no_mangle)]
pub extern "C" fn current_el() -> u64 {
    let value: u64;
    // 内核运行在 EL1，可以读取 CurrentEL。
    unsafe { core::arch::asm!("mrs {raw}, CurrentEL", raw = out(reg) value) }
    (value >> 2) & 0b11
}
```

用同一个裸机目标再次编译，查看 `current_el` 的函数体：

```sh
rustc +stable --target aarch64-unknown-none-softfloat --crate-type lib -O --emit=asm probe.rs -o probe.s
sed -n '/^current_el:/,/^[[:space:]]*ret/p' probe.s
```

::: details 查看运行结果
```asm
current_el:
    .cfi_startproc
    //APP
    mrs x8, CurrentEL
    //NO_APP
    ubfx    x0, x8, #2, #2
    ret
```
:::

这次编译器选了 `x8` 接收寄存器值，再用 `ubfx x0, x8, #2, #2` 从第 2 位起取出两位，放进返回值寄存器 `x0`。在 EL1 执行时，`CurrentEL` 的编码是 `0b0100`，函数取出第 3、2 位后得到 1。这里在宿主机上做的是交叉编译。真正执行这条特权寄存器读取指令，需要等内核在虚拟机里运行。
