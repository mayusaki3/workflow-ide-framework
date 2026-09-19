# WFIDE Framework font setup
$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$FontDir = Join-Path $RepoRoot "assets/fonts/default"
$FontFile = Join-Path $FontDir "NotoSansCJK-Regular.ttc"
$DownloadUrl = "https://github.com/notofonts/noto-cjk/raw/main/Sans/OTC/NotoSansCJK-Regular.ttc"

New-Item -ItemType Directory -Force -Path $FontDir | Out-Null
Write-Host "Download Framework default font..."
Invoke-WebRequest -Uri $DownloadUrl -OutFile $FontFile
Write-Host "Complete"
Write-Host $FontFile
