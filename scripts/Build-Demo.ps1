# Build stable DEMO package — safe mode, simple launch, demo config.

param(
    [switch]$Zip,
    [switch]$SkipCompile
)

$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$KernelDir = Join-Path $Root 'scripts\kernel'
. (Join-Path $KernelDir 'Build.ps1')

Write-Host "`n==> Utah Browser DEMO build (stable profile)" -ForegroundColor Cyan
Push-Location $Root
try {
    if (-not $SkipCompile) {
        $code = Invoke-Cargo -ArgumentList @('build', '--release', '--bin', 'utah-browser', '--bin', 'utah-launch')
        if ($code -ne 0) { throw "cargo build failed (exit $code)" }
    }

    $OutDir = Join-Path $Root 'dist'
    $ReleaseDir = Join-Path $Root 'release'
    New-Item -ItemType Directory -Force -Path $OutDir, (Join-Path $OutDir 'logs'), (Join-Path $OutDir 'scripts') | Out-Null

    Copy-Item (Join-Path $Root 'target\release\utah-browser.exe') (Join-Path $OutDir 'utah-browser.exe') -Force
    Copy-Item (Join-Path $Root 'target\release\utah-launch.exe') (Join-Path $OutDir 'Utah Browser.exe') -Force
    Copy-Item (Join-Path $Root 'target\release\utah-launch.exe') (Join-Path $OutDir 'UtahBrowser.exe') -Force

    New-Item -ItemType Directory -Force -Path (Join-Path $OutDir 'config') | Out-Null
    Copy-Item (Join-Path $Root 'config\demo.toml') (Join-Path $OutDir 'config\default.toml') -Force
    Copy-Item (Join-Path $Root 'assets') (Join-Path $OutDir 'assets') -Recurse -Force
    Copy-Item (Join-Path $Root 'scripts\kernel') (Join-Path $OutDir 'scripts\kernel') -Recurse -Force
    Copy-Item (Join-Path $Root 'scripts\Ensure-Services.ps1') (Join-Path $OutDir 'scripts\Ensure-Services.ps1') -Force

    @'
@echo off
cd /d "%~dp0"
set UTAH_BROWSER_HOME=%~dp0
set UTAH_DEMO_MODE=1
start "" "%~dp0utah-browser.exe"
'@ | Set-Content (Join-Path $OutDir 'UtahBrowser.cmd') -Encoding ASCII

    @'
# Demo launcher — safe mode, no PowerShell space issues
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$env:UTAH_BROWSER_HOME = $here
$env:UTAH_DEMO_MODE = '1'
Set-Location $here
& (Join-Path $here 'utah-browser.exe')
'@ | Set-Content (Join-Path $OutDir 'Launch-UtahBrowser.ps1') -Encoding UTF8

    @'
Utah Browser — DEMO (stable)

Launch:  Double-click UtahBrowser.cmd
         (Do not type "Utah Browser.exe" in PowerShell without quotes.)

Mode:    UTAH_DEMO_MODE=1 → single WebView (safe mode) for reliable demos.

Logs:    %APPDATA%\UtahBrowser\logs\browser.log
         logs\browser.log (mirror in this folder)

Truth:   Optional — start Ollama, run scripts\Ensure-Services.ps1

Docs:    https://github.com/utahisnotastate/utahbrowser/blob/main/docs/DEMO.md
'@ | Set-Content (Join-Path $OutDir 'DEMO.txt') -Encoding UTF8

    Copy-Item (Join-Path $Root 'docs\DEMO.md') (Join-Path $OutDir 'DEMO.md') -Force

    Write-Host "`n==> Demo package ready: $OutDir" -ForegroundColor Green
    Write-Host "  Launch:  $OutDir\UtahBrowser.cmd" -ForegroundColor Cyan

    if ($Zip) {
        New-Item -ItemType Directory -Force -Path $ReleaseDir | Out-Null
        $zipPath = Join-Path $ReleaseDir 'UtahBrowser-Demo-win64.zip'
        if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
        Compress-Archive -Path (Join-Path $OutDir '*') -DestinationPath $zipPath -Force
        Write-Host "  Zip:     $zipPath" -ForegroundColor Cyan
        Write-Host "  Upload this file to GitHub Releases (tag v1.0-demo)" -ForegroundColor DarkGray
    }
}
finally {
    Pop-Location
}
