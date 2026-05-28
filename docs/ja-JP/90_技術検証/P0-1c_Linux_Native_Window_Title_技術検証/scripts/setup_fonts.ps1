# P0-1c/P0-1d font setup script
#
# 目的:
# - P0-1d fallback 検証用
# - P0-1c + EmbeddedFont 構成確認
#
# 注意:
# - P0-1c 単独検証では通常実行しない
# - P0-1d 検証時のみ利用する

$ScriptRoot = Split-Path -Parent $PSScriptRoot
$FontDir = Join-Path $ScriptRoot "assets/fonts/default"

New-Item -ItemType Directory -Force -Path $FontDir | Out-Null

$FontPath = Join-Path $FontDir "NotoSansJP-Regular.ttf"

$FontUrl = "https://raw.githubusercontent.com/notofonts/noto-cjk/main/Sans/TTF/Japanese/NotoSansJP-Regular.ttf"

Invoke-WebRequest `
    -Uri $FontUrl `
    -OutFile $FontPath

$FontSize = (Get-Item $FontPath).Length

if ($FontSize -lt 100000) {
    throw "Invalid font download: file too small ($FontSize bytes)"
}

Write-Host "Font downloaded: $FontPath"
Write-Host "Font size: $FontSize bytes"
