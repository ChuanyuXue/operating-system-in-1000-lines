---
title: Rust 核心库
---

# Rust 核心库

如果把 `hello.txt` 写进一个 12 字节的缓冲区，文件名占几个字节？剩下的位置该放什么？读回来时，又怎样判断它和另一个文件名是否相同？本章沿用第 05 章的内核和串口，用这个例子学习裸机 Rust 里的基本类型、内存操作和字符串操作。

> **与原书对照**　原书在 `common.h` 和 `common.c` 中定义类型，并实现 `memcpy`、`memset`、`strcpy` 和 `strcmp`。我们的内核使用 `#![no_std]`，仍然可以使用 Rust 内置的整数类型，以及 `core` 为切片和字符串提供的方法。上一章用过的 `core::fmt` 也来自这个库。[Rust 核心库文档](https://doc.rust-lang.org/core/)

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

## 内存操作

准备一个 12 字节的字段来存放文件名。约定很简单：文件名能放下就复制进去，剩余位置填零，并返回实际复制的字节数。如果放不下，整个字段保持原样。

创建 `kernel/src/name.rs`：

<<< @/../examples/ch06-core/kernel/src/name.rs

`&mut [u8]` 是可写的字节切片，带有长度。`split_at_mut_checked` 先检查容量，再把字段分成两段。第一段和源文件名的字节数相同，可以用 `copy_from_slice` 复制，这一步对应原书的 `memcpy`。第二段用 `fill(0)` 填零，对应 `memset`。文件名过长时，第一行的 `?` 直接返回 `None`，两次写操作都还没开始。[切片方法文档](https://doc.rust-lang.org/core/primitive.slice.html#method.split_at_mut_checked)

在 `kernel/src/main.rs` 的 `mod console;` 旁添加 `mod name;`，再把这三行接在刚才的对齐示例后：

```rust
let mut field = [0xa5; 12];
let len = name::copy_name(&mut field, "hello.txt").unwrap();
writeln!(console, "name field: {field:02x?}").unwrap();
```

初始值特意选了 `0xa5`，这样在串口输出里能看清哪些位置被改写了。`hello.txt` 占 9 字节，因此前 9 字节会变为文件名，最后 3 字节会变为零。如果文件名恰好占满 12 字节，填充部分就是空切片。这种写法适用于已经取得可写切片的普通内存。第 04 章的启动代码则要在使用静态变量之前，按链接器给出的地址清零整个 `.bss`。

## 字符串操作

原书的 `strcpy` 会复制到第一个 `'\0'` 为止，并把这个结束符一起写入目标。Rust 的 `&str` 携带 UTF-8 字节长度。刚才保存的 `len` 就是有效文本的长度，因此即使字段恰好写满，也能读回完整文件名。

```rust
let stored = core::str::from_utf8(&field[..len]).unwrap();
writeln!(console, "{stored} == hello.txt: {}", stored == "hello.txt").unwrap();
writeln!(
    console,
    "{stored} cmp meow.txt: {:?}",
    stored.cmp("meow.txt")
)
.unwrap();
```

`from_utf8` 检查这段字节是否是有效的 UTF-8。本例中的字节刚从一个 `&str` 完整复制过来，所以可以 `unwrap()`。比较两个名字是否相等直接用 `==`。如果还关心顺序，`cmp` 会返回 `Less`、`Equal` 或 `Greater`，对应 C 的 `strcmp` 返回负数、零或正数。这里第一个不同的字节是 `h` 和 `m`，因此结果是 `Less`。[Rust 字符串文档](https://doc.rust-lang.org/core/primitive.str.html)

现在从工程目录启动 QEMU。想直接运行完整示例，可以进入教程仓库的 `examples/ch06-core/`。[本章完整源码](https://github.com/ChuanyuXue/operating-system-in-1000-lines/tree/main/examples/ch06-core)

```sh
sh run.sh
```

::: details 查看运行结果（客机串口）
```text
Hello World!
1 + 2 = 3, 1234abcd
usize: 64 bits
0x1234 -> 0x2000, aligned: true
name field: [68, 65, 6c, 6c, 6f, 2e, 74, 78, 74, 00, 00, 00]
hello.txt == hello.txt: true
hello.txt cmp meow.txt: Less
```
:::

可以把文件名改成 `"你好.txt"` 再运行一次。两个汉字各占 3 个 UTF-8 字节，加上 `.txt`，总共 10 字节。观察字段末尾还剩几个零，以及和 `"hello.txt"` 比较的结果。

::: details 查看运行结果（仅将传给 copy_name 的文件名改为“你好.txt”，串口节选）
```text
name field: [e4, bd, a0, e5, a5, bd, 2e, 74, 78, 74, 00, 00]
你好.txt == hello.txt: false
你好.txt cmp meow.txt: Greater
```
:::
