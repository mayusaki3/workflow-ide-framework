#!/usr/bin/env bash

# P0-1e font setup script
#
# 目的:
# - P0-1e Custom Title Bar 検証用
# - P0-1e 独立検証用フォント配置
#
# 注意:
# - P0-1e は独立検証
# - cargo run 前に実行する

set -euo pipefail

FONT_DIR="$(cd "$(dirname "$0")/.." && pwd)/assets/fonts/default"

mkdir -p "$FONT_DIR"

FONT_PATH="$FONT_DIR/NotoSansCJKjp-Regular.otf"
HOST_NAME="raw.githubusercontent.com"
REPO_PATH="notofonts/noto-cjk/main/Sans/OTF/Japanese/NotoSansCJKjp-Regular.otf"
FONT_URL="https://${HOST_NAME}/${REPO_PATH}"

curl -L \
  -o "$FONT_PATH" \
  "$FONT_URL"

FONT_SIZE=$(stat -c%s "$FONT_PATH")

if [ "$FONT_SIZE" -lt 100000 ]; then
    echo "Invalid font download: file too small (${FONT_SIZE} bytes)" >&2
    exit 1
fi

printf 'Font downloaded: %s\n' "$FONT_PATH"
printf 'Font size: %s bytes\n' "$FONT_SIZE"