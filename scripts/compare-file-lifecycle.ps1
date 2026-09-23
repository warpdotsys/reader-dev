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
if (-not $OutputPath) { $OutputPath = Join-Path $projectRoot "reports\file-lifecycle-diff-latest.json" }
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

function Read-Response([Net.Http.HttpResponseMessage]$Response) {
    $bytes = $Response.Content.ReadAsByteArrayAsync().GetAwaiter().GetResult()
    $type = [string]$Response.Content.Headers.ContentType
    $value = if ($type -match 'json') {
        [Text.Encoding]::UTF8.GetString($bytes) | ConvertFrom-Json -Depth 50
    } else { $null }
    return [pscustomobject]@{
        status = [int]$Response.StatusCode
        contentType = $type
        disposition = [string]$Response.Content.Headers.ContentDisposition
        cacheControl = [string]$Response.Headers.CacheControl
        bytes = $bytes
        value = $value
    }
}

function Invoke-Api([Net.Http.HttpClient]$Client, [string]$BaseUri, [string]$Method, [string]$Path, [object]$Body = $null) {
    $request = [Net.Http.HttpRequestMessage]::new([Net.Http.HttpMethod]::new($Method), $BaseUri + $Path)
    try {
        if ($null -ne $Body) {
            $json = ConvertTo-Json -InputObject $Body -Compress -Depth 10
            $request.Content = [Net.Http.StringContent]::new($json, [Text.Encoding]::UTF8, "application/json")
        }
        $response = $Client.SendAsync($request).GetAwaiter().GetResult()
        try { return Read-Response $response }
        finally { $response.Dispose() }
    }
    finally { $request.Dispose() }
}

function Invoke-Upload([Net.Http.HttpClient]$Client, [string]$BaseUri, [string]$Path, [byte[]]$Bytes) {
    $request = [Net.Http.HttpRequestMessage]::new([Net.Http.HttpMethod]::Post, $BaseUri + $Path)
    $multipart = [Net.Http.MultipartFormDataContent]::new()
    $fileContent = [Net.Http.ByteArrayContent]::new($Bytes)
    $fileContent.Headers.ContentType = [Net.Http.Headers.MediaTypeHeaderValue]::Parse('application/octet-stream')
    $multipart.Add($fileContent, 'file', '上传.bin')
    $request.Content = $multipart
    try {
        $response = $Client.SendAsync($request).GetAwaiter().GetResult()
        try { return Read-Response $response }
        finally { $response.Dispose() }
    }
    finally { $request.Dispose() }
}

function Assert-Success([object]$Response, [string]$Probe) {
    if ($Response.status -ne 200 -or -not $Response.value.isSuccess) {
        throw "$Probe failed: HTTP $($Response.status), $($Response.value.errorMsg)"
    }
}

function Format-Response([object]$Response, [string]$Probe) {
    $result = [ordered]@{
        probe = $Probe
        status = $Response.status
        contentType = $Response.contentType
    }
    if ($null -ne $Response.value) {
        $result.isSuccess = [bool]$Response.value.isSuccess
        $result.errorMsg = [string]$Response.value.errorMsg
        $data = $Response.value.data
        if ($Probe -in @('root-list', 'notes-list', 'notes-with-directory-list', 'upload-binary', 'notes-with-upload-list')) {
            $result.data = @($data | ForEach-Object {
                [ordered]@{
                    name = [string]$_.name
                    path = [string]$_.path
                    size = [long]$_.size
                    isDirectory = [bool]$_.isDirectory
                    lastModifiedIsNumber = $null -ne $_.lastModified -and
                        $_.lastModified -is [ValueType]
                }
            } | Sort-Object name)
        } elseif ($Probe -eq 'parse-preview') {
            $result.data = @($data | ForEach-Object {
                $book = [ordered]@{}
                foreach ($property in @($_.book.PSObject.Properties | Sort-Object Name)) {
                    $book[$property.Name] = if ($property.Name -in @('latestChapterTime', 'lastCheckTime', 'durChapterTime')) {
                        if ($property.Value -is [ValueType]) { 'number' } else { 'not-number' }
                    } else { $property.Value }
                }
                [ordered]@{
                    name = [string]$_.name
                    path = [string]$_.path
                    size = [long]$_.size
                    lastModifiedIsNumber = $_.lastModified -is [ValueType]
                    book = $book
                }
            } | Sort-Object name)
        } else {
            $result.data = $data
        }
    } else {
        $result.byteLength = $Response.bytes.Length
        $result.sha256 = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($Response.bytes))
        $result.disposition = $Response.disposition
        $result.cacheControl = $Response.cacheControl
    }
    return [pscustomobject]$result
}

function Wait-Reader([Net.Http.HttpClient]$Client, [string]$BaseUri, [Diagnostics.Process]$Process) {
    for ($attempt = 0; $attempt -lt 90; $attempt++) {
        if ($Process.HasExited) { throw "Reader exited during startup with code $($Process.ExitCode)" }
        try {
            $response = Invoke-Api $Client $BaseUri "GET" "/reader3/getSystemInfo"
            if ($response.status -eq 200 -and $response.value.isSuccess) { return }
        }
        catch { Start-Sleep -Milliseconds 500 }
    }
    throw "Reader did not become ready: $BaseUri"
}

function Run-Lifecycle([string]$Jar, [string]$WorkDir, [int]$Port, [string]$UserA, [string]$UserB, [string]$Password) {
    New-Item -ItemType Directory -Path $WorkDir -Force | Out-Null
    $baseUri = "http://127.0.0.1:$Port"
    $arguments = @(
        "-jar", ('"' + (Resolve-Path -LiteralPath $Jar).Path + '"'),
        "--reader.app.workDir=$WorkDir", "--reader.server.port=$Port",
        "--reader.app.secure=true", "--reader.app.licenseCheckEnabled=false",
        "--spring.profiles.active=prod"
    )
    $process = Start-Process -FilePath $java -ArgumentList $arguments -PassThru -WindowStyle Hidden `
        -RedirectStandardOutput (Join-Path $WorkDir "stdout.log") `
        -RedirectStandardError (Join-Path $WorkDir "stderr.log")
    $ready = New-Client
    $anonymous = New-Client
    $clientA = New-Client
    $clientB = New-Client
    $results = @()
    try {
        Wait-Reader $ready $baseUri $process
        $results += Format-Response (Invoke-Api $anonymous $baseUri "GET" "/reader3/file/list?home=__HOME__&path=/") "anonymous-list"

        foreach ($pair in @(@($clientA, $UserA), @($clientB, $UserB))) {
            $registration = Invoke-Api $pair[0] $baseUri "POST" "/reader3/login" `
                @{ username = $pair[1]; password = $Password; isLogin = $false }
            Assert-Success $registration "register"
            $login = Invoke-Api $pair[0] $baseUri "POST" "/reader3/login" `
                @{ username = $pair[1]; password = $Password; isLogin = $true }
            Assert-Success $login "login"
        }

        $filePath = "/notes/差分.txt"
        $queryPath = [uri]::EscapeDataString($filePath)
        $content = "跨版本文件内容`nUnicode ✓"
        $results += Format-Response (Invoke-Api $clientA $baseUri "GET" "/reader3/file/list?home=__HOME__&path=/") "root-list"
        $save = Invoke-Api $clientA $baseUri "POST" "/reader3/file/save" `
            @{ home = "__HOME__"; path = $filePath; content = $content }
        Assert-Success $save "save-text"
        $results += Format-Response $save "save-text"
        $get = Invoke-Api $clientA $baseUri "GET" "/reader3/file/get?home=__HOME__&path=$queryPath"
        Assert-Success $get "get-text"
        if ($get.value.data -cne $content) { throw "get-text did not preserve UTF-8 content" }
        $results += Format-Response $get "get-text"
        $notesList = Invoke-Api $clientA $baseUri "GET" "/reader3/file/list?home=__HOME__&path=/notes"
        Assert-Success $notesList "notes-list"
        if ($notesList.value.data.Count -ne 1 -or $notesList.value.data[0].name -cne "差分.txt") {
            throw "notes-list did not return the saved file"
        }
        $results += Format-Response $notesList "notes-list"
        $attachment = Invoke-Api $clientA $baseUri "GET" "/reader3/file/download?home=__HOME__&path=$queryPath"
        $stream = Invoke-Api $clientA $baseUri "GET" "/reader3/file/download?home=__HOME__&path=$queryPath&stream=1"
        foreach ($download in @($attachment, $stream)) {
            if ($download.status -ne 200 -or [Text.Encoding]::UTF8.GetString($download.bytes) -cne $content) {
                throw "download did not preserve file bytes"
            }
        }
        if (-not $attachment.disposition -or $stream.disposition) {
            throw "download Content-Disposition differs between attachment and stream"
        }
        $results += Format-Response $attachment "download-attachment"
        $results += Format-Response $stream "download-stream"
        $mkdir = Invoke-Api $clientA $baseUri "POST" "/reader3/file/mkdir" `
            @{ home = "__HOME__"; path = "/notes"; name = "子目录" }
        Assert-Success $mkdir "mkdir"
        $results += Format-Response $mkdir "mkdir"
        $results += Format-Response (Invoke-Api $clientA $baseUri "GET" "/reader3/file/list?home=__HOME__&path=/notes") "notes-with-directory-list"
        $preview = Invoke-Api $clientA $baseUri "GET" "/reader3/file/parse?home=__HOME__&path=/notes&import=0"
        Assert-Success $preview "parse-preview"
        if ($preview.value.data.Count -ne 1 -or $preview.value.data[0].name -cne "差分.txt" -or
            $preview.value.data[0].book.name -cne "差分") {
            throw "parse-preview did not return the local text book"
        }
        $results += Format-Response $preview "parse-preview"
        $binary = [byte[]]@(0, 1, 2, 13, 10, 127, 128, 255)
        $upload = Invoke-Upload $clientA $baseUri "/reader3/file/upload?home=__HOME__&path=/notes" $binary
        Assert-Success $upload "upload-binary"
        if ($upload.value.data.Count -ne 1 -or $upload.value.data[0].name -cne "上传.bin") {
            throw "upload-binary did not return the uploaded file"
        }
        $results += Format-Response $upload "upload-binary"
        $results += Format-Response (Invoke-Api $clientA $baseUri "GET" "/reader3/file/list?home=__HOME__&path=/notes") "notes-with-upload-list"
        $binaryPath = [uri]::EscapeDataString("/notes/上传.bin")
        $binaryDownload = Invoke-Api $clientA $baseUri "GET" "/reader3/file/download?home=__HOME__&path=$binaryPath"
        if ($binaryDownload.status -ne 200 -or
            -not [Linq.Enumerable]::SequenceEqual[byte]($binaryDownload.bytes, $binary)) {
            throw "Downloaded binary differs from the uploaded bytes"
        }
        $results += Format-Response $binaryDownload "download-uploaded-binary"
        $otherUserGet = Invoke-Api $clientB $baseUri "GET" "/reader3/file/get?home=__HOME__&path=$queryPath"
        if ($otherUserGet.value.isSuccess -or $otherUserGet.value.errorMsg -ne "路径不存在") {
            throw "The second user could access the first user's file"
        }
        $results += Format-Response $otherUserGet "other-user-get"
        $delete = Invoke-Api $clientA $baseUri "POST" "/reader3/file/delete" `
            @{ home = "__HOME__"; path = $filePath }
        Assert-Success $delete "delete-text"
        $results += Format-Response $delete "delete-text"
        $results += Format-Response (Invoke-Api $clientA $baseUri "GET" "/reader3/file/get?home=__HOME__&path=$queryPath") "get-after-delete"
        $deleteBinary = Invoke-Api $clientA $baseUri "POST" "/reader3/file/delete" `
            @{ home = "__HOME__"; path = "/notes/上传.bin" }
        Assert-Success $deleteBinary "delete-uploaded-binary"
        $results += Format-Response $deleteBinary "delete-uploaded-binary"

        $storageA = Join-Path $WorkDir "storage\data\$UserA\notes"
        $storageB = Join-Path $WorkDir "storage\data\$UserB\notes"
        $storage = [ordered]@{
            userANotesDirectory = Test-Path -LiteralPath $storageA -PathType Container
            userANotesFileDeleted = -not (Test-Path -LiteralPath (Join-Path $storageA "差分.txt"))
            userAUploadedFileDeleted = -not (Test-Path -LiteralPath (Join-Path $storageA "上传.bin"))
            userBNotesDirectoryAbsent = -not (Test-Path -LiteralPath $storageB)
        }
        if (-not $storage.userANotesDirectory -or -not $storage.userANotesFileDeleted -or
            -not $storage.userAUploadedFileDeleted -or -not $storage.userBNotesDirectoryAbsent) {
            throw "User file storage layout or isolation is incorrect"
        }
        return [pscustomobject]@{
            probes = $results
            storage = $storage
        }
    }
    finally {
        foreach ($client in @($ready, $anonymous, $clientA, $clientB)) { $client.Dispose() }
        if (-not $process.HasExited) { Stop-Process -Id $process.Id -Force }
        $process.Dispose()
    }
}

$runRoot = Join-Path $projectRoot (".tools\file-diff-" + [guid]::NewGuid().ToString("N"))
$userA = "fileprobe" + [guid]::NewGuid().ToString("N").Substring(0, 10)
$userB = "otherprobe" + [guid]::NewGuid().ToString("N").Substring(0, 10)
$password = "Probe-" + [guid]::NewGuid().ToString("N")
$originalPort = Get-FreePort
$restoredPort = Get-FreePort
while ($originalPort -eq $restoredPort) { $restoredPort = Get-FreePort }

try {
    $original = Run-Lifecycle $OriginalJar (Join-Path $runRoot "original") $originalPort $userA $userB $password
    $restored = Run-Lifecycle $RestoredJar (Join-Path $runRoot "restored") $restoredPort $userA $userB $password
    $comparisons = foreach ($index in 0..($original.probes.Count - 1)) {
        $left = $original.probes[$index]
        $right = $restored.probes[$index]
        [pscustomobject]@{
            probe = $left.probe
            semanticEqual = (ConvertTo-Json $left -Compress -Depth 20) -ceq (ConvertTo-Json $right -Compress -Depth 20)
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
    $report | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $OutputPath -Encoding utf8
    $comparisons | Select-Object probe, semanticEqual | Format-Table -AutoSize
    Write-Host "Report: $OutputPath"
    Write-Host "Isolated run data: $runRoot"
    if (@($comparisons | Where-Object { -not $_.semanticEqual }).Count -ne 0) {
        throw "File lifecycle differs from original JAR"
    }
    if ((ConvertTo-Json $original.storage -Compress -Depth 10) -cne
        (ConvertTo-Json $restored.storage -Compress -Depth 10)) {
        throw "File storage layout differs from original JAR"
    }
}
finally {
    $password = $null
}
