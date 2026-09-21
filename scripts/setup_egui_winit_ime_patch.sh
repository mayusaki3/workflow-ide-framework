#!/usr/bin/env bash
set -euo pipefail
VERSION="${1:-0.34.3}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REGISTRY_ROOT="${CARGO_HOME:-$HOME/.cargo}/registry/src"
SOURCE="$(find "$REGISTRY_ROOT" -mindepth 2 -maxdepth 2 -type d -name "egui-winit-$VERSION" -print -quit)"
if [[ -z "$SOURCE" ]]; then echo "egui-winit $VERSION was not found. Run cargo fetch first." >&2; exit 1; fi
PATCH_ROOT="$REPO_ROOT/patches/egui-winit-$VERSION"
rm -rf "$PATCH_ROOT"
cp -a "$SOURCE" "$PATCH_ROOT"
python3 - "$PATCH_ROOT/src/lib.rs" <<'PY'
from pathlib import Path
import sys
p=Path(sys.argv[1]); s=p.read_text()
old="let ime_rect_px = pixels_per_point * ime.rect;"
new="""// WFIDE temporary workaround:
// IME candidate windows must follow the primary text cursor, not
// the full TextEdit widget rectangle.
            let ime_rect_px = pixels_per_point * ime.cursor_rect;"""
if s.count(old) != 1:
    raise SystemExit(f"Expected exactly one upstream IME assignment, found {s.count(old)}. Refusing to patch.")
p.write_text(s.replace(old,new))
PY
echo "Prepared WFIDE egui-winit $VERSION IME workaround:"
echo "  $PATCH_ROOT"
echo "  ime.rect -> ime.cursor_rect"
