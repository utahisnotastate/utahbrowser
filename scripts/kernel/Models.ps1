# Ollama model auto-pull for Zero-Click Kernel.

function Invoke-OllamaPull {
    param(
        [string]$Model,
        [switch]$Quiet
    )

    if (-not (Get-Command ollama -ErrorAction SilentlyContinue)) {
        throw 'ollama CLI not found in PATH'
    }

    if (-not $Quiet) {
        Write-Host "  Pulling model: $Model" -ForegroundColor Cyan
    }

    & ollama pull $Model
    if ($LASTEXITCODE -ne 0) {
        throw "ollama pull failed for $Model (exit $LASTEXITCODE)"
    }
}

function Ensure-OllamaModels {
    param(
        [string]$HostUrl,
        [string[]]$Models,
        [switch]$ForcePull
    )

    $health = Test-OllamaHealth -HostUrl $HostUrl
    if (-not $health.Ok) {
        throw $health.Message
    }

    $installed = Get-OllamaInstalledModels -TagsJson $health.TagsJson
    $results = @()

    foreach ($model in $Models) {
        if ($ForcePull -or -not (Test-ModelPresent -Installed $installed -Wanted $model)) {
            try {
                Invoke-OllamaPull -Model $model
                $results += [PSCustomObject]@{
                    Model  = $model
                    Status = 'pulled'
                }
            }
            catch {
                $results += [PSCustomObject]@{
                    Model  = $model
                    Status = 'failed'
                    Error  = $_.Exception.Message
                }
            }
        }
        else {
            $results += [PSCustomObject]@{
                Model  = $model
                Status = 'present'
            }
        }
    }

    return $results
}
