# Ensures Ollama + Qdrant are ready before launching Utah Browser.
param([string]$ProjectRoot = $null)

$ErrorActionPreference = 'Stop'
if (-not $ProjectRoot) {
    $ProjectRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
}

$KernelDir = Join-Path $ProjectRoot 'scripts\kernel'
. (Join-Path $KernelDir 'Read-UtahConfig.ps1')
. (Join-Path $KernelDir 'Health.ps1')

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
