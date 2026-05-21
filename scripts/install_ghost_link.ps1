# Utah Browser — Ghost-Link Sovereign Engine Installer
# Establishes peripheral sensory daemon under ~/.utah_browser/ghost-link

param(
    [string]$InstallDir,
    [switch]$RegisterStartup,
    [switch]$StartNow,
    [switch]$Foreground
)

$ErrorActionPreference = 'Stop'
$RepoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$GhostSrc = Join-Path $RepoRoot 'ghost-link'

if (-not $InstallDir) {
    $InstallDir = Join-Path $env:USERPROFILE '.utah_browser\ghost-link'
}

Write-Host '[GHOST-LINK] Establishing Sovereign Input Node...' -ForegroundColor Cyan

foreach ($sub in @('logs', 'out', 'cache')) {
    New-Item -ItemType Directory -Force -Path (Join-Path $InstallDir $sub) | Out-Null
}

# Python venv + deps
$venv = Join-Path $InstallDir 'env'
if (-not (Test-Path (Join-Path $venv 'Scripts\python.exe'))) {
    Write-Host '  Creating Python venv...'
    python -m venv $venv
}
$py = Join-Path $venv 'Scripts\python.exe'
$pip = Join-Path $venv 'Scripts\pip.exe'
& $pip install --upgrade pip --quiet
& $pip install -r (Join-Path $GhostSrc 'requirements.txt') --quiet
Write-Host '  [OK] Dependencies installed' -ForegroundColor DarkGreen

$launcher = Join-Path $InstallDir 'launch_ghost_link.ps1'
@"
`$env:UTAH_VAULT = '$([IO.Path]::GetDirectoryName($InstallDir))'
`$env:GHOST_LINK_HOME = '$InstallDir'
`$env:OLLAMA_HOST = if (`$env:OLLAMA_HOST) { `$env:OLLAMA_HOST } else { 'http://127.0.0.1:11434' }
Set-Location '$GhostSrc'
& '$py' -m ghost_link @args
"@ | Set-Content $launcher -Encoding UTF8

if ($RegisterStartup) {
    $Action = New-ScheduledTaskAction -Execute $py -Argument "-m ghost_link" -WorkingDirectory $GhostSrc
    $Trigger = New-ScheduledTaskTrigger -AtLogOn
    $Settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries
    Register-ScheduledTask -Action $Action -Trigger $Trigger -Settings $Settings `
        -TaskName 'UtahBrowser_GhostLink' -Description 'Ghost-Link sensory daemon for Utah Browser' -Force
    Write-Host '  [OK] Registered scheduled task UtahBrowser_GhostLink' -ForegroundColor DarkGreen
}

if ($StartNow) {
    $env:UTAH_VAULT = Split-Path -Parent $InstallDir
    $env:GHOST_LINK_HOME = $InstallDir
    Push-Location $GhostSrc
    if ($Foreground) {
        & $py -m ghost_link --verbose
    }
    else {
        Start-Process -FilePath $py -ArgumentList '-m', 'ghost_link' -WindowStyle Hidden -WorkingDirectory $GhostSrc
        Write-Host '  [OK] Daemon started in background' -ForegroundColor DarkGreen
    }
    Pop-Location
}

Write-Host '[GHOST-LINK] Sovereign Engine manifested at:' -ForegroundColor Green
Write-Host "  $InstallDir"
Write-Host "  Launch: $launcher"
Write-Host "  Telemetry: $(Join-Path $InstallDir 'logs\telemetry.log')"
