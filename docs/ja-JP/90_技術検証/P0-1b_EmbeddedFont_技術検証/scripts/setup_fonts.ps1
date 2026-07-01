# P0-1b EmbeddedFont 技術検証
#
# Windows font setup script
#
# 目的:
# - Noto Sans CJK download
# - local assets/fonts/default へ配置
# - 技術検証再現性向上

$ErrorActionPreference = "Stop"

$ValidationRoot = Split-Path -Parent $PSScriptRoot

$FontDir = Join-Path $ValidationRoot "assets/fonts/default"

$FontFile = Join-Path $FontDir "NotoSansCJK-Regular.ttc"

$DownloadUrl = "https://github.com/notofonts/noto-cjk/raw/main/Sans/OTC/NotoSansCJK-Regular.ttc"

Write-Host "Create font directory..."
New-Item -ItemType Directory -Force -Path $FontDir | Out-Null

Write-Host "Download font..."
Invoke-WebRequest `
    -Uri $DownloadUrl `
    -OutFile $FontFile

Write-Host "Complete"
Write-Host $FontFile
