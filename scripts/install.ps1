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
    [switch]$InstallGhostLink,
    [switch]$GhostLinkStartup,
    [switch]$InstallURM,
    [switch]$UrmStartup,
    [switch]$UrmProgramData,
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

function Initialize-UtahVault {
    $vault = Join-Path $env:USERPROFILE '.utah_browser'
    foreach ($sub in @('vault', 'cache\tabs', 'extensions', 'logs')) {
        $p = Join-Path $vault $sub
        New-Item -ItemType Directory -Force -Path $p | Out-Null
    }
    Write-Host "  [OK] Vault seeded at $vault" -ForegroundColor DarkGreen
    return $vault
}

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

    Write-Step 'Zero-Click Kernel - vault manifestation'
    $null = Initialize-UtahVault

    if ($InstallGhostLink) {
        Write-Step 'Ghost-Link sensory daemon'
        $ghostInstaller = Join-Path $Root 'scripts\install_ghost_link.ps1'
        $ghostArgs = @{ StartNow = $true }
        if ($GhostLinkStartup) { $ghostArgs['RegisterStartup'] = $true }
        & $ghostInstaller @ghostArgs
    }

    if ($InstallURM) {
        Write-Step 'Utah Unified Reality Manifold (Nexus)'
        $urmInstaller = Join-Path $Root 'scripts\install_urm.ps1'
        $urmArgs = @{ StartNow = $true; StartGhostLink = $true }
        if ($UrmStartup) { $urmArgs['RegisterStartup'] = $true }
        if ($UrmProgramData) { $urmArgs['UseProgramData'] = $true }
        & $urmInstaller @urmArgs
    }

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

        # Qdrant must be up before Truth Engine steps (model pull does not need it;
        # collection prep and ingest do — ensure now so later steps never hit a dead API).
        Write-Host '  Ensuring Qdrant (required for Truth Engine)...' -ForegroundColor DarkGray
        $qdrant = Ensure-QdrantReady -BaseUrl $cfg.QdrantUrl -ProjectRoot $Root -NoAutoStart:$SkipQdrantStart
        $report.qdrant = $qdrant

        if ($qdrant.Ok) {
            $col = Ensure-QdrantCollection -BaseUrl $cfg.QdrantUrl -Collection $cfg.QdrantCollection -VectorSize $cfg.QdrantVectorSize
            if ($col.Ok) {
                Write-Host "  [OK] $($col.Message)" -ForegroundColor DarkGreen
            }
            else {
                Write-Warn $col.Message
            }
            $bmCol = 'utah_bookmarks'
            $bm = Ensure-QdrantCollection -BaseUrl $cfg.QdrantUrl -Collection $bmCol -VectorSize $cfg.QdrantVectorSize
            if ($bm.Ok) {
                Write-Host "  [OK] Semantic bookmarks: $bmCol" -ForegroundColor DarkGreen
            }
        }
        else {
            Write-Err $qdrant.Message
            if ($KnowledgePath) {
                throw @'
Qdrant could not be started (native download and Docker fallback both failed).
- Check internet for first-time Qdrant binary download
- Or install Docker Desktop and re-run: .\scripts\install.ps1 -KnowledgePath "<path>"
See docs/guides/BUILD_TROUBLESHOOTING.md
'@
            }
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

    # --- Phase 1b: Qdrant must be up before dist (launcher + Truth Engine depend on it) ---
    if (-not $SkipHealth) {
        Write-Step 'Ensuring Qdrant before packaging Truth Engine'
        $qdrant = Ensure-QdrantReady -BaseUrl $cfg.QdrantUrl -ProjectRoot $Root -NoAutoStart:$SkipQdrantStart
        $report.qdrant = $qdrant
        if ($qdrant.Ok) {
            $col = Ensure-QdrantCollection -BaseUrl $cfg.QdrantUrl -Collection $cfg.QdrantCollection -VectorSize $cfg.QdrantVectorSize
            if ($col.Ok) {
                Write-Host "  [OK] $($col.Message)" -ForegroundColor DarkGreen
            }
        }
        elseif ($KnowledgePath) {
            throw 'Qdrant is required but could not be started. Re-run install.ps1 (native Qdrant is downloaded automatically).'
        }
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
    $LaunchExe = Join-Path $Root 'target\release\utah-launch.exe'
    if (Test-Path $LaunchExe) {
        Copy-Item $LaunchExe (Join-Path $OutDir 'Utah Browser.exe') -Force
        Write-Host "  Launcher:    $OutDir\Utah Browser.exe  (double-click, no CLI)" -ForegroundColor Cyan
    }
    else {
        Write-Warn 'utah-launch.exe not built — run: cargo build --release --bin utah-launch'
    }
    Copy-Item (Join-Path $Root 'config') (Join-Path $OutDir 'config') -Recurse -Force
    Copy-Item (Join-Path $Root 'assets') (Join-Path $OutDir 'assets') -Recurse -Force
    Copy-Item (Join-Path $Root 'scripts\kernel') (Join-Path $OutDir 'scripts\kernel') -Recurse -Force
    Copy-Item (Join-Path $Root 'scripts\Ensure-Qdrant.ps1') (Join-Path $OutDir 'scripts\Ensure-Qdrant.ps1') -Force
    Copy-Item (Join-Path $Root 'scripts\Ensure-Services.ps1') (Join-Path $OutDir 'scripts\Ensure-Services.ps1') -Force
    Copy-Item $MyInvocation.MyCommand.Path (Join-Path $OutDir 'install.ps1') -Force

    $healthPath = Join-Path $OutDir 'health-report.json'
    $report | ConvertTo-Json -Depth 6 | Set-Content $healthPath -Encoding UTF8

    $launcher = Join-Path $OutDir 'Launch-UtahBrowser.ps1'
    @"
# Utah Browser launcher - ensures Qdrant, loads env, opens the app
`$ErrorActionPreference = 'Continue'
`$here = Split-Path -Parent `$MyInvocation.MyCommand.Path
if (Test-Path "`$here\utah.env.ps1") { . "`$here\utah.env.ps1" }

`$ensure = Join-Path `$here 'scripts\Ensure-Services.ps1'
if (Test-Path `$ensure) {
    & powershell -NoProfile -ExecutionPolicy Bypass -File `$ensure -ProjectRoot `$here
    if (`$LASTEXITCODE -ne 0) {
        Write-Host 'Warning: Qdrant/Ollama not fully ready. Truth Guard may be limited.' -ForegroundColor Yellow
    }
}

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
    Write-Host "  Launch:      .\dist\Utah Browser.exe"
    Write-Host "  (or)         .\dist\Launch-UtahBrowser.ps1"
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
