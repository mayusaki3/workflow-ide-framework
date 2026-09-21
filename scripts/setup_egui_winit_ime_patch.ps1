param([string]$Version = "0.34.3")
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$registryRoot = if ($env:CARGO_HOME) { Join-Path $env:CARGO_HOME "registry\src" } else { Join-Path $env:USERPROFILE ".cargo\registry\src" }
$source = Get-ChildItem -Path $registryRoot -Directory | ForEach-Object { Join-Path $_.FullName "egui-winit-$Version" } | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $source) { throw "egui-winit $Version was not found. Run cargo fetch first." }

$patchRoot = Join-Path $repoRoot "patches\egui-winit-$Version"
if (Test-Path $patchRoot) { Remove-Item -Recurse -Force $patchRoot }
Copy-Item -Recurse -Force $source $patchRoot

$lib = Join-Path $patchRoot "src\lib.rs"
$text = Get-Content -Raw -Encoding UTF8 $lib
$old = "let ime_rect_px = pixels_per_point * ime.rect;"
$new = @"
// WFIDE temporary workaround:
// IME candidate windows must follow the primary text cursor, not
// the full TextEdit widget rectangle.
            let ime_rect_px = pixels_per_point * ime.cursor_rect;
"@
$count = ([regex]::Matches($text, [regex]::Escape($old))).Count
if ($count -ne 1) { throw "Expected exactly one upstream IME assignment, found $count. Refusing to patch." }
$text = $text.Replace($old, $new.TrimEnd())
Set-Content -Path $lib -Value $text -Encoding UTF8 -NoNewline

Write-Host "Prepared WFIDE egui-winit $Version IME workaround:"
Write-Host "  $patchRoot"
Write-Host "  ime.rect -> ime.cursor_rect"
