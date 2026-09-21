#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-0.34.3}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REGISTRY_ROOT="${CARGO_HOME:-$HOME/.cargo}/registry/src"

SOURCE="$(find "$REGISTRY_ROOT" -mindepth 2 -maxdepth 2 -type d -name "egui-winit-$VERSION" -print -quit)"
if [[ -z "$SOURCE" ]]; then
  echo "egui-winit $VERSION was not found in Cargo registry. Run: cargo fetch" >&2
  exit 1
fi

PATCH_ROOT="$REPO_ROOT/patches/egui-winit-$VERSION"
rm -rf "$PATCH_ROOT"
cp -a "$SOURCE" "$PATCH_ROOT"

LIB="$PATCH_ROOT/src/lib.rs"
python3 - "$LIB" <<'PY'
from pathlib import Path
import sys
p = Path(sys.argv[1])
s = p.read_text()
old = "let ime_rect_px = pixels_per_point * ime.rect;"
new = """let ime_rect_px = pixels_per_point * ime.cursor_rect;
            eprintln!(
                "WFIDE IME PATCH cursor_rect=({:.1},{:.1},{:.1},{:.1})",
                ime.cursor_rect.min.x,
                ime.cursor_rect.min.y,
                ime.cursor_rect.width(),
                ime.cursor_rect.height()
            );"""
count = s.count(old)
if count != 1:
    raise SystemExit(f"Expected exactly one IME rect assignment, found {count}. Source was not modified.")
p.write_text(s.replace(old, new))
PY

echo "Prepared full egui-winit $VERSION diagnostic patch:"
echo "  $PATCH_ROOT"
echo "Changed only IME candidate area source: ime.rect -> ime.cursor_rect"
