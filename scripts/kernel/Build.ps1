# Robust release build with cache repair and Windows policy diagnostics.

function Get-UtahBuildLogPath {
    param([string]$Root)
    Join-Path $Root 'target\install-build.log'
}

function Repair-UtahCargoCache {
    param([string]$Root)

    Write-Host '  Repairing Cargo cache (cargo clean + refresh zerovec crates)...' -ForegroundColor Cyan
    Push-Location $Root
    try {
        cargo clean 2>&1 | Out-Null
        $registrySrc = Join-Path $env:USERPROFILE '.cargo\registry\src'
        if (Test-Path $registrySrc) {
            $patterns = @('zerovec-*', 'zerovec_derive-*', 'icu_*', 'yoke-*')
            Get-ChildItem $registrySrc -Directory -ErrorAction SilentlyContinue | ForEach-Object {
                $indexDir = $_.FullName
                foreach ($pattern in $patterns) {
                    Get-ChildItem $indexDir -Filter $pattern -Directory -ErrorAction SilentlyContinue |
                        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
                }
            }
        }
        cargo fetch 2>&1 | Out-Null
    }
    finally {
        Pop-Location
    }
}

function Show-UtahBuildFailureHelp {
    param(
        [string]$Log,
        [string]$Root
    )
    $script:UtahBuildLogRoot = $Root

    if ($Log -match '4551|Application Control policy') {
        Write-Host ''
        Write-Host '=== Windows blocked Cargo build scripts (error 4551) ===' -ForegroundColor Yellow
        Write-Host @'
Application Control (WDAC / Smart App Control / enterprise policy) blocked
target\release\build\*\build-script-build.exe

Try these fixes (in order):
  1. Run the repair helper as Administrator:
       .\scripts\Repair-BuildEnvironment.ps1
  2. Windows Security > Virus & threat protection > Manage settings > Exclusions
       Add folders:
         - Your utahbrowser folder (e.g. C:\code\utahbrowser)
         - %USERPROFILE%\.cargo
  3. If this is a school/work PC, ask IT to allow Cargo build scripts under:
         <project>\target\release\build\
  4. Temporarily disable Smart App Control (Settings > Privacy & Security
       > Windows Security > App & browser control) only if your policy allows.

Then re-run:
  .\scripts\install.ps1
'@ -ForegroundColor Gray
    }

    if ($Log -match 'zerovec_derive|can''t find crate') {
        Write-Host ''
        Write-Host '=== Corrupted Rust dependency cache (zerovec) ===' -ForegroundColor Yellow
        Write-Host @'
The installer already attempted a cache repair. If build still fails, run:

  .\scripts\Repair-BuildEnvironment.ps1
  cd C:\code\utahbrowser
  cargo build --release

Or reinstall Rust: https://rustup.rs/
'@ -ForegroundColor Gray
    }

    Write-Host ''
    if ($script:UtahBuildLogRoot) {
        Write-Host "Full log: $(Get-UtahBuildLogPath -Root $script:UtahBuildLogRoot)" -ForegroundColor DarkGray
    }
}

function Invoke-UtahCargoBuild {
    param(
        [string]$Root,
        [switch]$SkipRepair
    )

    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw 'cargo not found in PATH. Install Rust from https://rustup.rs/ then reopen PowerShell.'
    }

    $logPath = Get-UtahBuildLogPath -Root $Root
    $logDir = Split-Path $logPath -Parent
    if (-not (Test-Path $logDir)) {
        New-Item -ItemType Directory -Force -Path $logDir | Out-Null
    }

    Write-Host "  $(rustc --version 2>&1)" -ForegroundColor DarkGray
    Write-Host "  $(cargo --version 2>&1)" -ForegroundColor DarkGray

    Push-Location $Root
    try {
        cargo fetch 2>&1 | Out-Null

        function Invoke-ReleaseBuild {
            $prev = $ErrorActionPreference
            $ErrorActionPreference = 'Continue'
            try {
                & cargo build --release 2>&1 | Tee-Object -FilePath $logPath
                return $LASTEXITCODE
            }
            finally {
                $ErrorActionPreference = $prev
            }
        }

        $code = Invoke-ReleaseBuild
        if ($code -eq 0) {
            return $true
        }

        $log = ''
        if (Test-Path $logPath) {
            $log = Get-Content $logPath -Raw -ErrorAction SilentlyContinue
        }

        $needsRepair = ($log -match 'zerovec_derive|can''t find crate|E0463|E0432') -and -not $SkipRepair
        $needsClean = $log -match 'zerovec|num-traits|build-script|4551'

        if ($needsRepair -or $needsClean) {
            Write-Host '  First build failed; running automatic repair...' -ForegroundColor Yellow
            Repair-UtahCargoCache -Root $Root
            $code = Invoke-ReleaseBuild
            if ($code -eq 0) {
                Write-Host '  Build succeeded after repair.' -ForegroundColor DarkGreen
                return $true
            }
            if (Test-Path $logPath) {
                $log = Get-Content $logPath -Raw -ErrorAction SilentlyContinue
            }
        }

        Show-UtahBuildFailureHelp -Log $log -Root $Root
        return $false
    }
    finally {
        Pop-Location
    }
}
