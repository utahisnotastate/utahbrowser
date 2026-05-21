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
        Message = "Qdrant unreachable at $base - start Docker Desktop, then: docker run -d -p 6333:6333 --name utah-qdrant qdrant/qdrant"
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
        [int]$WaitSeconds = 45
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
            Message = 'Docker is installed but not running. Start Docker Desktop and retry.'
        }
    }

    try {
        $existing = Invoke-Docker -ArgumentList @('ps', '-a', '--filter', "name=$ContainerName", '--format', '{{.Names}}')
        if ($existing.Output -match $ContainerName) {
            $start = Invoke-Docker -ArgumentList @('start', $ContainerName)
            if ($start.ExitCode -ne 0) {
                return [PSCustomObject]@{
                    Ok      = $false
                    Message = "docker start $ContainerName failed: $($start.Output)"
                }
            }
        }
        else {
            Write-Host '  Pulling qdrant/qdrant image (first time may take a minute)...' -ForegroundColor DarkGray
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
                'qdrant/qdrant'
            )
            if ($run.ExitCode -ne 0) {
                if ($run.Output -match 'already in use|Conflict') {
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
            Message = "Container $ContainerName started but API not ready. Try: docker logs $ContainerName"
        }
    }
    catch {
        return [PSCustomObject]@{
            Ok      = $false
            Message = $_.Exception.Message
        }
    }
}
