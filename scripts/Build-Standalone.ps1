# Build portable dist/ folder with double-click "Utah Browser.exe" launcher (no CLI required).

param(
    [switch]$SkipHealth
)

$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$KernelDir = Join-Path $Root 'scripts\kernel'
. (Join-Path $KernelDir 'Build.ps1')

Write-Host "`n==> Building Utah Browser standalone (release)" -ForegroundColor Green
Push-Location $Root
try {
    $code = Invoke-Cargo -ArgumentList @('build', '--release', '--bin', 'utah-browser', '--bin', 'utah-launch')
    if ($code -ne 0) {
        throw "cargo build failed (exit $code). Try .\scripts\Repair-BuildEnvironment.ps1"
    }

    $OutDir = Join-Path $Root 'dist'
    New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
    New-Item -ItemType Directory -Force -Path (Join-Path $OutDir 'scripts') | Out-Null

    Copy-Item (Join-Path $Root 'target\release\utah-browser.exe') (Join-Path $OutDir 'utah-browser.exe') -Force
    Copy-Item (Join-Path $Root 'target\release\utah-launch.exe') (Join-Path $OutDir 'Utah Browser.exe') -Force
    Copy-Item (Join-Path $Root 'target\release\utah-launch.exe') (Join-Path $OutDir 'UtahBrowser.exe') -Force
    New-Item -ItemType Directory -Force -Path (Join-Path $OutDir 'logs') | Out-Null

    @'
@echo off
cd /d "%~dp0"
start "" "%~dp0utah-browser.exe"
'@ | Set-Content (Join-Path $OutDir 'UtahBrowser.cmd') -Encoding ASCII

    @'
# Launch Utah Browser (use this if PowerShell breaks on spaces in "Utah Browser.exe")
$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $here
$env:UTAH_BROWSER_HOME = $here
& (Join-Path $here 'utah-browser.exe')
'@ | Set-Content (Join-Path $OutDir 'Launch-UtahBrowser.ps1') -Encoding UTF8
    Copy-Item (Join-Path $Root 'config') (Join-Path $OutDir 'config') -Recurse -Force
    Copy-Item (Join-Path $Root 'assets') (Join-Path $OutDir 'assets') -Recurse -Force
    Copy-Item (Join-Path $Root 'scripts\kernel') (Join-Path $OutDir 'scripts\kernel') -Recurse -Force
    Copy-Item (Join-Path $Root 'scripts\Ensure-Qdrant.ps1') (Join-Path $OutDir 'scripts\Ensure-Qdrant.ps1') -Force
    Copy-Item (Join-Path $Root 'scripts\Ensure-Services.ps1') (Join-Path $OutDir 'scripts\Ensure-Services.ps1') -Force

    @"
# Utah Browser — how to launch (PowerShell needs quotes if the name has a space!)

  Double-click:  UtahBrowser.cmd   OR   UtahBrowser.exe
  PowerShell:    & ".\Utah Browser.exe"
                 .\Launch-UtahBrowser.ps1
                 .\utah-browser.exe

Logs (primary + mirror):
  %APPDATA%\UtahBrowser\logs\browser.log
  logs\browser.log  (mirror in this folder)
  %TEMP%\utah-browser.log

Recovery / safe mode:  %APPDATA%\UtahBrowser\recovery.json

Default home page: https://www.cia.gov (Web view).
"@ | Set-Content (Join-Path $OutDir 'README.txt') -Encoding UTF8

    Write-Host "`n==> Standalone build ready" -ForegroundColor Green
    Write-Host "  Launch:  $OutDir\UtahBrowser.cmd  (recommended)" -ForegroundColor Cyan
    Write-Host "  Or:      $OutDir\utah-browser.exe" -ForegroundColor DarkGray
    Write-Host "  PS:      & `"$OutDir\Utah Browser.exe`"" -ForegroundColor DarkGray
}
finally {
    Pop-Location
}
