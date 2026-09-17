#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/cargo-target/workflow-ide-p0-2}"

echo "WV-12 Linux VM GPU Surface verification"
echo "CARGO_TARGET_DIR=$CARGO_TARGET_DIR"
echo

run_probe() {
  local bin="$1"
  local label="$2"

  echo "============================================================"
  echo "$label"
  echo "cargo run --no-default-features --bin $bin"
  echo "============================================================"
  cargo run --no-default-features --bin "$bin"
  echo
}

run_probe wgpu_dock_surface_probe "WV-12-02: GPU Surface Dock display"
run_probe wgpu_surface_lifecycle_probe "WV-12-03: Resize / Visibility / Lifecycle"
run_probe wgpu_surface_input_probe "WV-12-04: Input event transfer"

echo "WV-12 Linux VM probe sequence completed."
echo
echo "Acceptance:"
echo "  WV-12-02: GPU drawing / Dock display / no Native Child Window / continuous update"
echo "  WV-12-03: resize / visibility / explicit destroy-create / repeated recreation"
echo "  WV-12-04: PointerMove / PointerButton / Wheel / Keyboard-Text / Focus"
echo
echo "Close each probe window after confirming its behavior; the next probe starts afterward."
