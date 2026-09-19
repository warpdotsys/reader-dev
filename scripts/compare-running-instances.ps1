param(
    [string]$OriginalBaseUri = "http://127.0.0.1:18080",
    [string]$RestoredBaseUri = "http://127.0.0.1:18081",
    [string]$OutputPath = ""
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($OutputPath)) {
    $OutputPath = Join-Path $projectRoot "reports\blackbox-diff-latest.json"
}

$probes = @(
    @{ name = "home"; method = "GET"; path = "/" },
    @{ name = "system"; method = "GET"; path = "/reader3/getSystemInfo" },
    @{ name = "user-info"; method = "GET"; path = "/reader3/getUserInfo" },
    @{ name = "user-config"; method = "GET"; path = "/reader3/getUserConfig" },
    @{ name = "bookshelf"; method = "GET"; path = "/reader3/getBookshelf" },
    @{ name = "groups"; method = "GET"; path = "/reader3/getBookGroups" },
    @{ name = "toc-rules"; method = "GET"; path = "/reader3/getTxtTocRules" },
    @{ name = "rss"; method = "GET"; path = "/reader3/getRssSources" },
    @{ name = "tts"; method = "GET"; path = "/reader3/httpTTS/list" },
    @{
        name = "login-invalid"
        method = "POST"
        path = "/reader3/login"
        body = '{"username":"__recovery_probe__","password":"__invalid__"}'
    }
)

function Get-Sha256Hex([byte[]]$Bytes) {
    $sha = [Security.Cryptography.SHA256]::Create()
    try {
        return ([BitConverter]::ToString($sha.ComputeHash($Bytes))).Replace("-", "")
    }
    finally {
        $sha.Dispose()
    }
}

function Invoke-Probe([string]$BaseUri, [hashtable]$Probe) {
    $request = [Net.Http.HttpRequestMessage]::new(
        [Net.Http.HttpMethod]::new($Probe.method),
        $BaseUri.TrimEnd("/") + $Probe.path
    )
    if ($Probe.body) {
        $request.Content = [Net.Http.StringContent]::new(
            $Probe.body,
            [Text.Encoding]::UTF8,
            "application/json"
        )
    }

    try {
        $response = $script:httpClient.SendAsync($request).GetAwaiter().GetResult()
        $bytes = $response.Content.ReadAsByteArrayAsync().GetAwaiter().GetResult()
        $text = [Text.Encoding]::UTF8.GetString($bytes)
        $summary = [ordered]@{
            status = [int]$response.StatusCode
            contentType = [string]$response.Content.Headers.ContentType
            bytes = $bytes.Length
            sha256 = Get-Sha256Hex $bytes
            isSuccess = $null
            errorMsg = $null
            dataType = "non-json"
            dataCount = $null
        }

        try {
            $json = $text | ConvertFrom-Json -Depth 100
            if ($null -ne $json.PSObject.Properties["isSuccess"]) {
                $summary.isSuccess = $json.isSuccess
            }
            if ($null -ne $json.PSObject.Properties["errorMsg"]) {
                $summary.errorMsg = $json.errorMsg
            }
            if ($null -ne $json.PSObject.Properties["data"]) {
                if ($null -eq $json.data) {
                    $summary.dataType = "null"
                }
                elseif ($json.data -is [Array]) {
                    $summary.dataType = "array"
                    $summary.dataCount = $json.data.Count
                }
                else {
                    $summary.dataType = $json.data.GetType().Name
                }
            }
        }
        catch {
            # Non-JSON responses are intentionally retained as hash/size evidence.
        }

        return [pscustomobject]@{ summary = $summary; body = $text }
    }
    finally {
        $request.Dispose()
    }
}

Add-Type -AssemblyName System.Net.Http
$script:httpClient = [Net.Http.HttpClient]::new()
$script:httpClient.Timeout = [TimeSpan]::FromSeconds(90)
$results = @()

try {
    foreach ($probe in $probes) {
        $original = Invoke-Probe $OriginalBaseUri $probe
        $restored = Invoke-Probe $RestoredBaseUri $probe
        $results += [pscustomobject]@{
            name = $probe.name
            rawEqual = $original.body -ceq $restored.body
            original = $original.summary
            restored = $restored.summary
        }
    }
}
finally {
    $script:httpClient.Dispose()
}

$parent = Split-Path -Parent $OutputPath
if (-not (Test-Path -LiteralPath $parent)) {
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
}
$results | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $OutputPath -Encoding utf8
$results | Select-Object name, rawEqual, @{n="originalStatus";e={$_.original.status}}, @{n="restoredStatus";e={$_.restored.status}}, @{n="originalBytes";e={$_.original.bytes}}, @{n="restoredBytes";e={$_.restored.bytes}} | Format-Table -AutoSize
Write-Host "Report: $OutputPath"
