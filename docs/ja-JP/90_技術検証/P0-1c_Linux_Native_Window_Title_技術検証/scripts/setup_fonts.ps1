# P0-1c/P0-1d font setup script
#
# Purpose:
# - Used by P0-1d fallback validation.
# - Places the font asset under the P0-1c validation project.
#
# Notes:
# - Do not run this for the normal P0-1c standalone validation.
# - Use this only for P0-1d validation.

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
