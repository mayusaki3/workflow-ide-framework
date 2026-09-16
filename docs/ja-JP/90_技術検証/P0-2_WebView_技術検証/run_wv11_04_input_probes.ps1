param(
    [string]$CargoTargetDir = "D:\cargo-target\workflow-ide-p0-2"
)

$ErrorActionPreference = "Stop"
$env:CARGO_TARGET_DIR = $CargoTargetDir

$probes = @(
    [pscustomobject]@{
        Name = "Pointer move"
        Bin = "cef_input_move_probe"
        Check = "Move the pointer across all four Browser Surface zones. Confirm hover/coordinates follow the pointer."
    },
    [pscustomobject]@{
        Name = "Pointer click"
        Bin = "cef_input_button_probe"
        Check = "Click the Browser Surface test button/target. Confirm the browser-side click result changes."
    },
    [pscustomobject]@{
        Name = "Wheel"
        Bin = "cef_input_wheel_probe"
        Check = "Place the pointer over Browser Surface and use the wheel. Confirm the browser page scrolls."
    },
    [pscustomobject]@{
        Name = "Keyboard"
        Bin = "cef_input_keyboard_probe"
        Check = "Click Browser Surface first, then press and release keys. Confirm browser-side keydown/keyup changes."
    },
    [pscustomobject]@{
        Name = "Focus / Windows IME"
        Bin = "cef_ime_ui_thread_child_probe"
        Check = "Click the input, enable Japanese IME, type 'nihon', convert with Space, commit with Enter. Confirm candidate UI is positioned by the input and text commits."
    }
)

Write-Host "WV-11-04 integrated input verification" -ForegroundColor Cyan
Write-Host "Cargo target: $env:CARGO_TARGET_DIR"
Write-Host "Each probe is interactive. Perform the displayed check, then close its window to continue."
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

Write-Host "All WV-11-04 probe processes completed normally." -ForegroundColor Green
Write-Host "Record visual pass/fail separately; a zero process exit code alone does not prove the interactive criterion."