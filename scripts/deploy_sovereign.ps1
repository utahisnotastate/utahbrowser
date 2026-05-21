# Utah Sovereign Node — Windows deployment (Zero-Click Kernel + Ghost-Link + dist)

param(
    [string]$KnowledgePath = $env:UTAH_KNOWLEDGE_PATH,
    [switch]$SkipBuild,
    [switch]$InstallGhostLink,
    [switch]$InstallURM,
    [switch]$RegisterGhostStartup
)

$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)

Write-Host "`n==> MANIFESTING SOVEREIGN NODE (Windows)" -ForegroundColor Cyan

$installArgs = @{
    KnowledgePath = $KnowledgePath
}
if ($SkipBuild) { $installArgs['SkipBuild'] = $true }
if ($InstallGhostLink) { $installArgs['InstallGhostLink'] = $true }
if ($InstallURM) { $installArgs['InstallURM'] = $true }

& (Join-Path $Root 'scripts\install.ps1') @installArgs

if ($InstallGhostLink -or $RegisterGhostStartup) {
    $ghostArgs = @{ StartNow = $true }
    if ($RegisterGhostStartup) { $ghostArgs['RegisterStartup'] = $true }
    & (Join-Path $Root 'scripts\install_ghost_link.ps1') @ghostArgs
}

& (Join-Path $Root 'scripts\Build-Standalone.ps1')

Write-Host "`n==> NODE ONLINE" -ForegroundColor Green
Write-Host "  Launch:  $Root\dist\UtahBrowser.cmd" -ForegroundColor Cyan
Write-Host "  Logs:    $env:APPDATA\UtahBrowser\logs\browser.log" -ForegroundColor DarkGray
Write-Host "  Vault:   $env:APPDATA\UtahBrowser\vault\inject\queue.jsonl" -ForegroundColor DarkGray
