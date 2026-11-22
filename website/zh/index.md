---
title: 简介
---

# 用 Rust 写一个运行在 AArch64 上的小型操作系统

这本教程改编自 Seiya Nuta 的《Operating System in 1,000 Lines》。原书用 C 为 32 位 RISC-V 实现了一个小型操作系统。我们会沿着原书的章节顺序，用 Rust 重新实现这个系统，并让它在 QEMU 模拟的 AArch64（64 位 Arm）机器上运行。

## 原书与参考代码

原书从内核启动讲起，逐步加入进程、虚拟内存、系统调用和文件系统。你可以对照原书阅读每一章，也可以直接查看对应的 C 实现。

- [原书英文版](https://1000os.seiya.me/en/)
- [原书简体中文版](https://1000os.seiya.me/zh/)，由 YianAndCode 翻译
- [原书仓库与 C 参考实现](https://github.com/nuta/operating-system-in-1000-lines)，其中的 `kernel.c` 是内核源码，`run.sh` 展示了如何编译并启动它

原书正文采用 CC BY 4.0 许可，示例代码采用 MIT 许可。详见[原书许可说明](https://github.com/nuta/operating-system-in-1000-lines/blob/main/LICENCE.md)。

## 我们要移植什么

第一个里程碑是让内核打印一行 `Hello, World!`。在 Rust 这一侧，我们会用 `#![no_std]` 编写裸机程序，让内核在没有宿主操作系统和标准输出的情况下运行。在 AArch64 这一侧，我们会编写启动代码，设置栈指针，把控制权交给 Rust，再通过 QEMU 虚拟机的串口输出这行文字。

同样的分工会贯穿后面的章节。例如，讲到分页时，我们仍然要解决原书里的问题：怎样把虚拟地址映射到物理内存？我们会用 Rust 表示页表项，并按 AArch64 的页表格式填写它们。

本书正在逐章改写。后续章节中仍可看到原书的 C 代码和 RISC-V 命令；对应的 Rust 实现和 AArch64 运行步骤会随移植进度更新。你可以在[本书的 GitHub 仓库](https://github.com/ChuanyuXue/operating-system-in-1000-lines)查看进展。
