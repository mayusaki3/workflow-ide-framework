#!/usr/bin/env bash

# P0-1c Linux fallback 起動 script
#
# 目的:
# - Hyper-V Ubuntu Desktop fallback
# - software renderer fallback
# - x11 fallback
# - Linux GUI backend 安定化
#
# 注意:
# - native title 日本語問題の改善ではなく
#   Linux GUI backend 起動安定化を目的とする

set -euo pipefail

export LIBGL_ALWAYS_SOFTWARE=1
export WINIT_UNIX_BACKEND=x11

SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

cd "$SCRIPT_DIR"

cargo run
