#!/bin/bash
set -e
SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)
cd "${SCRIPT_DIR}/.."

docker build -f ./scripts/Dockerfile.cross-compiler --tag os-from-scratch-cross-compiler .
