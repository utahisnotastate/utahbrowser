# Launch Utah Unified Reality Manifold — Nexus + Browser + optional Ghost-Link

$ErrorActionPreference = 'Continue'
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$env:UTAH_VAULT = if ($env:UTAH_VAULT) { $env:UTAH_VAULT } else { Join-Path $env:USERPROFILE '.utah_browser' }

$nexusInstaller = Join-Path $Root 'scripts\install_urm.ps1'
if (Test-Path $nexusInstaller) {
    & powershell -NoProfile -ExecutionPolicy Bypass -File $nexusInstaller -StartNow -StartGhostLink
}

$dist = Join-Path $Root 'dist'
$launcher = Join-Path $dist 'Launch-UtahBrowser.ps1'
$exe = Join-Path $dist 'utah-browser.exe'

if (Test-Path $launcher) {
    & powershell -NoProfile -ExecutionPolicy Bypass -File $launcher
}
elseif (Test-Path $exe) {
    & $exe
}
else {
    $built = Join-Path $Root 'target\release\utah-browser.exe'
    if (Test-Path $built) {
        & $built
    }
    else {
        Write-Host 'Build first: .\scripts\install.ps1' -ForegroundColor Yellow
    }
}
