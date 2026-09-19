#!/usr/bin/env bash
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FONT_DIR="$REPO_ROOT/assets/fonts/default"
FONT_FILE="$FONT_DIR/NotoSansCJK-Regular.ttc"
DOWNLOAD_URL="https://github.com/notofonts/noto-cjk/raw/main/Sans/OTC/NotoSansCJK-Regular.ttc"

mkdir -p "$FONT_DIR"
echo "Download Framework default font..."
curl -L "$DOWNLOAD_URL" -o "$FONT_FILE"
echo "Complete"
echo "$FONT_FILE"
