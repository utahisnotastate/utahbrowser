# Utah Unified Reality Manifold — Nexus Orchestrator installer
# Seeds ProgramData vault, venv, optional startup task, launches stack.

param(
    [switch]$UseProgramData,
    [switch]$RegisterStartup,
    [switch]$StartNow,
    [switch]$StartGhostLink,
    [switch]$Foreground
)

$ErrorActionPreference = 'Stop'
$RepoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$UrmSrc = Join-Path $RepoRoot 'urm'

if ($UseProgramData) {
    $env:URM_USE_PROGRAMDATA = '1'
    $UrmHome = Join-Path $env:PROGRAMDATA 'Utah_URM'
}
else {
    $UrmHome = Join-Path $env:USERPROFILE '.utah_browser\urm'
}

$env:UTAH_VAULT = if ($env:UTAH_VAULT) { $env:UTAH_VAULT } else { Join-Path $env:USERPROFILE '.utah_browser' }
$env:URM_HOME = $UrmHome
$env:UTAH_REPO = $RepoRoot

Write-Host '[URM] Manifesting Utah Unified Reality Manifold...' -ForegroundColor Cyan

foreach ($sub in @('logs', 'snapshots', 'nexus', 'mutagenesis', 'swarm', 'licensing')) {
    New-Item -ItemType Directory -Force -Path (Join-Path $UrmHome $sub) | Out-Null
}

$venv = Join-Path $UrmHome 'env'
if (-not (Test-Path (Join-Path $venv 'Scripts\python.exe'))) {
    python -m venv $venv
}
$py = Join-Path $venv 'Scripts\python.exe'
& $py -m pip install -q -r (Join-Path $UrmSrc 'requirements.txt')

$launcher = Join-Path $UrmHome 'launch_nexus.ps1'
@"
`$env:UTAH_VAULT = '$env:UTAH_VAULT'
`$env:URM_HOME = '$UrmHome'
`$env:UTAH_REPO = '$RepoRoot'
Set-Location '$UrmSrc'
& '$py' nexus_orchestrator.py @args
"@ | Set-Content $launcher -Encoding UTF8

if ($RegisterStartup) {
    $Action = New-ScheduledTaskAction -Execute $py -Argument 'nexus_orchestrator.py' -WorkingDirectory $UrmSrc
    $Trigger = New-ScheduledTaskTrigger -AtLogOn
    Register-ScheduledTask -Action $Action -Trigger $Trigger -TaskName 'UtahBrowser_URM_Nexus' -Force
    Write-Host '  [OK] Scheduled task UtahBrowser_URM_Nexus' -ForegroundColor DarkGreen
}

if ($StartGhostLink) {
    $ghost = Join-Path $RepoRoot 'scripts\install_ghost_link.ps1'
    if (Test-Path $ghost) {
        & $ghost -StartNow
    }
}

if ($StartNow) {
    Push-Location $UrmSrc
    if ($Foreground) {
        & $py nexus_orchestrator.py
    }
    else {
        Start-Process -FilePath $py -ArgumentList 'nexus_orchestrator.py' -WorkingDirectory $UrmSrc -WindowStyle Hidden
        Write-Host '  [OK] Nexus Orchestrator started' -ForegroundColor DarkGreen
    }
    Pop-Location
}

$distLauncher = Join-Path $RepoRoot 'scripts\Launch-URM.ps1'
Copy-Item $MyInvocation.MyCommand.Path (Join-Path $UrmHome 'install_urm.ps1') -Force -ErrorAction SilentlyContinue

Write-Host '[URM] Sovereign manifold ready.' -ForegroundColor Green
Write-Host "  Vault:   $UrmHome"
Write-Host "  Nexus:   $launcher"
Write-Host "  Launch:  .\scripts\Launch-URM.ps1"
