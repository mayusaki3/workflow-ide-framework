#!/usr/bin/env bash

# P0-1d Linux GUI fallback 起動 script
#
# 目的:
# - Hyper-V Ubuntu Desktop fallback
# - software renderer fallback
# - x11 fallback
# - Linux GUI 安定化
#
# 注意:
# - P0-1d は fallback 運用方針の検証であり、独自 Cargo project は持たない
# - 実行対象は P0-1c の native window title 検証 project とする

set -euo pipefail

export LIBGL_ALWAYS_SOFTWARE=1
export WINIT_UNIX_BACKEND=x11

VALIDATION_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TARGET_DIR="$VALIDATION_ROOT/P0-1c_Linux_Native_Window_Title_技術検証"

cd "$TARGET_DIR"

cargo run
