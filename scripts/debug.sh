#!/bin/bash
set -e
SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)
cd "${SCRIPT_DIR}/.."

./scripts/run-in-docker.sh make

qemu-system-x86_64 -s -fda os-image.bin &
gdb \
  -ex "target extended-remote localhost:1234" \
  -ex "symbol-file kernel.elf"
