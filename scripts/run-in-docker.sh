#!/bin/bash
set -e
SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)
cd "${SCRIPT_DIR}/.."

docker run --rm -it \
  -v "$(pwd):/app" \
  -u $(id -u ${USER}):$(id -g ${USER}) \
  --net=host \
  os-from-scratch \
  "$@"
