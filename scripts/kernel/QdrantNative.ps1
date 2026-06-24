# Native Qdrant for Utah Browser — no Docker required.
# Downloads the official Windows binary on first use and runs it locally.

$script:UtahQdrantVersion = 'v1.13.2'

function Get-UtahQdrantStateRoot {
    param([string]$ProjectRoot = $null)
    if ($ProjectRoot) {
        return Join-Path $ProjectRoot '.utah-browser\qdrant'
    }
    return Join-Path $env:LOCALAPPDATA 'UtahBrowser\qdrant'
}

function Get-QdrantNativePaths {
    param([string]$ProjectRoot = $null)

    $root = Get-UtahQdrantStateRoot -ProjectRoot $ProjectRoot
    [PSCustomObject]@{
        Root       = $root
        BinDir     = Join-Path $root 'bin'
        StorageDir = Join-Path $root 'storage'
        ConfigPath = Join-Path $root 'config.yaml'
        PidFile    = Join-Path $root 'qdrant.pid'
        LogOut     = Join-Path $root 'qdrant.out.log'
        LogErr     = Join-Path $root 'qdrant.err.log'
        ExePath    = Join-Path $root 'bin\qdrant.exe'
    }
}

function Get-QdrantNativeDownloadAsset {
    if ($env:PROCESSOR_ARCHITECTURE -eq 'ARM64') {
        return "qdrant-aarch64-pc-windows-msvc.zip"
    }
    return 'qdrant-x86_64-pc-windows-msvc.zip'
}

function Install-QdrantNativeBinary {
    param([string]$ProjectRoot = $null)

    $paths = Get-QdrantNativePaths -ProjectRoot $ProjectRoot
    New-Item -ItemType Directory -Force -Path $paths.BinDir, $paths.StorageDir | Out-Null

    if (Test-Path $paths.ExePath) {
        return [PSCustomObject]@{ Ok = $true; Message = 'Native Qdrant binary already installed' }
    }

    $asset = Get-QdrantNativeDownloadAsset
    $url = "https://github.com/qdrant/qdrant/releases/download/$($script:UtahQdrantVersion)/$asset"
    $zipPath = Join-Path $paths.Root "qdrant-download.zip"

    Write-Host "  Downloading Qdrant $asset (one-time, no Docker)..." -ForegroundColor Cyan
    Write-Host "  $url" -ForegroundColor DarkGray

    try {
        Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing
        Expand-Archive -Path $zipPath -DestinationPath $paths.BinDir -Force
        Remove-Item $zipPath -Force -ErrorAction SilentlyContinue

        $found = Get-ChildItem -Path $paths.BinDir -Filter 'qdrant.exe' -Recurse -ErrorAction SilentlyContinue |
            Select-Object -First 1
        if (-not $found) {
            return [PSCustomObject]@{
                Ok      = $false
                Message = 'Download succeeded but qdrant.exe was not found in the archive'
            }
        }
        if ($found.FullName -ne $paths.ExePath) {
            Copy-Item $found.FullName $paths.ExePath -Force
        }

        return [PSCustomObject]@{
            Ok      = $true
            Message = "Installed native Qdrant at $($paths.ExePath)"
        }
    }
    catch {
        return [PSCustomObject]@{
            Ok      = $false
            Message = "Failed to download native Qdrant: $($_.Exception.Message)"
        }
    }
}

function Write-QdrantNativeConfig {
    param($Paths)

    $storage = $Paths.StorageDir -replace '\\', '/'
    @"
storage:
  storage_path: $storage
service:
  host: 127.0.0.1
  http_port: 6333
  grpc_port: 6334
"@ | Set-Content -Path $Paths.ConfigPath -Encoding UTF8
}

function Get-QdrantNativeRunningPid {
    param($Paths)

    if (-not (Test-Path $Paths.PidFile)) {
        return $null
    }
    $pidRaw = Get-Content $Paths.PidFile -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($pidRaw -match '^\d+$') {
        $proc = Get-Process -Id ([int]$pidRaw) -ErrorAction SilentlyContinue
        if ($proc -and $proc.Path -like '*qdrant*') {
            return [int]$pidRaw
        }
    }
    return $null
}

function Start-QdrantNative {
    param(
        [string]$BaseUrl = 'http://127.0.0.1:6333',
        [string]$ProjectRoot = $null,
        [int]$WaitSeconds = 60
    )

    $paths = Get-QdrantNativePaths -ProjectRoot $ProjectRoot

    $existing = Test-QdrantHealth -BaseUrl $BaseUrl -TimeoutSec 2
    if ($existing.Ok) {
        return [PSCustomObject]@{
            Ok      = $true
            Message = 'Qdrant already running (native or other)'
        }
    }

    $installed = Install-QdrantNativeBinary -ProjectRoot $ProjectRoot
    if (-not $installed.Ok) {
        return $installed
    }

    $runningPid = Get-QdrantNativeRunningPid -Paths $paths
    if (-not $runningPid) {
        Write-QdrantNativeConfig -Paths $paths
        Write-Host '  Starting native Qdrant (no Docker)...' -ForegroundColor Cyan

        try {
            $proc = Start-Process `
                -FilePath $paths.ExePath `
                -ArgumentList @('--config-path', $paths.ConfigPath) `
                -WorkingDirectory $paths.Root `
                -WindowStyle Hidden `
                -PassThru `
                -RedirectStandardOutput $paths.LogOut `
                -RedirectStandardError $paths.LogErr

            $proc.Id | Set-Content $paths.PidFile -Encoding ASCII
        }
        catch {
            $err = $_.Exception.Message
            if ($err -match "Application Control policy" -or $err -match "blocked by") {
                return [PSCustomObject]@{
                    Ok      = $false
                    Message = "Native Qdrant BLOCKED by Windows Smart App Control or Policy. You may need to right-click qdrant.exe and select 'Unblock' or add an exclusion. Path: $($paths.ExePath)"
                }
            }
            return [PSCustomObject]@{
                Ok      = $false
                Message = "Failed to start native Qdrant: $err"
            }
        }
    }

    Write-Host '  Waiting for native Qdrant API...' -ForegroundColor DarkGray
    $deadline = (Get-Date).AddSeconds($WaitSeconds)
    while ((Get-Date) -lt $deadline) {
        $health = Test-QdrantHealth -BaseUrl $BaseUrl -TimeoutSec 3
        if ($health.Ok) {
            return [PSCustomObject]@{
                Ok      = $true
                Message = "Native Qdrant ready ($($paths.ExePath))"
            }
        }
        Start-Sleep -Seconds 2
    }

    return [PSCustomObject]@{
        Ok      = $false
        Message = "Native Qdrant started but API not ready. Logs: $($paths.LogOut) / $($paths.LogErr)"
    }
}
