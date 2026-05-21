# Ensures Ollama + Qdrant are ready before launching Utah Browser.
param([string]$ProjectRoot = $null)

$ErrorActionPreference = 'Stop'
if (-not $ProjectRoot) {
    $ProjectRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
}

$KernelDir = Join-Path $ProjectRoot 'scripts\kernel'
$readCfg = Join-Path $KernelDir 'Read-UtahConfig.ps1'
$health = Join-Path $KernelDir 'Health.ps1'
if (-not (Test-Path $readCfg) -or -not (Test-Path $health)) {
    Write-Host "  [ERROR] Missing scripts\kernel (Read-UtahConfig.ps1 / Health.ps1)." -ForegroundColor Red
    Write-Host '  Re-run: .\scripts\Build-Standalone.ps1  (or copy scripts\kernel into dist\scripts\kernel)' -ForegroundColor Yellow
    exit 1
}
. $readCfg
. $health
if (-not (Get-Command Ensure-QdrantReady -ErrorAction SilentlyContinue)) {
    Write-Host '  [ERROR] Health.ps1 is outdated (Ensure-QdrantReady missing).' -ForegroundColor Red
    Write-Host '  Re-run: .\scripts\Build-Standalone.ps1  (copies scripts\kernel into dist)' -ForegroundColor Yellow
    exit 1
}

$cfg = Read-UtahConfig -ConfigPath (Join-Path $ProjectRoot 'config\default.toml')

Write-Host 'Utah Browser - ensuring local services...' -ForegroundColor Cyan

$ollama = Test-OllamaHealth -HostUrl $cfg.OllamaHost
if ($ollama.Ok) {
    Write-Host "  [OK] $($ollama.Message)" -ForegroundColor DarkGreen
}
else {
    Write-Host "  [WARN] $($ollama.Message)" -ForegroundColor Yellow
    Write-Host '  Start Ollama Desktop or run: ollama serve' -ForegroundColor Yellow
}

$qdrant = Ensure-QdrantReady -BaseUrl $cfg.QdrantUrl -ProjectRoot $ProjectRoot
if ($qdrant.Ok) {
    $col = Ensure-QdrantCollection -BaseUrl $cfg.QdrantUrl -Collection $cfg.QdrantCollection -VectorSize $cfg.QdrantVectorSize
    if ($col.Ok) {
        Write-Host "  [OK] $($col.Message)" -ForegroundColor DarkGreen
    }
}

if (-not $qdrant.Ok) {
    Write-Host '  Truth Engine (ingest/verify) requires Qdrant.' -ForegroundColor Red
    exit 1
}

exit 0
