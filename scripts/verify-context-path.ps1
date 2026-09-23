param(
    [string]$JavaHome = "",
    [string]$JarPath = ""
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
if (-not $JavaHome) { $JavaHome = Join-Path $projectRoot ".tools\jdk-11.0.8" }
if (-not $JarPath) { $JarPath = Join-Path $projectRoot "build\libs\reader-4.0.7.jar" }
$java = Join-Path $JavaHome "bin\java.exe"
if (-not (Test-Path -LiteralPath $java) -or -not (Test-Path -LiteralPath $JarPath)) {
    throw "JDK or built JAR is missing. Run scripts/build.ps1 first."
}

$tempRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
$workDir = Join-Path $tempRoot ("reader-context-smoke-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $workDir | Out-Null
$processes = @()
$passed = $false
$handler = [Net.Http.HttpClientHandler]::new()
$handler.AllowAutoRedirect = $false
$client = [Net.Http.HttpClient]::new($handler)
$client.Timeout = [TimeSpan]::FromSeconds(10)

function Get-FreePort {
    $listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
    $listener.Start()
    try { return ([Net.IPEndPoint]$listener.LocalEndpoint).Port }
    finally { $listener.Stop() }
}

function Start-Reader([string]$Name, [string]$ContextPath) {
    $instanceDir = Join-Path $workDir $Name
    New-Item -ItemType Directory -Path $instanceDir | Out-Null
    $port = Get-FreePort
    $arguments = @("-jar", $JarPath, "--reader.server.port=$port")
    if ($ContextPath) { $arguments += "--reader.server.contextPath=$ContextPath" }
    $process = Start-Process -FilePath $java -ArgumentList $arguments `
        -WorkingDirectory $instanceDir `
        -RedirectStandardOutput (Join-Path $instanceDir "out.log") `
        -RedirectStandardError (Join-Path $instanceDir "err.log") `
        -WindowStyle Hidden -PassThru
    $script:processes += $process
    return "http://127.0.0.1:$port"
}

function Wait-Ready([string]$Url) {
    $deadline = [DateTime]::UtcNow.AddSeconds(45)
    while ([DateTime]::UtcNow -lt $deadline) {
        try {
            $response = $client.GetAsync($Url).GetAwaiter().GetResult()
            try { if ([int]$response.StatusCode -eq 200) { return } }
            finally { $response.Dispose() }
        } catch { }
        Start-Sleep -Milliseconds 500
    }
    throw "Reader did not become ready at $Url; logs: $workDir"
}

function Assert-Http([string]$Base, [string]$Path, [int]$Status, [string]$Location = "") {
    $response = $client.GetAsync($Base + $Path).GetAwaiter().GetResult()
    try {
        $actualStatus = [int]$response.StatusCode
        $actualLocation = if ($response.Headers.Location) { $response.Headers.Location.ToString() } else { "" }
        if ($actualStatus -ne $Status -or $actualLocation -cne $Location) {
            throw "$Path returned $actualStatus Location=$actualLocation; expected $Status Location=$Location"
        }
        Write-Host "$actualStatus $Path"
    } finally { $response.Dispose() }
}

function Get-HomeAssetPaths([string]$Base) {
    $response = $client.GetAsync($Base + "/").GetAwaiter().GetResult()
    try {
        $html = $response.Content.ReadAsStringAsync().GetAwaiter().GetResult()
        $css = [regex]::Match($html, 'href="(css/[^"]+\.css)"').Groups[1].Value
        $js = [regex]::Match($html, 'src="(js/[^"]+\.js)"').Groups[1].Value
        if (-not $css -or -not $js) { throw "Home page has no relative CSS/JS references" }
        return [pscustomobject]@{ Css = $css; Js = $js }
    } finally { $response.Dispose() }
}

try {
    $root = Start-Reader "root" ""
    Wait-Ready "$root/reader3/getSystemInfo"
    $assets = Get-HomeAssetPaths $root
    Assert-Http $root "/" 200
    Assert-Http $root ("/" + $assets.Css) 200
    Assert-Http $root ("/" + $assets.Js) 200
    Assert-Http $root "/reader3/getSystemInfo" 200
    Assert-Http $root "/simple-web/" 200

    $sub = Start-Reader "subpath" "/reader"
    Wait-Ready "$sub/reader/reader3/getSystemInfo"
    Assert-Http $sub "/reader" 308 "/reader/"
    Assert-Http $sub "/reader?x=1" 308 "/reader/?x=1"
    Assert-Http $sub "/reader/" 200
    Assert-Http $sub ("/reader/" + $assets.Css) 200
    Assert-Http $sub ("/reader/" + $assets.Js) 200
    Assert-Http $sub "/reader/service-worker.js" 200
    Assert-Http $sub "/reader/manifest.json" 200
    Assert-Http $sub "/reader/assets/reader.css" 200
    Assert-Http $sub "/reader/reader3/getSystemInfo" 200
    foreach ($page in @("", "reader.html", "search.html", "rss.html")) {
        Assert-Http $sub ("/reader/simple-web/" + $page) 200
    }
    Assert-Http $sub ("/" + $assets.Css) 404
    $passed = $true
} finally {
    foreach ($process in $processes) {
        if (-not $process.HasExited) { Stop-Process -Id $process.Id -ErrorAction Stop }
    }
    $client.Dispose()
    $handler.Dispose()
    if ($passed) {
        $resolved = (Resolve-Path -LiteralPath $workDir).Path
        $prefix = $tempRoot.TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
        if ($resolved.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase) -and
            (Split-Path -Leaf $resolved).StartsWith("reader-context-smoke-")) {
            Remove-Item -LiteralPath $resolved -Recurse -Force
        }
    } else {
        Write-Warning "Retained isolated test logs: $workDir"
    }
}
