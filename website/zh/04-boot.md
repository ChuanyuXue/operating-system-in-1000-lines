---
title: 引导
---

# 引导加载内核

第 01 章里，`rustc` 生成了一个目标文件，QEMU 也能启动一颗虚拟 CPU。这一章把两件事连起来：把代码链接成一个可执行的内核，让 CPU 从我们指定的入口开始，设置栈指针，再跳进 Rust 函数。即使还没有串口输出，也可以用寄存器验证内核确实走完了这几步。

## 谁把控制权交给内核？

固件可以初始化机器、装入内核，还可以在内核启动后提供服务。原书使用的 RISC-V 固件 OpenSBI 就承担了这些工作。我们在 QEMU 的 Arm `virt` 板上可以从更短的一条路径开始：把一个裸机 ELF 可执行文件交给 `-kernel`。QEMU 根据 ELF 的程序头把各个可加载段放入客机内存，并用 ELF 头里的入口地址初始化 PC。

> **与原书对照**　原书先观察 OpenSBI 启动，再由 OpenSBI 把控制权交给内核。这里由 QEMU 直接加载 ELF。下面的地址和启动步骤针对这台虚拟机；真实 Arm 开发板的固件有自己的交接约定。[QEMU Arm virt 裸机启动说明](https://www.qemu.org/docs/master/system/arm/virt.html#hardware-configuration-information-for-bare-metal-programming)

## 先看启动时的 CPU

先沿用第 01 章的命令，观察还没有装入内核时的状态。`-S` 让 CPU 暂停，`-monitor stdio` 把当前终端接到 QEMU 监视器：

```sh
qemu-system-aarch64 -machine virt -cpu cortex-a53 -S -display none -serial none -monitor stdio
```

输入 `info registers`，记下 PC 和 SP，再输入 `quit`。稍后用同一条监视器命令观察装入内核后的变化。

::: details 查看运行结果（寄存器节选）
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

## 链接器脚本

从这里开始，按第 03 章的布局建立一个独立的 Cargo 工程。可以在一个空目录里创建下文标注的文件，也可以从教程仓库根目录进入 `examples/ch04-boot/` 使用完整示例。后面的宿主机命令都在这个工程目录里运行。[本章完整源码](https://github.com/ChuanyuXue/operating-system-in-1000-lines/tree/main/examples/ch04-boot)

先看 `kernel/kernel.ld`。它回答两个问题：指令和数据分别放在哪里？CPU 从哪个符号开始执行？

<<< @/../examples/ch04-boot/kernel/kernel.ld{text}

`virt` 板的 RAM 从 `0x40000000` 开始。QEMU 在裸机启动时把描述硬件的设备树放在 RAM 起始处。本例选择从 `0x40400000` 布置内核，给起始区域留出 4 MiB。`ENTRY(boot)` 将符号 `boot` 的地址写入 ELF 的入口字段。等会儿可以在监视器里看到，内核映像的起点和第一条指令的地址可以不同。

| 区域 | 放什么 |
| --- | --- |
| `.text` | 可执行指令。`KEEP(*(.text.boot))` 保留我们的启动代码，并把它排在这个段的开头。 |
| `.rodata`、`.data` | 只读常量，以及带初始值的可写数据。 |
| `.bss` | 初值为零的静态数据。两个符号标出需要清零的地址区间。 |
| `.stack` | 向下增长的内核栈。起点按 16 字节对齐，预留 128 KiB，高地址端标为 `__stack_top`。 |

脚本里的 `.` 表示链接器当前分配到的地址。`NOLOAD` 让栈和零初始化数据占用客机内存空间，无需在 ELF 文件里存放同样大小的一串零。

## 最小内核

将下面的代码放在 `kernel/src/main.rs`。先读 `boot` 和 `kernel_main`：前者是第一段汇编，后者是我们要到达的第一个 Rust 函数。

<<< @/../examples/ch04-boot/kernel/src/main.rs

### 内核入口点

`ldr x0, =__stack_top` 把栈顶地址放进 `x0`，`mov sp, x0` 设置栈指针。栈向低地址增长，所以这里从栈的高地址端开始。普通 Rust 函数可能在执行第一行代码之前就使用栈，因此这两条指令要放在汇编入口。第三条指令 `b kernel_main` 跳入 Rust。`kernel_main` 的返回类型是 `!`，启动代码无需保存返回地址。

### 怎样定义启动代码

`#![no_std]` 让内核使用 `core`，`#![no_main]` 让我们自己安排入口。`global_asm!` 把汇编直接放进目标文件。`.section .text.boot, "ax"` 选中可装入内存、可执行的代码段，`.global boot` 将入口符号交给链接器。`#[unsafe(no_mangle)]` 保留 `kernel_main` 这个符号名，使汇编里的跳转能找到它。

Rust 还要求裸机程序提供 `#[panic_handler]`。暂时没有输出设备，示例在 panic 时原地循环。正常路径进入 `kernel_main` 后，先调用 `clear_bss()`，再循环执行 `wfe`（等待事件），被唤醒后继续等待。

### 取链接器符号的地址

`__bss_start` 和 `__bss_end` 由链接器定义。Rust 通过 `unsafe extern "C"` 声明这两个外部符号，再用 `&raw mut __bss_start` 和 `&raw mut __bss_end` 取得它们的地址。取地址不会读取内存，所以即使结束符号指向段末尾之后，也可以用它来标记循环边界。

### 初始化 .bss

`clear_bss()` 遍历半开区间 `start..end`。每轮用 `write_volatile(0)` 向当前地址写一个零字节，让编译器保留这次写入。最后写入的是地址 `end - 1`。如果区间为空，循环执行零次，本章示例就属于这种情况。

这里的 `unsafe` 依赖两个启动条件：链接器划出的区间是可写 RAM，而且此时只有一个核心在运行，还没有代码开始使用这些静态变量。因此 `kernel_main` 把清零放在第一步。

> **与原书对照**　原书的汇编入口负责设置栈并跳到 C 函数，随后由 C 代码清零 `.bss`。这里保留相同的分工，由 Rust 完成清零。

## 运行起来！

工作区根目录的 `Cargo.toml` 目前只包含内核。release 配置使用 `panic = "abort"`，无需为 panic 展开调用栈。

<<< @/../examples/ch04-boot/Cargo.toml

在 `.cargo/config.toml` 中选用第 01 章安装的目标：

<<< @/../examples/ch04-boot/.cargo/config.toml

在 `kernel/Cargo.toml` 中给内核包命名：

<<< @/../examples/ch04-boot/kernel/Cargo.toml

Cargo 在宿主机上执行 `kernel/build.rs`。它把链接器脚本的完整路径传给内核的链接步骤，并生成 `kernel.map`，记录符号对应的地址。修改 `kernel.ld` 后，Cargo 会重新运行这一步。

<<< @/../examples/ch04-boot/kernel/build.rs

最后在工程根目录创建 `run.sh`。Cargo 生成的可执行文件叫 `kernel`，QEMU 可以直接识别它的 ELF 格式。

<<< @/../examples/ch04-boot/run.sh{sh}

从工程目录运行：

```sh
sh run.sh
```

::: details 查看运行结果（构建和启动节选）
```text
   Compiling kernel v0.1.0 (…/ch04-boot/kernel)
    Finished `release` profile [optimized] target(s) in 0.37s
QEMU 11.1.2 monitor - type 'help' for more information
(qemu)
```
:::

保留这个终端。现在 ELF 已经装进内存，`-S` 让 CPU 仍然停在第一条指令之前。

## 你的第一次内核调试

先在监视器里输入 `info registers`，再输入 `xp /3i $pc`。第二条命令从当前 PC 对应的物理地址反汇编三条指令。本章还没有启用分页。

::: details 查看运行结果（执行第一条指令之前）
```text
(qemu) info registers
 PC=0000000040400048 X00=0000000000000000 X01=0000000000000000
...
X29=0000000000000000 X30=0000000000000000  SP=0000000000000000
(qemu) xp /3i $pc
0x40400048:  58000080  ldr      x0, #0x40400058
0x4040004c:  9100001f  mov      sp, x0
0x40400050:  14000004  b        #0x40400060
```
:::

这次 PC 已经指向 `boot`。反汇编里第一条 `ldr` 从附近的常量池取得栈顶地址。此时 `sp` 还是零，因为第二条指令还没有执行。

输入 `cont` 让客机运行，稍等片刻再输入 `stop`。再次查看寄存器，并用 `xp /1i $pc` 看看这次停在哪里：

::: details 查看运行结果（执行启动代码之后）
```text
(qemu) cont
(qemu) stop
(qemu) info registers
 PC=0000000040400088 X00=0000000040420090 X01=0000000000000000
...
X29=0000000000000000 X30=0000000000000000  SP=0000000040420090
PSTATE=00000000600003c5 -ZC- EL1h    FPU disabled
(qemu) xp /1i $pc
0x40400088:  17ffffff  b        #0x40400084
```
:::

这些地址对应哪段源码？在监视器输入 `quit` 回到宿主机，查看同一次构建生成的链接映射：

```sh
grep -E ' (boot|kernel_main|__stack_top = \.)$' kernel.map
```

::: details 查看运行结果（链接器分配的地址）
```text
        40400048         40400048        0     1                 boot
        40400060         40400060       2c     1                 kernel_main
        40420090         40420090        0     1         __stack_top = .
```
:::

第一列是符号地址，第三列是大小（十六进制）。这次构建中，`kernel_main` 从 `0x40400060` 开始，大小是 `0x2c`（44 字节）。暂停时的 PC 是 `0x40400088`，落在函数内部，当前指令正跳回循环开头。再看 SP，它与 `__stack_top` 的地址 `0x40420090` 一致。编译器版本变化可能改变具体地址，你可以用自己的 `kernel.map` 做这两项核对。

现在你已经在 AArch64 上启动了第一个 Rust 内核。下一章我们会让它通过串口输出一行文字。
