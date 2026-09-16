param(
    [string]$CargoTargetDir = "D:\cargo-target\workflow-ide-p0-2"
)

$ErrorActionPreference = "Stop"
$env:CARGO_TARGET_DIR = $CargoTargetDir

# WV-11-05 verifies Browser Surface behavior inside one Dock panel.
# The framework does not assume one tab per panel, so Dock relocation and tab
# switching are not Browser Surface acceptance criteria here.
$probes = @(
    [pscustomobject]@{
        Name = "Dock display"
        Bin = "cef_dock_probe"
        Check = "Confirm Browser Surface is rendered inside the Dock panel and remains stable while the application window is moved."
    },
    [pscustomobject]@{
        Name = "Dock resize follow"
        Bin = "cef_dock_resize_probe"
        Check = "Resize the application window repeatedly. Confirm Browser Surface follows the Dock panel size and Requested / Paint / Texture sizes update without losing the surface."
    },
    [pscustomobject]@{
        Name = "Dock basic input"
        Bin = "cef_input_keyboard_probe"
        Check = "Click the Browser Surface to focus it, then confirm A / Enter / Arrow keydown and keyup are reflected in the browser."
    }
)

Write-Host "WV-11-05 Windows Dock verification" -ForegroundColor Cyan
Write-Host "Cargo target: $env:CARGO_TARGET_DIR"
Write-Host "Each probe is interactive. Perform the displayed check, then close its window to continue."
Write-Host "Dock relocation and tab switching are outside WV-11-05 because Browser Surface is verified as content of one Dock panel, not as one-tab-per-panel UI."
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
Write-Host "Confirm separately: Dock display / resize follow / basic input."
