#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/cargo-target/workflow-ide-p0-2}"
echo "WV-12-03: click Hide then Show; confirm visibility log"
cargo run --no-default-features --bin wgpu_surface_lifecycle_probe
echo "WV-12-04: focus GPU Surface and use mouse wheel both directions"
cargo run --no-default-features --bin wgpu_surface_input_probe
echo "WV-12 supplemental probes completed"
