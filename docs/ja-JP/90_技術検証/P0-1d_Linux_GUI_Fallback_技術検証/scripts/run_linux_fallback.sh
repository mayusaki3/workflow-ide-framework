#!/usr/bin/env bash

# P0-1d Linux GUI fallback 起動 script
#
# 目的:
# - Hyper-V Ubuntu Desktop fallback
# - software renderer fallback
# - x11 fallback
# - Linux GUI 安定化

set -euo pipefail

export LIBGL_ALWAYS_SOFTWARE=1
export WINIT_UNIX_BACKEND=x11

SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

cd "$SCRIPT_DIR"

cargo run
