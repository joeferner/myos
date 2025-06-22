#!/bin/bash
set -e
SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)
cd "${SCRIPT_DIR}/.."

mkdir -p build

# ./scripts/run-in-docker.sh nasm -f bin src/main.asm -o build/main.bin
# ./scripts/run-in-docker.sh i386-elf-gcc -ffreestanding -c src/function.c -o build/function.o
# ./scripts/run-in-docker.sh i386-elf-ld -o build/function.bin -Ttext 0x0 --oformat binary build/function.o
./scripts/run-in-docker.sh make

qemu-system-x86_64 -fda build/os-image.bin
