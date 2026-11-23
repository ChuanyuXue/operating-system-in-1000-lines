---
title: 入门
---

# 入门

即使手边没有 Arm 开发板，你也可以跟着本书编写内核。我们会在自己的电脑上用 Rust 编译 AArch64 机器码，再用 QEMU 模拟一台能运行这些指令的机器。本章先检查这两件事能否分别做通。等到引导一章，我们再把它们连起来，启动第一个内核。

## 安装 QEMU

在 macOS 上，用 [Homebrew](https://brew.sh/) 安装 QEMU：

```sh
brew install qemu
```

在 Ubuntu 上，安装 Arm 系统模拟器。使用 Windows 的读者可以在 WSL2 的 Ubuntu 终端里执行这一步，并在同一终端里完成本章的后续操作。

```sh
sudo apt update
sudo apt install qemu-system-arm curl file
```

两种环境都可以用下面的命令确认模拟器已就绪：

```sh
qemu-system-aarch64 --version
```

::: details 查看运行结果（本机版本）
```text
QEMU emulator version 11.1.2
Copyright (c) 2003-2026 Fabrice Bellard and the QEMU Project developers
```
:::

## 为裸机安装 Rust 编译目标

如果还没有 `rustup`，请按 [Rust 官方安装说明](https://rust-lang.org/tools/install/)安装，并重新打开终端。用下面的命令准备稳定版工具链和本书的编译目标：

```sh
rustup toolchain install stable
rustup target add --toolchain stable aarch64-unknown-none-softfloat
```

安装后，查看编译器运行在哪个平台，并确认裸机目标已经就绪：

```sh
rustc +stable -vV
rustup target list --toolchain stable --installed | grep '^aarch64-unknown-none-softfloat$'
```

::: details 查看运行结果（Apple 芯片的 Mac）
```text
rustc 1.97.1 (8bab26f4f 2026-07-14)
binary: rustc
commit-hash: 8bab26f4f68e0e26f0bb7960be334d5b520ea452
commit-date: 2026-07-14
host: aarch64-apple-darwin
release: 1.97.1
LLVM version: 22.1.6
aarch64-unknown-none-softfloat
```
:::

为什么 Apple 芯片的 Mac 也要安装这个目标？在这台机器上，`host: aarch64-apple-darwin` 表示编译器本身运行在 Arm 架构的 macOS 上。我们的内核虽然使用同一套指令，却要在没有 macOS 的机器上独立运行。

在目标名 `aarch64-unknown-none-softfloat` 中，`aarch64` 指定指令集，`none` 表示目标上没有操作系统，`softfloat` 表示函数调用时使用软浮点约定。这个目标提供 Rust 的 `core` 库。内核可以使用整数、切片等基础类型，串口输出等功能则要由我们自己实现。[Rust 裸机 AArch64 目标说明](https://doc.rust-lang.org/rustc/platform-support/aarch64-unknown-none.html)

## 编译一个 AArch64 目标文件

先在一个空目录里创建 `probe.rs`。我们用一个整数加法函数来检验编译器，文件开头的 `#![no_std]` 让这段代码使用 `core` 提供的基础类型和方法，编译时无需链接标准库 `std`。

```rust [probe.rs]
#![no_std]

pub fn add_one(value: u32) -> u32 {
    value.wrapping_add(1)
}
```

在同一目录里运行：

```sh
rustc +stable --target aarch64-unknown-none-softfloat --crate-type lib --emit=obj probe.rs -o probe.o
file probe.o
```

::: details 查看运行结果
`rustc` 编译成功后直接返回终端，下面这行来自 `file`。

```text
probe.o: ELF 64-bit LSB relocatable, ARM aarch64, version 1 (SYSV), not stripped
```
:::

输出中应该能找到 `ELF 64-bit`、`relocatable` 和 `ARM aarch64`。这三个字段说明编译器生成了 64 位 ELF 格式的可重定位目标文件，里面装的是 AArch64 指令。可重定位意味着函数最终放在内存中的哪个地址，要等链接时才确定。

## 启动一台虚拟机

我们选用 QEMU 的 `virt` 开发板模型和 64 位的 Cortex-A53 处理器。执行下面的命令，让虚拟 CPU 在执行第一条指令之前暂停，并在当前终端打开 QEMU 监视器：

```sh
qemu-system-aarch64 -machine virt -cpu cortex-a53 -S -display none -serial none -monitor stdio
```

> **与原书对照**　原书在这里为 RISC-V 虚拟机准备 OpenSBI 固件，后面的内核会通过它调用 SBI 服务。我们在 Arm 上先用 QEMU 监视器检查 CPU。到引导一章，再让 QEMU 加载内核，串口输出则由内核访问 `virt` 板上的 PL011 完成。

看到 `(qemu)` 提示符后，输入 `info registers`。找到程序计数器 `PC`、通用寄存器 `X0`（QEMU 显示为 `X00`）和栈指针 `SP`，然后输入 `quit` 返回终端。后面调试启动代码时，我们还会用这个方法查看 CPU 执行到了哪里。[QEMU virt 开发板说明](https://www.qemu.org/docs/master/system/arm/virt.html)

::: details 查看运行结果（监视器会话节选）
```text
QEMU 11.1.2 monitor - type 'help' for more information
(qemu) info registers

CPU#0
 PC=0000000000000000 X00=0000000000000000 X01=0000000000000000
...
X29=0000000000000000 X30=0000000000000000  SP=0000000000000000
PSTATE=00000000400003c5 -Z-- EL1h    FPU disabled
(qemu) quit
```
:::

现在你已经生成了第一段 AArch64 机器码，也能查看虚拟 CPU 的状态。要让它执行我们写的函数，还需要给内核安排入口地址和栈，这些工作留到引导一章。
