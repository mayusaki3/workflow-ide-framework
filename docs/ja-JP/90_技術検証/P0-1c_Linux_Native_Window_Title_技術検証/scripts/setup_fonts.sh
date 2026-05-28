#!/usr/bin/env bash

# P0-1c/P0-1d font setup script
#
# 目的:
# - P0-1d fallback 検証用
# - P0-1c + EmbeddedFont 構成確認
#
# 注意:
# - P0-1c 単独検証では通常実行しない
# - P0-1d 検証時のみ利用する

set -euo pipefail

FONT_DIR="$(cd "$(dirname "$0")/.." && pwd)/assets/fonts/default"

mkdir -p "$FONT_DIR"

FONT_PATH="$FONT_DIR/NotoSansJP-Regular.ttf"
FONT_URL="https://raw.githubusercontent.com/notofonts/noto-cjk/main/Sans/TTF/Japanese/NotoSansJP-Regular.ttf"

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
