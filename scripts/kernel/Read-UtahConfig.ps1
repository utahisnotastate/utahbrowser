# Parses config/default.toml for kernel bootstrap (no external TOML dependency).

function Read-UtahConfig {
    param([string]$ConfigPath)

    if (-not (Test-Path $ConfigPath)) {
        throw "Config not found: $ConfigPath"
    }

    $raw = Get-Content $ConfigPath -Raw

    function Get-TomlValue {
        param([string]$Section, [string]$Key)

        $sectionPattern = '(?ms)\[' + [regex]::Escape($Section) + '\]\s*\r?\n(.*?)(?=\r?\n\[|\z)'
        if ($raw -notmatch $sectionPattern) {
            return $null
        }
        $body = $Matches[1]

        $keyPattern = '(?m)^' + [regex]::Escape($Key) + '\s*=\s*"([^"]*)"'
        if ($body -match $keyPattern) {
            return $Matches[1]
        }

        $keyPatternBare = '(?m)^' + [regex]::Escape($Key) + '\s*=\s*([^\r\n#]+)'
        if ($body -match $keyPatternBare) {
            return $Matches[1].Trim()
        }

        return $null
    }

    $ollamaHost = Get-TomlValue 'ollama' 'host'
    if (-not $ollamaHost) { $ollamaHost = 'http://127.0.0.1:11434' }
    $embed = Get-TomlValue 'ollama' 'embed_model'
    if (-not $embed) { $embed = 'nomic-embed-text' }
    $chat = Get-TomlValue 'ollama' 'chat_model'
    if (-not $chat) { $chat = 'llama3.2' }
    $qdrant = Get-TomlValue 'qdrant' 'url'
    if (-not $qdrant) { $qdrant = 'http://127.0.0.1:6333' }
    $collection = Get-TomlValue 'qdrant' 'collection'
    if (-not $collection) { $collection = 'utah_notebooks' }
    $vectorSize = Get-TomlValue 'qdrant' 'vector_size'
    if (-not $vectorSize) { $vectorSize = '768' }

    [PSCustomObject]@{
        OllamaHost       = $ollamaHost
        EmbedModel       = $embed
        ChatModel        = $chat
        QdrantUrl        = $qdrant
        QdrantCollection = $collection
        QdrantVectorSize = [int]$vectorSize
        KnowledgePath    = (Get-TomlValue 'knowledge' 'path')
    }
}
