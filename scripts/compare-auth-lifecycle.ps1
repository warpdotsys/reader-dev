param(
    [string]$JavaHome = "",
    [string]$OriginalJar = "",
    [string]$RestoredJar = "",
    [string]$OutputPath = ""
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
if (-not $JavaHome) { $JavaHome = Join-Path $projectRoot ".tools\jdk-11.0.8" }
if (-not $OriginalJar) { $OriginalJar = Join-Path $projectRoot "reference\original\reader-pro-3.2.14.original.jar" }
if (-not $RestoredJar) { $RestoredJar = Join-Path $projectRoot "build\libs\reader-4.0.7.jar" }
if (-not $OutputPath) { $OutputPath = Join-Path $projectRoot "reports\auth-lifecycle-diff-latest.json" }

$java = Join-Path $JavaHome "bin\java.exe"
foreach ($path in @($java, $OriginalJar, $RestoredJar)) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Required file not found: $path" }
}

function Get-FreePort {
    $listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
    $listener.Start()
    try { return ([Net.IPEndPoint]$listener.LocalEndpoint).Port }
    finally { $listener.Stop() }
}

function New-Client {
    $handler = [Net.Http.HttpClientHandler]::new()
    $handler.UseCookies = $true
    $handler.CookieContainer = [Net.CookieContainer]::new()
    $client = [Net.Http.HttpClient]::new($handler)
    $client.Timeout = [TimeSpan]::FromSeconds(20)
    return $client
}

function Invoke-ReaderApi([Net.Http.HttpClient]$Client, [string]$BaseUri, [string]$Method, [string]$Path, [object]$Body = $null) {
    $request = [Net.Http.HttpRequestMessage]::new([Net.Http.HttpMethod]::new($Method), $BaseUri + $Path)
    try {
        if ($null -ne $Body) {
            $json = ConvertTo-Json -InputObject $Body -Compress -Depth 8
            $request.Content = [Net.Http.StringContent]::new($json, [Text.Encoding]::UTF8, "application/json")
        }
        $response = $Client.SendAsync($request).GetAwaiter().GetResult()
        try {
            $text = $response.Content.ReadAsStringAsync().GetAwaiter().GetResult()
            $data = $text | ConvertFrom-Json -Depth 50
            return [pscustomobject]@{
                status = [int]$response.StatusCode
                contentType = [string]$response.Content.Headers.ContentType
                value = $data
            }
        }
        finally { $response.Dispose() }
    }
    finally { $request.Dispose() }
}

function Summarize([object]$Response, [string]$ProbeName) {
    $data = $Response.value.data
    $summary = [ordered]@{
        probe = $ProbeName
        status = $Response.status
        contentType = $Response.contentType
        isSuccess = [bool]$Response.value.isSuccess
        errorMsg = [string]$Response.value.errorMsg
        dataType = if ($null -eq $data) { "null" } elseif ($data -is [Array]) { "array" } else { $data.GetType().Name }
    }
    if ($data -is [Array]) { $summary.dataCount = $data.Count }
    if ($ProbeName -in @("register", "login-correct")) {
        $summary.userFields = @($data.PSObject.Properties.Name | Sort-Object)
        $summary.tokenHasUserPrefix = [bool]($data.accessToken -match '^[a-zA-Z0-9]+:.+')
    }
    if ($ProbeName -in @("user-info-cookie", "user-info-token")) {
        $summary.userInfoPresent = $null -ne $data.userInfo
        $summary.secure = [bool]$data.secure
    }
    if ($ProbeName -eq "get-user-config") {
        $summary.configFields = @($data.PSObject.Properties.Name | Sort-Object)
        $summary.probeMarker = [string]$data.probe
        $summary.fontSize = [int]$data.options.fontSize
        $summary.updateTimeIsNumber = $data.'@updateTime' -is [long]
    }
    return [pscustomobject]$summary
}

function Wait-Reader([Net.Http.HttpClient]$Client, [string]$BaseUri, [Diagnostics.Process]$Process) {
    for ($attempt = 0; $attempt -lt 90; $attempt++) {
        if ($Process.HasExited) { throw "Reader exited during startup with code $($Process.ExitCode)" }
        try {
            $response = Invoke-ReaderApi $Client $BaseUri "GET" "/reader3/getSystemInfo"
            if ($response.status -eq 200 -and $response.value.isSuccess) { return }
        }
        catch { Start-Sleep -Milliseconds 500 }
    }
    throw "Reader did not become ready: $BaseUri"
}

function Run-Lifecycle([string]$Jar, [string]$WorkDir, [int]$Port, [string]$Username, [string]$Password) {
    New-Item -ItemType Directory -Path $WorkDir -Force | Out-Null
    $baseUri = "http://127.0.0.1:$Port"
    $stdout = Join-Path $WorkDir "stdout.log"
    $stderr = Join-Path $WorkDir "stderr.log"
    $arguments = @(
        "-jar", ('"' + (Resolve-Path -LiteralPath $Jar).Path + '"'),
        "--reader.app.workDir=$WorkDir", "--reader.server.port=$Port",
        "--reader.app.secure=true", "--reader.app.licenseCheckEnabled=false",
        "--spring.profiles.active=prod"
    )
    $process = Start-Process -FilePath $java -ArgumentList $arguments -PassThru -WindowStyle Hidden `
        -RedirectStandardOutput $stdout -RedirectStandardError $stderr
    $readyClient = New-Client
    $registerClient = New-Client
    $loginClient = New-Client
    $tokenClient = New-Client
    $invalidClient = New-Client
    try {
        Wait-Reader $readyClient $baseUri $process
        $results = @()
        $body = @{ username = $Username; password = $Password; isLogin = $true }
        $results += Summarize (Invoke-ReaderApi $loginClient $baseUri "POST" "/reader3/login" $body) "login-unknown"

        $body.isLogin = $false
        $registered = Invoke-ReaderApi $registerClient $baseUri "POST" "/reader3/login" $body
        $results += Summarize $registered "register"
        if (-not $registered.value.isSuccess) { throw "Registration failed in isolated workdir: $($registered.value.errorMsg)" }

        $body.isLogin = $true
        $wrongBody = @{ username = $Username; password = ($Password + "wrong"); isLogin = $true }
        $results += Summarize (Invoke-ReaderApi $loginClient $baseUri "POST" "/reader3/login" $wrongBody) "login-wrong-password"
        $loggedIn = Invoke-ReaderApi $loginClient $baseUri "POST" "/reader3/login" $body
        $results += Summarize $loggedIn "login-correct"
        if (-not $loggedIn.value.isSuccess) { throw "Login failed in isolated workdir: $($loggedIn.value.errorMsg)" }

        $token = [string]$loggedIn.value.data.accessToken
        if (-not $token.StartsWith($Username + ":")) { throw "Login response lacks the expected token format" }
        $results += Summarize (Invoke-ReaderApi $loginClient $baseUri "GET" "/reader3/getUserInfo") "user-info-cookie"
        $encodedToken = [uri]::EscapeDataString($token)
        $results += Summarize (Invoke-ReaderApi $tokenClient $baseUri "GET" "/reader3/getUserInfo?accessToken=$encodedToken") "user-info-token"
        $results += Summarize (Invoke-ReaderApi $tokenClient $baseUri "GET" "/reader3/getBookshelf?accessToken=$encodedToken") "bookshelf-token"
        $configBody = @{ probe = "source-parity"; options = @{ fontSize = 16; dark = $false } }
        $results += Summarize (Invoke-ReaderApi $loginClient $baseUri "POST" "/reader3/saveUserConfig" $configBody) "save-user-config"
        $results += Summarize (Invoke-ReaderApi $tokenClient $baseUri "GET" "/reader3/getUserConfig?accessToken=$encodedToken") "get-user-config"
        $results += Summarize (Invoke-ReaderApi $loginClient $baseUri "POST" "/reader3/logout?accessToken=$encodedToken" @{}) "logout"
        $results += Summarize (Invoke-ReaderApi $invalidClient $baseUri "GET" "/reader3/getBookshelf?accessToken=$encodedToken") "bookshelf-revoked-token"

        $configFile = Join-Path $WorkDir "storage\data\$Username\userConfig.json"
        $storedConfig = if (Test-Path -LiteralPath $configFile) {
            Get-Content -LiteralPath $configFile -Raw | ConvertFrom-Json -Depth 20
        } else { $null }

        return [pscustomobject]@{
            probes = $results
            storage = [pscustomobject]@{
                usersFile = Test-Path -LiteralPath (Join-Path $WorkDir "storage\data\users.json")
                userNamespace = Test-Path -LiteralPath (Join-Path $WorkDir "storage\data\$Username")
                configFile = $null -ne $storedConfig
                configFields = if ($storedConfig) { @($storedConfig.PSObject.Properties.Name | Sort-Object) } else { @() }
                configMarker = if ($storedConfig) { [string]$storedConfig.probe } else { "" }
            }
        }
    }
    finally {
        foreach ($client in @($readyClient, $registerClient, $loginClient, $tokenClient, $invalidClient)) { $client.Dispose() }
        if (-not $process.HasExited) { Stop-Process -Id $process.Id -Force }
        $process.Dispose()
    }
}

$runRoot = Join-Path $projectRoot (".tools\auth-diff-" + [guid]::NewGuid().ToString("N"))
$username = "probe" + [guid]::NewGuid().ToString("N").Substring(0, 12)
$password = "Probe-" + [guid]::NewGuid().ToString("N")
$originalPort = Get-FreePort
$restoredPort = Get-FreePort
while ($originalPort -eq $restoredPort) { $restoredPort = Get-FreePort }

try {
    $original = Run-Lifecycle $OriginalJar (Join-Path $runRoot "original") $originalPort $username $password
    $restored = Run-Lifecycle $RestoredJar (Join-Path $runRoot "restored") $restoredPort $username $password
    $comparisons = foreach ($index in 0..($original.probes.Count - 1)) {
        $left = $original.probes[$index]
        $right = $restored.probes[$index]
        [pscustomobject]@{
            probe = $left.probe
            semanticEqual = (ConvertTo-Json $left -Compress -Depth 10) -ceq (ConvertTo-Json $right -Compress -Depth 10)
            original = $left
            restored = $right
        }
    }
    $report = [ordered]@{
        originalJarSha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $OriginalJar).Hash
        restoredJarSha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $RestoredJar).Hash
        comparisons = @($comparisons)
        storage = [ordered]@{ original = $original.storage; restored = $restored.storage }
    }
    $report | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $OutputPath -Encoding utf8
    $comparisons | Select-Object probe, semanticEqual | Format-Table -AutoSize
    Write-Host "Report: $OutputPath"
    Write-Host "Isolated run data: $runRoot"
    if (@($comparisons | Where-Object { -not $_.semanticEqual }).Count -ne 0) {
        throw "Authentication lifecycle differs from original JAR"
    }
    if ((ConvertTo-Json $original.storage -Compress -Depth 5) -cne
        (ConvertTo-Json $restored.storage -Compress -Depth 5)) {
        throw "Storage layout differs from original JAR"
    }
}
finally {
    $password = $null
}
