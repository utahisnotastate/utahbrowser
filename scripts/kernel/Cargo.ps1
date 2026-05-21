# Run cargo/rustc without PowerShell treating stderr progress as terminating errors.

function Convert-NativeOutput {
    param([object]$Line)
    if ($Line -is [System.Management.Automation.ErrorRecord]) {
        return $Line.ToString()
    }
    return [string]$Line
}

function Invoke-Cargo {
    param(
        [Parameter(Mandatory = $true, Position = 0)]
        [string[]]$ArgumentList,
        [switch]$ShowOutput,
        [string]$LogPath
    )

    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        $lines = & cargo @ArgumentList 2>&1
        $exit = $LASTEXITCODE

        $text = foreach ($line in $lines) { Convert-NativeOutput $line }

        if ($LogPath) {
            $text | Out-File -FilePath $LogPath -Encoding utf8
            if ($ShowOutput) {
                $text | ForEach-Object { Write-Host $_ }
            }
        }
        elseif ($ShowOutput) {
            $text | ForEach-Object { Write-Host $_ }
        }

        return [int]$exit
    }
    finally {
        $ErrorActionPreference = $prev
    }
}

function Get-ToolchainLine {
    param([string]$Tool)
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        $out = & $Tool --version 2>&1 | ForEach-Object { Convert-NativeOutput $_ }
        return ($out | Select-Object -First 1)
    }
    catch {
        return "$Tool (not found)"
    }
    finally {
        $ErrorActionPreference = $prev
    }
}
