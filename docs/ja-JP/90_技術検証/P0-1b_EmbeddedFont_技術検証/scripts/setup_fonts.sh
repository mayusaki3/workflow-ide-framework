#!/usr/bin/env bash

# P0-1b EmbeddedFont 技術検証
#
# Linux font setup script
#
# 目的:
# - Noto Sans CJK download
# - local assets/fonts/default へ配置
# - 技術検証再現性向上

set -euo pipefail

VALIDATION_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

FONT_DIR="$VALIDATION_ROOT/assets/fonts/default"

FONT_FILE="$FONT_DIR/NotoSansCJK-Regular.ttc"

DOWNLOAD_URL="https://github.com/notofonts/noto-cjk/raw/main/Sans/OTC/NotoSansCJK-Regular.ttc"

mkdir -p "$FONT_DIR"

curl -L "$DOWNLOAD_URL" -o "$FONT_FILE"

echo "Complete"
echo "$FONT_FILE"
