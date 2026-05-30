# Millennium Pipeline — periodic dependency refresh and release build (dev helper)

$RepoRoot = Resolve-Path "$PSScriptRoot\.."

Write-Host "[MILLENNIUM] Periodic cargo update + release build" -ForegroundColor Cyan

function Run-Optimization {
    Write-Host "[MILLENNIUM] Running cargo update..." -ForegroundColor Yellow
    Set-Location $RepoRoot
    cargo update
}

function Run-SilentBuild {
    Write-Host "[MILLENNIUM] Release build..." -ForegroundColor Green
    cargo build --release --quiet
}

while ($true) {
    try {
        Run-Optimization
        Run-SilentBuild
        Write-Host "[MILLENNIUM] Cycle complete." -ForegroundColor Green
    } catch {
        Write-Host "[!] Pipeline error: $_" -ForegroundColor Red
    }

    Start-Sleep -Seconds 3600
}
