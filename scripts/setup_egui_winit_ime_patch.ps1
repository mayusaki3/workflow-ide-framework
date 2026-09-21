param(
    [string]$Version = "0.34.3"
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$registryRoot = Join-Path $env:USERPROFILE ".cargo\registry\src"
$source = Get-ChildItem -Path $registryRoot -Directory |
    ForEach-Object { Join-Path $_.FullName "egui-winit-$Version" } |
    Where-Object { Test-Path $_ } |
    Select-Object -First 1

if (-not $source) {
    throw "egui-winit $Version was not found in Cargo registry. Run cargo fetch first."
}

$patchRoot = Join-Path $repoRoot "patches\egui-winit-$Version"
if (Test-Path $patchRoot) {
    Remove-Item -Recurse -Force $patchRoot
}
Copy-Item -Recurse -Force $source $patchRoot

$lib = Join-Path $patchRoot "src\lib.rs"
$text = Get-Content -Raw -Encoding UTF8 $lib
$old = "let ime_rect_px = pixels_per_point * ime.rect;"
$new = @"
let ime_rect_px = pixels_per_point * ime.cursor_rect;
            eprintln!(
                "WFIDE IME PATCH cursor_rect=({:.1},{:.1},{:.1},{:.1})",
                ime.cursor_rect.min.x,
                ime.cursor_rect.min.y,
                ime.cursor_rect.width(),
                ime.cursor_rect.height()
            );
"@

$count = ([regex]::Matches($text, [regex]::Escape($old))).Count
if ($count -ne 1) {
    throw "Expected exactly one IME rect assignment, found $count. Source was not modified."
}

$text = $text.Replace($old, $new.TrimEnd())
Set-Content -Path $lib -Value $text -Encoding UTF8 -NoNewline

Write-Host "Prepared full egui-winit $Version diagnostic patch:"
Write-Host "  $patchRoot"
Write-Host "Changed only IME candidate area source: ime.rect -> ime.cursor_rect"
