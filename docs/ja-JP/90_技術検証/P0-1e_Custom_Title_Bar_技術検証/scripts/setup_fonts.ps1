# P0-1e font setup script
#
# Purpose:
# - Used by P0-1e Custom Title Bar validation.
# - Places the font asset under the P0-1e validation project.
#
# Notes:
# - P0-1e is a standalone validation.
# - Run before cargo run.

$ErrorActionPreference = "Stop"

$ScriptRoot = Split-Path -Parent $PSScriptRoot
$FontDir = Join-Path $ScriptRoot "assets/fonts/default"

New-Item -ItemType Directory -Force -Path $FontDir | Out-Null

$FontPath = Join-Path $FontDir "NotoSansCJKjp-Regular.otf"

$HostName = "raw.githubusercontent.com"
$RepoPath = "notofonts/noto-cjk/main/Sans/OTF/Japanese/NotoSansCJKjp-Regular.otf"
$FontUrl = "https://$HostName/$RepoPath"

Invoke-WebRequest `
    -Uri $FontUrl `
    -OutFile $FontPath

$FontSize = (Get-Item $FontPath).Length

if ($FontSize -lt 100000) {
    throw "Invalid font download: file too small ($FontSize bytes)"
}

Write-Host "Font downloaded: $FontPath"
Write-Host "Font size: $FontSize bytes"