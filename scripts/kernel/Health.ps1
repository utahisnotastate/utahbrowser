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
        Message = "Qdrant unreachable at $base - start with: docker run -d -p 6333:6333 --name utah-qdrant qdrant/qdrant"
        Url     = $base
    }
}

function Start-QdrantDocker {
    param([string]$ContainerName = 'utah-qdrant')

    if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
        return [PSCustomObject]@{ Ok = $false; Message = 'Docker not installed' }
    }

    try {
        $existing = docker ps -a --filter "name=$ContainerName" --format '{{.Names}}' 2>$null
        if ($existing -eq $ContainerName) {
            docker start $ContainerName 2>$null | Out-Null
        }
        else {
            docker run -d --name $ContainerName -p 6333:6333 -p 6334:6334 qdrant/qdrant 2>$null | Out-Null
        }
        Start-Sleep -Seconds 3
        return [PSCustomObject]@{ Ok = $true; Message = "Started container $ContainerName" }
    }
    catch {
        return [PSCustomObject]@{ Ok = $false; Message = $_.Exception.Message }
    }
}
