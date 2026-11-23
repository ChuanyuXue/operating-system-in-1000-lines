---
title: 用 Rust 实现 libc 基础函数
---

# 用 Rust 实现 libc 基础函数

第 05 章已经能把字节送到串口。这一章来写原书的四个基础函数：用 `memcpy` 复制字节，用 `memset` 填充内存，用 `strcpy` 复制以零结尾的字符串，用 `strcmp` 比较它们。观察同一块缓冲区，就能看出两类复制的区别：复制三个普通字节之后，谁来写字符串末尾的零？

> **与原书对照**　原书把这些函数放进 `common.c`，供内核和用户程序共用。这里对应的是一个名为 `common` 的 `#![no_std]` 库。四个函数保留原名，并采用 Rust 调用接口：切片携带缓冲区长度，`&CStr` 表示以零结尾的字节串。

## 基本类型

| 原书中的类型 | Rust 中的写法 | 在内核里的用途 |
| --- | --- | --- |
| `bool` | `bool` | 表示真或假，例如发送队列是否已满。 |
| `uint8_t`、`uint32_t`、`uint64_t` | `u8`、`u32`、`u64` | 明确指定宽度。一个串口字符用 `u8`，上一章的 PL011 寄存器按 `u32` 访问。 |
| `size_t`、`uintptr_t` | `usize` | 保存切片长度、下标或指针地址。本书的 AArch64 目标上，`usize` 是 64 位。 |

原书还为物理地址和虚拟地址分别命名为 `paddr_t`、`vaddr_t`。地址的**数值**可以放进 `usize`，地址的**含义**由使用位置决定。第 05 章串口基址 `0x0900_0000` 就是物理地址。等到分页一章，我们还要处理由页表翻译的虚拟地址。

先试一个页对齐的例子。把下面的代码放在 `kernel_main` 中，接在上一章的两个 `writeln!` 之后、等待循环之前。后面两节的代码也依次加在这里。

```rust
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
```

一页是 `0x1000` 字节。地址 `0x1234` 落在页内，从这里向上取最近的页边界会得到 `0x2000`。如果起点已经对齐，结果就是起点本身。`checked_next_multiple_of` 在结果溢出或除数为零时返回 `None`，本例中的两个常量可以确定结果存在。检查对齐用 `is_multiple_of`。原书另一个常用宏 `offsetof` 在 Rust 中对应 `core::mem::offset_of!(类型, 字段)`，结果是字段距结构体起点的字节数。[usize 文档](https://doc.rust-lang.org/core/primitive.usize.html#method.checked_next_multiple_of) · [offset_of! 文档](https://doc.rust-lang.org/core/mem/macro.offset_of.html)

从第 05 章的工程继续，在根目录的 `Cargo.toml` 中把 `members` 改成 `["kernel", "common"]`，在 `kernel/Cargo.toml` 中追加：

```toml
[dependencies]
common = { path = "../common" }
```

创建 `common/Cargo.toml` 和 `common/src/lib.rs`。后面两个小节会分别补上 `memory.rs` 和 `string.rs`。

<<< @/../examples/ch06-core/common/Cargo.toml

<<< @/../examples/ch06-core/common/src/lib.rs

## 内存操作

原书的 `memcpy(dst, src, n)` 用两个指针逐字节复制。`n` 在 Rust 版本里由切片长度给出：调用方传入要复制的源区间和同样长的目标区间。`memset` 只需要一个目标区间和一个填充值。把这两个函数写进 `common/src/memory.rs`：

<<< @/../examples/ch06-core/common/src/memory.rs

`memcpy` 先核对长度，再把每个源字节写到目标的对应位置。两个切片分别以共享和独占方式借用，安全 Rust 调用方需要给出互不重叠的区间。`memset` 每次写入同一个值。两个循环都能处理空切片。

在 `kernel/src/main.rs` 顶部引入 `use common::{memcpy, memset, strcmp, strcpy};` 和 `use core::ffi::CStr;`。接着在刚才的对齐示例后写：

```rust
let mut buf = [0; 6];
memset(&mut buf, 0xa5);
memcpy(&mut buf[..3], b"abc");
writeln!(console, "memcpy + memset: {buf:02x?}").unwrap();
```

填充后六个字节都是 `a5`。复制后，前三个变成 `61 62 63`，也就是 ASCII 的 `abc`。注意第四个字节仍是 `a5`：`memcpy` 只复制指定范围，包括范围内碰到的任何零字节。

> **与原书对照**　原书在这一节还把启动时清零 `.bss` 的 `memset` 移到了公共库。我们的第 04 章用链接器给出的原始地址和 volatile 写入来初始化 `.bss`。本章的 `memset` 接受已取得独占访问权的普通内存切片。启动入口继续使用 `clear_bss()`。

## 字符串操作

在 C 里，复制 `"hi"` 要写三个字节：`68 69 00`。只写前两个字符，读者就无法知道字符串在哪里结束。Rust 可以用 `c"hi"` 写出一个 `&CStr`，编译器会加上末尾的零。下面在 `common/src/string.rs` 实现 `strcpy` 和 `strcmp`：

<<< @/../examples/ch06-core/common/src/string.rs

`strcpy` 先用 `to_bytes()` 取得结束符之前的内容。目标至少要比这些内容多一个字节：能放下就用刚写好的 `memcpy` 复制正文，再亲自写入结尾的零。容量不足时返回 `None`，此时目标尚未被写入。比如 `c"hi"` 正好能放进三字节缓冲区。

`strcmp` 则要把结束符也拿来比较，因此用 `to_bytes_with_nul()`。它逐对比较 `u8`，在第一处不同的位置将两个字节扩大到 `i32` 后相减。这样 `0 - 116` 可以得到负数。如果一路比较到双方的结束符都相同，就返回零。

沿用刚才的六字节缓冲区，接着写：

```rust
strcpy(&mut buf, c"hi").unwrap();
writeln!(console, "strcpy: {buf:02x?}").unwrap();
let copied = CStr::from_bytes_until_nul(&buf).unwrap();
for other in [c"hi", c"hit", c"he"] {
    writeln!(
        console,
        "strcmp({copied:?}, {other:?}) = {}",
        strcmp(copied, other)
    )
    .unwrap();
}
```

`CStr::from_bytes_until_nul` 把缓冲区开头一直到第一个零的部分作为一个 C 字符串。后面的三个 `a5` 仍在内存里，但不参与比较。和原书一样，调用方可以用 `strcmp(a, b) == 0` 判断相等，用结果的正负判断顺序。

在工程目录运行下面的命令，也可以从教程仓库根目录进入 `examples/ch06-core/` 运行完整示例。[本章源码](https://github.com/ChuanyuXue/operating-system-in-1000-lines/tree/main/examples/ch06-core)

```sh
sh run.sh
```

::: details 查看运行结果（客机串口）
```text
Hello World!
1 + 2 = 3, 1234abcd
usize: 64 bits
0x1234 -> 0x2000, aligned: true
memcpy + memset: [61, 62, 63, a5, a5, a5]
strcpy: [68, 69, 00, a5, a5, a5]
strcmp("hi", "hi") = 0
strcmp("hi", "hit") = -116
strcmp("hi", "he") = 4
```
:::

为什么是 `-116`？前两个字节 `h` 和 `i` 都相同，第三次比较遇到左侧的结束符 `0` 和右侧的 `t`（116）。这正是前缀字符串排在较长字符串前面的原因。
