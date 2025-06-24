#!/bin/bash
set -e
SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)
cd "${SCRIPT_DIR}/.."

if ! command -v i686-elf-gcc; then
  ./scripts/run-cross-compiler.sh ./scripts/build.sh
  exit 0
fi

mkdir -p build
i686-elf-as src/boot.s -o build/boot.o
i686-elf-gcc -c src/kernel.c -o build/kernel.o -std=gnu99 -ffreestanding -O2 -Wall -Wextra
i686-elf-gcc -T src/linker.ld -o build/myos.bin -ffreestanding -O2 -nostdlib build/boot.o build/kernel.o -lgcc
grub-file --is-x86-multiboot build/myos.bin

mkdir -p build/isodir/boot/grub
cp build/myos.bin build/isodir/boot/myos.bin
cp src/grub.cfg build/isodir/boot/grub/grub.cfg
grub-mkrescue -o build/myos.iso build/isodir

echo "complete!"
