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

curl -L \
  -o "$FONT_DIR/NotoSansJP-Regular.ttf" \
  https://github.com/notofonts/noto-cjk/raw/main/Sans/TTF/Japanese/NotoSansJP-Regular.ttf

printf 'Font downloaded: %s\n' "$FONT_DIR/NotoSansJP-Regular.ttf"
