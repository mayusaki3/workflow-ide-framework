#!/usr/bin/env bash
set -euo pipefail

# CEF + GUI dependencies require substantially more than a small tmpfs.
# Keep the default target under HOME while still allowing callers to override it.
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/cargo-target/workflow-ide-p0-2}"
export CARGO_TARGET_DIR

if [[ "$(uname -s)" != "Linux" ]]; then
    echo "WV-11-06 Linux verification must be run on Linux." >&2
    exit 2
fi

if [[ -z "${DISPLAY:-}" && -z "${WAYLAND_DISPLAY:-}" ]]; then
    echo "No DISPLAY or WAYLAND_DISPLAY is set." >&2
    echo "Run this from the Linux VM graphical desktop/session; WV-11-06 verifies Dock rendering visually." >&2
    exit 2
fi

mkdir -p "${CARGO_TARGET_DIR}"

echo "WV-11-06 Linux CEF OSR verification"
echo "Cargo target: ${CARGO_TARGET_DIR}"
echo "Session: DISPLAY=${DISPLAY:-<unset>} WAYLAND_DISPLAY=${WAYLAND_DISPLAY:-<unset>}"
echo ""
echo "This verifies the same CEF OSR -> egui Texture -> Dock route used on Windows."
echo "It does not use GTK/WebKitGTK Host Window embedding."
echo "Visual pass/fail must be recorded separately; process exit code alone is not a pass."
echo ""

run_probe() {
    local index="$1"
    local total="$2"
    local name="$3"
    local bin="$4"
    local check="$5"

    echo "[${index}/${total}] ${name}"
    echo "  Binary: cargo run --bin ${bin}"
    echo "  Check : ${check}"
    echo ""

    cargo run --bin "${bin}"

    echo "Completed: ${name}"
    echo ""
}

run_probe 1 3 \
    "CEF OSR minimum route" \
    "cef_buffer_probe" \
    "Confirm the process starts, receives CEF OSR Paint, and exits normally. No independent native browser window should appear."

run_probe 2 3 \
    "Browser Surface Dock display" \
    "cef_dock_probe" \
    "Confirm Browser Surface is rendered inside the egui Dock and remains visible while moving the application window."

run_probe 3 3 \
    "Browser Surface resize follow" \
    "cef_dock_resize_probe" \
    "Resize the application window repeatedly. Confirm Browser Surface follows the Dock size and Paint/Texture continue updating without losing the surface."

echo "All WV-11-06 Linux probe processes completed normally."
echo "Record separately: CEF OSR / Dock display / resize follow."
echo "If a probe fails, preserve the complete terminal log plus: uname -a, /etc/os-release, XDG_SESSION_TYPE, DISPLAY, WAYLAND_DISPLAY, and VM software/version."
