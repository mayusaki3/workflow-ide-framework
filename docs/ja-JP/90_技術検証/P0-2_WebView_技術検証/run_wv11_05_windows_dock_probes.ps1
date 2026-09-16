param(
    [string]$CargoTargetDir = "D:\cargo-target\workflow-ide-p0-2"
)

$ErrorActionPreference = "Stop"
$env:CARGO_TARGET_DIR = $CargoTargetDir

$probes = @(
    [pscustomobject]@{
        Name = "Dock display and move"
        Bin = "cef_dock_probe"
        Check = "Confirm Browser Surface is rendered inside the Dock. Drag the Browser Surface Dock tab and move/re-dock it where egui_dock permits; confirm the Browser Surface remains rendered. Also move the application window and confirm rendering remains stable."
    },
    [pscustomobject]@{
        Name = "Dock resize follow"
        Bin = "cef_dock_resize_probe"
        Check = "Resize the application window repeatedly. Confirm Browser Surface follows the Dock size and Requested / Paint / Texture sizes update without losing the surface."
    },
    [pscustomobject]@{
        Name = "Dock tab switch and basic input"
        Bin = "cef_input_keyboard_probe"
        Check = "Use the Browser Surface Dock tab normally, click the Browser Surface to focus it, and confirm A / Enter / Arrow keydown and keyup are reflected in the browser. If the Dock layout exposes another tab during the check, switch away and back and confirm the Browser Surface resumes normally."
    }
)

Write-Host "WV-11-05 Windows Dock verification" -ForegroundColor Cyan
Write-Host "Cargo target: $env:CARGO_TARGET_DIR"
Write-Host "Each probe is interactive. Perform the displayed check, then close its window to continue."
Write-Host "Visual pass/fail must be recorded separately; process exit code alone is not a pass."
Write-Host ""

$index = 0
foreach ($probe in $probes) {
    $index++
    Write-Host "[$index/$($probes.Count)] $($probe.Name)" -ForegroundColor Yellow
    Write-Host "  Binary: cargo run --bin $($probe.Bin)"
    Write-Host "  Check : $($probe.Check)"
    Write-Host ""

    & cargo run --bin $probe.Bin
    if ($LASTEXITCODE -ne 0) {
        throw "Probe failed: $($probe.Bin) (exit=$LASTEXITCODE)"
    }

    Write-Host "Completed: $($probe.Name)" -ForegroundColor Green
    Write-Host ""
}

Write-Host "All WV-11-05 probe processes completed normally." -ForegroundColor Green
Write-Host "Confirm separately: Dock display / Dock move / resize follow / tab behavior / basic input."