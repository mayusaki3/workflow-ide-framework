# P0-1c font cleanup script
#
# 目的:
# - P0-1c 単独検証条件へ戻す
# - EmbeddedFont asset を削除する

$ScriptRoot = Split-Path -Parent $PSScriptRoot
$FontDir = Join-Path $ScriptRoot "assets/fonts/default"

if (Test-Path $FontDir) {
    Remove-Item "$FontDir/*" -Force -ErrorAction SilentlyContinue
}

Write-Host "Embedded fonts cleaned"
