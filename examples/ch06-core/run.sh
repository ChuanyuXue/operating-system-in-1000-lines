#!/bin/sh
set -eu
cd "$(dirname "$0")"
cargo +stable build --release -p kernel
exec qemu-system-aarch64 -machine virt -cpu cortex-a53 \
    -nographic -serial mon:stdio \
    -kernel target/aarch64-unknown-none-softfloat/release/kernel
