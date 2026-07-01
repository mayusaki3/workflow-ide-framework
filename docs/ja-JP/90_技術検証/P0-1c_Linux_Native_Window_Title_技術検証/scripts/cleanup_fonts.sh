#!/usr/bin/env bash

# P0-1c font cleanup script
#
# 目的:
# - P0-1c 単独検証条件へ戻す
# - EmbeddedFont asset を削除する

set -euo pipefail

FONT_DIR="$(cd "$(dirname "$0")/.." && pwd)/assets/fonts/default"

if [ -d "$FONT_DIR" ]; then
    rm -f "$FONT_DIR"/*
fi

echo "Embedded fonts cleaned"
