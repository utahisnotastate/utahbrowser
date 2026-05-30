# Lazarus Daemon — developer helper: relaunch utah-browser.exe after exit

$BinPath = Join-Path $PSScriptRoot "..\dist\utah-browser.exe"
if (-not (Test-Path $BinPath)) {
    $BinPath = Join-Path $PSScriptRoot "..\target\release\utah-browser.exe"
}

Write-Host "[LAZARUS] Relaunch loop for Utah Browser (dev only)" -ForegroundColor Cyan

while ($true) {
    if (Test-Path $BinPath) {
        Write-Host "[LAZARUS] Starting: $BinPath" -ForegroundColor Green
        Start-Process -FilePath $BinPath -Wait
    } else {
        Write-Host "[LAZARUS] Binary not found. Waiting..." -ForegroundColor Yellow
    }
    
    Write-Host "[LAZARUS] Process exited. Restarting in 5 seconds..." -ForegroundColor Red
    Start-Sleep -Seconds 5
}
