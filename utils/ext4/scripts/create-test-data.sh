#!/bin/bash
set -e
SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)
cd "${SCRIPT_DIR}/.."

if (( $EUID != 0 )); then
  echo "Please run as root."
  exit 1
fi

mkdir -p test-data/mnt
umount test-data/mnt || echo "ok"
rm -rf test-data/simple.ext4 || echo "ok"
dd if=/dev/zero of=test-data/simple.ext4 bs=4k count=1024
mkfs.ext4 -L ext4-test test-data/simple.ext4
tune2fs -c0 -i0 test-data/simple.ext4
mount test-data/simple.ext4 test-data/mnt
echo "Hello from root directory!" > test-data/mnt/root.txt
mkdir test-data/mnt/dir1
echo "Hello from dir1!" > test-data/mnt/dir1/test.txt
umount test-data/mnt
chmod a+r test-data/simple.ext4

echo "complete!"
