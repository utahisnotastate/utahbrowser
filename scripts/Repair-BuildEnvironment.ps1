# Utah Browser - repair Cargo cache and suggest Windows exclusions for blocked builds.
# Run from repo root: .\scripts\Repair-BuildEnvironment.ps1
# Use -AddExclusions only if you are allowed to change Windows Security settings.

param(
    [switch]$AddExclusions,
    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$KernelDir = Join-Path $Root 'scripts\kernel'
. (Join-Path $KernelDir 'Build.ps1')

Write-Host 'Utah Browser - Build Environment Repair' -ForegroundColor Green
Write-Host "Project: $Root"

Repair-UtahCargoCache -Root $Root

if ($AddExclusions) {
    $cargoHome = Join-Path $env:USERPROFILE '.cargo'
    $paths = @($Root, $cargoHome)
    Write-Host 'Adding Windows Defender exclusions (requires Administrator)...' -ForegroundColor Cyan
    foreach ($p in $paths) {
        if (Test-Path $p) {
            try {
                Add-MpPreference -ExclusionPath $p -ErrorAction Stop
                Write-Host "  Added exclusion: $p" -ForegroundColor DarkGreen
            }
            catch {
                Write-Warning "Could not add exclusion for $p : $($_.Exception.Message)"
                Write-Host '  Run this script in an elevated PowerShell, or add exclusions manually.' -ForegroundColor Yellow
            }
        }
    }
}
else {
    Write-Host ''
    Write-Host 'Tip: re-run with -AddExclusions in Administrator PowerShell to whitelist:' -ForegroundColor Yellow
    Write-Host "  $Root"
    Write-Host "  $(Join-Path $env:USERPROFILE '.cargo')"
}

if (-not $SkipBuild) {
    Write-Host ''
    Write-Host 'Testing release build...' -ForegroundColor Cyan
    $ok = Invoke-UtahCargoBuild -Root $Root -SkipRepair
    if ($ok) {
        Write-Host 'SUCCESS: target\release\utah-browser.exe is ready.' -ForegroundColor Green
        exit 0
    }
    exit 1
}

Write-Host 'Repair complete (build skipped). Run: cargo build --release' -ForegroundColor Green
