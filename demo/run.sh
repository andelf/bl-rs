#!/bin/bash


~/opt/riscv64-unknown-elf-toolchain-10.2.0/bin/riscv64-unknown-elf-objdump -Cd ../target/riscv32imac-unknown-none-elf/debug/demo



 ~/opt/riscv64-unknown-elf-toolchain-10.2.0/bin/riscv64-unknown-elf-objcopy -O binary ../target/riscv32imac-unknown-none-elf/debug/demo firmware.bin
