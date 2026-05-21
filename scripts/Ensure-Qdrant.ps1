# Ensures Qdrant is up (auto-starts Docker). Called by launcher and optionally from the app.
param(
    [string]$ProjectRoot = $null,
    [switch]$NoAutoStart
)

$ErrorActionPreference = 'Stop'
if (-not $ProjectRoot) {
    $ProjectRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
}

$KernelDir = Join-Path $ProjectRoot 'scripts\kernel'
. (Join-Path $KernelDir 'Read-UtahConfig.ps1')
. (Join-Path $KernelDir 'Health.ps1')

$cfg = Read-UtahConfig -ConfigPath (Join-Path $ProjectRoot 'config\default.toml')
$qdrant = Ensure-QdrantReady -BaseUrl $cfg.QdrantUrl -NoAutoStart:$NoAutoStart

if ($qdrant.Ok) {
    $col = Ensure-QdrantCollection -BaseUrl $cfg.QdrantUrl -Collection $cfg.QdrantCollection -VectorSize $cfg.QdrantVectorSize
    if (-not $col.Ok) {
        Write-Host "WARN: $($col.Message)" -ForegroundColor Yellow
    }
    exit 0
}

exit 1
