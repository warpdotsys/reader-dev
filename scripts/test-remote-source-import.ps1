param(
    [string]$JavaHome = "",
    [string]$Jar = "",
    [switch]$SkipInvalidProbe
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
if (-not $JavaHome) { $JavaHome = Join-Path $root ".tools\jdk-11.0.8" }
if (-not $Jar) { $Jar = Join-Path $root "build\libs\reader-4.0.7.jar" }
$java = Join-Path $JavaHome "bin\java.exe"
$python = (Get-Command python -ErrorAction Stop).Source
foreach ($path in @($java, $python, $Jar)) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Required file not found: $path" }
}

function Get-FreePort {
    $listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
    $listener.Start()
    try { return ([Net.IPEndPoint]$listener.LocalEndpoint).Port }
    finally { $listener.Stop() }
}

function Invoke-Json([Net.Http.HttpClient]$Client, [string]$Url, [object]$Body = $null) {
    $request = [Net.Http.HttpRequestMessage]::new(
        $(if ($null -eq $Body) { [Net.Http.HttpMethod]::Get } else { [Net.Http.HttpMethod]::Post }), $Url)
    try {
        if ($null -ne $Body) {
            $json = ConvertTo-Json -InputObject $Body -Compress -Depth 20
            $request.Content = [Net.Http.StringContent]::new($json, [Text.Encoding]::UTF8, "application/json")
        }
        $response = $Client.SendAsync($request).GetAwaiter().GetResult()
        try {
            return [pscustomobject]@{
                status = [int]$response.StatusCode
                value = $response.Content.ReadAsStringAsync().GetAwaiter().GetResult() | ConvertFrom-Json -Depth 50
            }
        }
        finally { $response.Dispose() }
    }
    finally { $request.Dispose() }
}

function Assert-Success([object]$Response, [string]$Name) {
    if ($Response.status -ne 200 -or -not $Response.value.isSuccess) {
        throw "$Name failed: HTTP $($Response.status), $($Response.value.errorMsg)"
    }
}

$runDir = Join-Path ([IO.Path]::GetTempPath()) ("reader-remote-import-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $runDir | Out-Null
$fixturePort = Get-FreePort
$readerPort = Get-FreePort
$fixtureBase = "http://127.0.0.1:$fixturePort"
$readerBase = "http://127.0.0.1:$readerPort"
$fixture = $null
$reader = $null
$handler = [Net.Http.HttpClientHandler]::new()
$handler.UseCookies = $true
$client = [Net.Http.HttpClient]::new($handler)
$client.Timeout = [TimeSpan]::FromSeconds(20)
try {
    $fixture = Start-Process -FilePath $python -ArgumentList @(
        '-B', ('"' + (Join-Path $PSScriptRoot 'mock-book-source.py') + '"'), '--port', "$fixturePort"
    ) -PassThru -WindowStyle Hidden -RedirectStandardOutput (Join-Path $runDir 'fixture.out.log') `
        -RedirectStandardError (Join-Path $runDir 'fixture.err.log')
    $reader = Start-Process -FilePath $java -ArgumentList @(
        '-jar', ('"' + (Resolve-Path -LiteralPath $Jar).Path + '"'),
        "--reader.app.workDir=$runDir", "--reader.server.port=$readerPort",
        '--reader.app.secure=true', '--reader.app.licenseCheckEnabled=false',
        '--spring.profiles.active=prod'
    ) -PassThru -WindowStyle Hidden -RedirectStandardOutput (Join-Path $runDir 'reader.out.log') `
        -RedirectStandardError (Join-Path $runDir 'reader.err.log')

    $ready = $false
    for ($i = 0; $i -lt 120; $i++) {
        if ($fixture.HasExited -or $reader.HasExited) { throw "Fixture or Reader exited before readiness" }
        try {
            $fixtureReady = (Invoke-WebRequest -Uri "$fixtureBase/health" -TimeoutSec 2).StatusCode -eq 200
            $readerReady = (Invoke-Json $client "$readerBase/reader3/getSystemInfo").value.isSuccess
            if ($fixtureReady -and $readerReady) { $ready = $true; break }
        } catch { }
        Start-Sleep -Milliseconds 500
    }
    if (-not $ready) { throw "Reader or fixture did not become ready; logs: $runDir" }

    $username = "remoteprobe" + [guid]::NewGuid().ToString("N").Substring(0, 8)
    $password = "Probe-" + [guid]::NewGuid().ToString("N")
    Assert-Success (Invoke-Json $client "$readerBase/reader3/login" `
        @{ username = $username; password = $password; isLogin = $false }) 'register'
    Assert-Success (Invoke-Json $client "$readerBase/reader3/login" `
        @{ username = $username; password = $password; isLogin = $true }) 'login'

    $before = Invoke-Json $client "$readerBase/reader3/getBookSources"
    Assert-Success $before 'before-list'
    Write-Output 'Checking valid remote import...'
    $imported = Invoke-Json $client "$readerBase/reader3/saveFromRemoteSource" `
        @{ url = "$fixtureBase/source.json" }
    Assert-Success $imported 'remote-import'
    Write-Output "Import response: HTTP $($imported.status), isSuccess=$($imported.value.isSuccess), errorMsg=$($imported.value.errorMsg)"
    if (@($imported.value.data).Count -ne 1) { throw 'Remote response data shape changed' }
    $after = Invoke-Json $client "$readerBase/reader3/getBookSources"
    Assert-Success $after 'after-list'
    $found = @($after.value.data | Where-Object { $_.bookSourceUrl -eq $fixtureBase })
    if ($found.Count -ne 1) { throw "Imported source was not persisted exactly once" }
    if (@($before.value.data | Where-Object { $_.bookSourceUrl -eq $fixtureBase }).Count -ne 0) {
        throw 'Source existed before import'
    }

    if (-not $SkipInvalidProbe) {
        Write-Output 'Checking invalid remote content...'
        $invalid = Invoke-Json $client "$readerBase/reader3/saveFromRemoteSource" `
            @{ url = "$fixtureBase/book" }
        if ($invalid.status -ne 200 -or $invalid.value.isSuccess) {
            throw 'Non-JSON remote content was incorrectly marked as imported'
        }
    }
    Write-Output "PASS: remote import persisted one source; legacy data shape retained"
    if (-not $SkipInvalidProbe) { Write-Output "PASS: malformed content rejected" }
    Write-Output "Diagnostic logs: $runDir"
}
finally {
    if ($reader -and -not $reader.HasExited) { Stop-Process -Id $reader.Id -Force }
    if ($fixture -and -not $fixture.HasExited) { Stop-Process -Id $fixture.Id -Force }
    $client.Dispose()
    $handler.Dispose()
}
