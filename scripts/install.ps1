# Utah Browser - Zero-Click Kernel
# Build, health-check local AI stack, auto-pull models, deploy dist bundle.

param(
    [string]$KnowledgePath = $env:UTAH_KNOWLEDGE_PATH,
    [switch]$SkipBuild,
    [switch]$SkipHealth,
    [switch]$SkipPull,
    [switch]$SkipQdrantStart,
    [switch]$ForcePull,
    [switch]$RepairOnly,
    [int]$HealthTimeoutSec = 8
)

$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$KernelDir = Join-Path $Root 'scripts\kernel'

. (Join-Path $KernelDir 'Read-UtahConfig.ps1')
. (Join-Path $KernelDir 'Health.ps1')
. (Join-Path $KernelDir 'Models.ps1')
. (Join-Path $KernelDir 'Build.ps1')

function Write-Step { param([string]$Msg) Write-Host "`n==> $Msg" -ForegroundColor Green }
function Write-Warn { param([string]$Msg) Write-Host "WARN: $Msg" -ForegroundColor Yellow }
function Write-Err  { param([string]$Msg) Write-Host "ERROR: $Msg" -ForegroundColor Red }

Push-Location $Root
try {
    if ($RepairOnly) {
        Write-Step 'Repair-only mode'
        Repair-UtahCargoCache -Root $Root
        if (-not (Invoke-UtahCargoBuild -Root $Root)) {
            throw 'cargo build failed after repair'
        }
        Write-Step 'Repair complete'
        return
    }

    $ConfigPath = Join-Path $Root 'config\default.toml'
    $cfg = Read-UtahConfig -ConfigPath $ConfigPath

    $report = [ordered]@{
        timestamp_utc = (Get-Date).ToUniversalTime().ToString('o')
        ollama        = $null
        qdrant        = $null
        models        = @()
        build         = $null
        dist          = $null
    }

    # --- Phase 0: Health & models ---
    if (-not $SkipHealth) {
        Write-Step 'Zero-Click Kernel - health checks'

        $ollama = Test-OllamaHealth -HostUrl $cfg.OllamaHost -TimeoutSec $HealthTimeoutSec
        $report.ollama = $ollama
        if ($ollama.Ok) {
            Write-Host "  [OK] $($ollama.Message)" -ForegroundColor DarkGreen
        }
        else {
            Write-Err $ollama.Message
            Write-Warn 'Start Ollama (ollama serve) then re-run install.ps1'
        }

        $qdrant = Test-QdrantHealth -BaseUrl $cfg.QdrantUrl -TimeoutSec $HealthTimeoutSec
        if (-not $qdrant.Ok -and -not $SkipQdrantStart) {
            Write-Warn 'Qdrant offline - attempting Docker start (utah-qdrant)...'
            $started = Start-QdrantDocker -BaseUrl $cfg.QdrantUrl
            if ($started.Ok) {
                Write-Host "  [OK] $($started.Message)" -ForegroundColor DarkCyan
                $qdrant = Test-QdrantHealth -BaseUrl $cfg.QdrantUrl -TimeoutSec $HealthTimeoutSec
            }
            else {
                Write-Warn $started.Message
            }
        }
        $report.qdrant = $qdrant
        if ($qdrant.Ok) {
            Write-Host "  [OK] $($qdrant.Message)" -ForegroundColor DarkGreen
        }
        else {
            Write-Err $qdrant.Message
        }

        if (-not $SkipPull -and $ollama.Ok) {
            Write-Step 'Model auto-pull (from config/default.toml)'
            $models = @($cfg.EmbedModel, $cfg.ChatModel) | Select-Object -Unique
            $pullResults = Ensure-OllamaModels -HostUrl $cfg.OllamaHost -Models $models -ForcePull:$ForcePull
            $report.models = $pullResults
            foreach ($r in $pullResults) {
                $color = switch ($r.Status) {
                    'present' { 'DarkGray' }
                    'pulled'  { 'DarkGreen' }
                    default   { 'Red' }
                }
                Write-Host "  $($r.Model): $($r.Status)" -ForegroundColor $color
                if ($r.Error) { Write-Err "    $($r.Error)" }
            }
        }
        elseif ($SkipPull) {
            Write-Warn 'Skipped model pull (-SkipPull)'
        }
    }
    else {
        Write-Warn 'Skipped health checks (-SkipHealth)'
    }

    # --- Phase 1: Build ---
    if (-not $SkipBuild) {
        Write-Step 'Building Utah Browser (release)'
        if (-not (Invoke-UtahCargoBuild -Root $Root)) {
            throw 'cargo build failed - see messages above or run .\scripts\Repair-BuildEnvironment.ps1'
        }
        $report.build = 'ok'
    }
    else {
        Write-Warn 'Skipped cargo build (-SkipBuild)'
        $report.build = 'skipped'
    }

    # --- Phase 2: Dist bundle ---
    Write-Step 'Packaging dist/'
    $OutDir = Join-Path $Root 'dist'
    New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

    $Exe = Join-Path $Root 'target\release\utah-browser.exe'
    if (-not (Test-Path $Exe)) {
        if ($SkipBuild) {
            Write-Warn "Skipping dist packaging (binary missing). Run without -SkipBuild first."
            Write-Step 'Health check complete (no dist bundle)'
            if (-not $SkipHealth) {
                $ollamaOk = $report.ollama -and $report.ollama.Ok
                $qdrantOk = $report.qdrant -and $report.qdrant.Ok
                if (-not ($ollamaOk -and $qdrantOk)) { exit 2 }
            }
            return
        }
        throw "Binary not found: $Exe - run without -SkipBuild or build manually"
    }

    Copy-Item $Exe (Join-Path $OutDir 'utah-browser.exe') -Force
    Copy-Item (Join-Path $Root 'config') (Join-Path $OutDir 'config') -Recurse -Force
    Copy-Item (Join-Path $Root 'assets') (Join-Path $OutDir 'assets') -Recurse -Force
    Copy-Item (Join-Path $Root 'scripts\kernel') (Join-Path $OutDir 'scripts\kernel') -Recurse -Force
    Copy-Item $MyInvocation.MyCommand.Path (Join-Path $OutDir 'install.ps1') -Force

    $healthPath = Join-Path $OutDir 'health-report.json'
    $report | ConvertTo-Json -Depth 6 | Set-Content $healthPath -Encoding UTF8

    $launcher = Join-Path $OutDir 'Launch-UtahBrowser.ps1'
    @"
# Utah Browser launcher - loads env and opens the app
`$here = Split-Path -Parent `$MyInvocation.MyCommand.Path
if (Test-Path "`$here\utah.env.ps1") { . "`$here\utah.env.ps1" }
& "`$here\utah-browser.exe"
"@ | Set-Content $launcher -Encoding UTF8

    $envLines = @(
        "# Utah Browser environment - generated by Zero-Click Kernel",
        "`$env:OLLAMA_HOST = '$($cfg.OllamaHost)'",
        "`$env:QDRANT_URL = '$($cfg.QdrantUrl)'"
    )
    if ($KnowledgePath) {
        $envLines += "`$env:UTAH_KNOWLEDGE_PATH = '$KnowledgePath'"
    }
    elseif ($cfg.KnowledgePath) {
        $envLines += "# Default knowledge path (override before launch):"
        $envLines += "# `$env:UTAH_KNOWLEDGE_PATH = '$($cfg.KnowledgePath)'"
    }
    $envLines | Set-Content (Join-Path $OutDir 'utah.env.ps1') -Encoding UTF8

    $report.dist = $OutDir

    Write-Step 'Zero-Click Kernel complete'
    Write-Host "  Dist:        $OutDir"
    Write-Host "  Health JSON: $healthPath"
    Write-Host "  Launch:      .\dist\Launch-UtahBrowser.ps1"
    Write-Host ""

    $ready = $true
    if (-not $SkipHealth) {
        $ready = ($report.ollama.Ok -and $report.qdrant.Ok)
    }
    if (-not $ready) {
        Write-Warn 'Stack not fully healthy - browser will run but Truth Engine may be limited until Ollama/Qdrant are up.'
        exit 2
    }
}
finally {
    Pop-Location
}
