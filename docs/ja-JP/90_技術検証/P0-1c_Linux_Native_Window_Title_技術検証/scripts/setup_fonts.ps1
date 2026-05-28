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

Invoke-WebRequest `
    -Uri "https://github.com/notofonts/noto-cjk/raw/main/Sans/TTF/Japanese/NotoSansJP-Regular.ttf" `
    -OutFile $FontPath

Write-Host "Font downloaded: $FontPath"
