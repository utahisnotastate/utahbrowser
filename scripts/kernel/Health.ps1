# Local service health probes for Ollama and Qdrant.

function Test-OllamaHealth {
    param(
        [string]$HostUrl,
        [int]$TimeoutSec = 8
    )

    $base = $HostUrl.TrimEnd('/')
    try {
        $resp = Invoke-WebRequest -Uri "$base/api/tags" -Method Get -TimeoutSec $TimeoutSec -UseBasicParsing
        return [PSCustomObject]@{
            Ok       = $resp.StatusCode -eq 200
            Message  = 'Ollama API reachable'
            Host     = $base
            TagsJson = $resp.Content
        }
    }
    catch {
        return [PSCustomObject]@{
            Ok      = $false
            Message = "Ollama unreachable at $base - $($_.Exception.Message)"
            Host    = $base
        }
    }
}

function Get-OllamaInstalledModels {
    param([string]$TagsJson)
    $models = @()
    try {
        $data = $TagsJson | ConvertFrom-Json
        foreach ($m in $data.models) {
            if ($m.name) { $models += $m.name }
        }
    }
    catch { }
    return $models
}

function Test-ModelPresent {
    param(
        [string[]]$Installed,
        [string]$Wanted
    )
    foreach ($name in $Installed) {
        if ($name -eq $Wanted -or $name.StartsWith("${Wanted}:")) {
            return $true
        }
    }
    return $false
}

function Test-QdrantHealth {
    param(
        [string]$BaseUrl,
        [int]$TimeoutSec = 8
    )

    $base = $BaseUrl.TrimEnd('/')
    $endpoints = @(
        "$base/readyz",
        "$base/healthz",
        "$base/collections"
    )

    foreach ($uri in $endpoints) {
        try {
            $resp = Invoke-WebRequest -Uri $uri -Method Get -TimeoutSec $TimeoutSec -UseBasicParsing
            if ($resp.StatusCode -eq 200) {
                return [PSCustomObject]@{
                    Ok      = $true
                    Message = "Qdrant reachable ($uri)"
                    Url     = $base
                }
            }
        }
        catch { continue }
    }

    return [PSCustomObject]@{
        Ok      = $false
        Message = "Qdrant unreachable at $base"
        Url     = $base
    }
}

function Invoke-Docker {
    param([string[]]$ArgumentList)

    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        $out = & docker @ArgumentList 2>&1 | ForEach-Object {
            if ($_ -is [System.Management.Automation.ErrorRecord]) { $_.ToString() } else { $_ }
        }
        return @{
            ExitCode = [int]$LASTEXITCODE
            Output   = ($out -join "`n")
        }
    }
    finally {
        $ErrorActionPreference = $prev
    }
}

function Start-QdrantDocker {
    param(
        [string]$ContainerName = 'utah-qdrant',
        [string]$BaseUrl = 'http://127.0.0.1:6333',
        [int]$WaitSeconds = 60
    )

    if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
        return [PSCustomObject]@{
            Ok      = $false
            Message = 'Docker not installed. Install Docker Desktop: https://www.docker.com/products/docker-desktop/'
        }
    }

    $info = Invoke-Docker -ArgumentList @('info')
    if ($info.ExitCode -ne 0) {
        return [PSCustomObject]@{
            Ok      = $false
            Message = 'Docker is installed but not running. Start Docker Desktop, wait until it is ready, then retry.'
        }
    }

    try {
        $existing = Invoke-Docker -ArgumentList @('ps', '-a', '--filter', "name=$ContainerName", '--format', '{{.Names}}')
        $hasContainer = $existing.Output -match [regex]::Escape($ContainerName)

        if ($hasContainer) {
            $inspect = Invoke-Docker -ArgumentList @('inspect', '-f', '{{.State.Running}}', $ContainerName)
            if ($inspect.Output -notmatch 'true') {
                Write-Host "  Starting existing container $ContainerName..." -ForegroundColor DarkGray
                $start = Invoke-Docker -ArgumentList @('start', $ContainerName)
                if ($start.ExitCode -ne 0) {
                    Write-Warn "docker start failed, recreating container..."
                    Invoke-Docker -ArgumentList @('rm', '-f', $ContainerName) | Out-Null
                    $hasContainer = $false
                }
            }
        }

        if (-not $hasContainer) {
            Write-Host '  Pulling qdrant/qdrant (first run may take a minute)...' -ForegroundColor DarkGray
            $pull = Invoke-Docker -ArgumentList @('pull', 'qdrant/qdrant')
            if ($pull.ExitCode -ne 0) {
                return [PSCustomObject]@{
                    Ok      = $false
                    Message = "docker pull qdrant/qdrant failed: $($pull.Output)"
                }
            }

            $run = Invoke-Docker -ArgumentList @(
                'run', '-d',
                '--name', $ContainerName,
                '-p', '6333:6333',
                '-p', '6334:6334',
                '--restart', 'unless-stopped',
                'qdrant/qdrant'
            )
            if ($run.ExitCode -ne 0) {
                if ($run.Output -match 'already in use|Conflict|port is already allocated') {
                    Write-Warn 'Port 6333 in use - attempting to start existing utah-qdrant...'
                    Invoke-Docker -ArgumentList @('start', $ContainerName) | Out-Null
                }
                else {
                    return [PSCustomObject]@{
                        Ok      = $false
                        Message = "docker run failed: $($run.Output)"
                    }
                }
            }
        }

        Write-Host '  Waiting for Qdrant API...' -ForegroundColor DarkGray
        $deadline = (Get-Date).AddSeconds($WaitSeconds)
        while ((Get-Date) -lt $deadline) {
            $health = Test-QdrantHealth -BaseUrl $BaseUrl -TimeoutSec 3
            if ($health.Ok) {
                return [PSCustomObject]@{
                    Ok      = $true
                    Message = "Qdrant ready ($ContainerName)"
                }
            }
            Start-Sleep -Seconds 2
        }

        return [PSCustomObject]@{
            Ok      = $false
            Message = "Container $ContainerName running but API not ready. Check: docker logs $ContainerName"
        }
    }
    catch {
        return [PSCustomObject]@{
            Ok      = $false
            Message = $_.Exception.Message
        }
    }
}

<#
.SYNOPSIS
    Ensures Qdrant is reachable; auto-starts Docker container if needed.
#>
function Ensure-QdrantReady {
    param(
        [string]$BaseUrl = 'http://127.0.0.1:6333',
        [switch]$NoAutoStart,
        [switch]$Quiet,
        [int]$WaitSeconds = 60
    )

    if (-not $Quiet) {
        Write-Host '  Checking Qdrant...' -ForegroundColor DarkGray
    }

    $health = Test-QdrantHealth -BaseUrl $BaseUrl -TimeoutSec 5
    if ($health.Ok) {
        if (-not $Quiet) {
            Write-Host "  [OK] $($health.Message)" -ForegroundColor DarkGreen
        }
        return $health
    }

    if ($NoAutoStart) {
        if (-not $Quiet) {
            Write-Host "  [OFFLINE] $($health.Message)" -ForegroundColor Red
        }
        return $health
    }

    if (-not $Quiet) {
        Write-Host '  Qdrant offline - auto-starting via Docker...' -ForegroundColor Yellow
    }

    $started = Start-QdrantDocker -BaseUrl $BaseUrl -WaitSeconds $WaitSeconds
    if (-not $started.Ok) {
        if (-not $Quiet) {
            Write-Host "  [FAILED] $($started.Message)" -ForegroundColor Red
        }
        return [PSCustomObject]@{
            Ok      = $false
            Message = $started.Message
            Url     = $BaseUrl.TrimEnd('/')
        }
    }

    $health = Test-QdrantHealth -BaseUrl $BaseUrl -TimeoutSec 8
    if ($health.Ok) {
        if (-not $Quiet) {
            Write-Host "  [OK] $($started.Message)" -ForegroundColor DarkGreen
        }
    }
    else {
        if (-not $Quiet) {
            Write-Host "  [FAILED] Qdrant still unreachable after Docker start" -ForegroundColor Red
        }
    }
    return $health
}

function Ensure-QdrantCollection {
    param(
        [string]$BaseUrl,
        [string]$Collection,
        [int]$VectorSize = 768
    )

    $base = $BaseUrl.TrimEnd('/')
    $checkUri = "$base/collections/$Collection"
    try {
        $resp = Invoke-WebRequest -Uri $checkUri -Method Get -TimeoutSec 10 -UseBasicParsing
        if ($resp.StatusCode -eq 200) {
            return [PSCustomObject]@{ Ok = $true; Message = "Collection '$Collection' exists" }
        }
    }
    catch {
        if ($_.Exception.Response.StatusCode.value__ -ne 404) {
            # continue to create attempt
        }
    }

    $body = @{
        vectors = @{
            size     = $VectorSize
            distance = 'Cosine'
        }
    } | ConvertTo-Json -Depth 4

    try {
        Invoke-WebRequest -Uri $checkUri -Method Put -Body $body -ContentType 'application/json' -TimeoutSec 30 -UseBasicParsing | Out-Null
        return [PSCustomObject]@{ Ok = $true; Message = "Created collection '$Collection'" }
    }
    catch {
        return [PSCustomObject]@{
            Ok      = $false
            Message = "Failed to create collection '$Collection': $($_.Exception.Message)"
        }
    }
}
