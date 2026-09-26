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

function Format-Book([object]$Book) {
    if ($null -eq $Book) { return $null }
    $result = [ordered]@{}
    foreach ($property in @($Book.PSObject.Properties | Sort-Object Name)) {
        $result[$property.Name] = if ($property.Name -in @('latestChapterTime', 'lastCheckTime', 'durChapterTime')) {
            if ($property.Value -is [ValueType]) { 'number' } else { 'not-number' }
        } else { $property.Value }
    }
    return $result
}

function Format-Epub([byte[]]$Bytes) {
    $stream = [IO.MemoryStream]::new($Bytes, $false)
    try {
        $archive = [IO.Compression.ZipArchive]::new($stream, [IO.Compression.ZipArchiveMode]::Read, $true)
        try {
            $entries = @($archive.Entries | Where-Object { -not $_.FullName.EndsWith('/') } | ForEach-Object {
                $entry = $_
                $entryStream = $entry.Open()
                try {
                    $content = [IO.MemoryStream]::new()
                    try {
                        $entryStream.CopyTo($content)
                        $entryBytes = $content.ToArray()
                    } finally { $content.Dispose() }
                } finally { $entryStream.Dispose() }
                [ordered]@{
                    path = $entry.FullName
                    length = $entryBytes.Length
                    sha256 = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($entryBytes))
                    normalizedSha256 = if ($entry.FullName -in @('OEBPS/toc.ncx', 'OEBPS/content.opf')) {
                        $normalized = [Text.Encoding]::UTF8.GetString($entryBytes)
                        $normalized = [regex]::Replace($normalized,
                            '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}', '<uuid>')
                        $normalized = [regex]::Replace($normalized, '\b20\d{2}-\d{2}-\d{2}\b', '<date>')
                        [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData(
                            [Text.Encoding]::UTF8.GetBytes($normalized)))
                    } else { $null }
                }
            } | Sort-Object path)
            if (-not @($entries | Where-Object { $_.path -eq 'mimetype' }).Count -or
                -not @($entries | Where-Object { $_.path -eq 'META-INF/container.xml' }).Count -or
                -not @($entries | Where-Object { $_.path -match '\.opf$' }).Count -or
                @($entries | Where-Object { $_.path -match '\.html$|\.xhtml$' }).Count -lt 2) {
                throw 'Exported EPUB lacks required container, metadata, or HTML entries'
            }
            return $entries
        } finally { $archive.Dispose() }
    } finally { $stream.Dispose() }
}

function Test-Epub-Encoding-Divergence([object]$Original, [object]$Restored) {
    if ($Original.status -ne 200 -or $Restored.status -ne 200 -or
        $Original.contentType -cne 'application/epub+zip' -or
        $Original.contentType -cne $Restored.contentType -or
        $Original.disposition -cne $Restored.disposition -or
        $Original.cacheControl -cne $Restored.cacheControl -or
        $Original.epubEntries.Count -ne $Restored.epubEntries.Count) { return $false }

    # This fixed synthetic TXT fixture exposes the old JAR's platform-default
    # HTML encoding and corrupted TOC titles. Accept only these exact outputs;
    # all other EPUB entries must match by uncompressed bytes.
    $expected = @{
        'OEBPS/Text/intro.html' = @('79E5B410CAC3F236AEE3ACF64C4FAA77B32D40018ECD54528D8031FC3E62C23E',
            'E6BAA8A38DF3F9FB17FB943094D89644478431C1B916508CE782366C89370202')
        'OEBPS/Text/cover.html' = @('5E3D3E7DBC830CC7970F9F11C43818CECB37ADA1B9B6805F806B60DC3940843B',
            'E631C94AE72BF98A912146D5844C94C174D6CC23492A722564283C99AB64C059')
        'OEBPS/Text/chapter_0.html' = @('7916DFB070CE86EA1EE13F86A58C437C4398252A96F8ACE65873AFF89085B00C',
            'CABA449A9A90D440F310005A3E31ECA33D463FF25A11EA98C13BF919E059C1B7')
        'OEBPS/Text/chapter_1.html' = @('D54390E75766990C87277CD1E5C4DE91EB42C186CF8E3E0BAF356CCBF4679181',
            '1E74F3AFAE9CA0EE339C29E5E99A84E55006F4160BC1F04570770E3602780E9F')
        'OEBPS/toc.ncx' = @('997560D6FF7D72DD7F75612661E2AF33BDF027EE64B24BF515F4B263FEBC6260',
            '9702C985E4500ADC92967D0C2032CC8E10C0DA53A03DDD1B0C278FB54528C3AA')
    }
    foreach ($index in 0..($Original.epubEntries.Count - 1)) {
        $left = $Original.epubEntries[$index]
        $right = $Restored.epubEntries[$index]
        if ($left.path -cne $right.path) { return $false }
        if ($left.path -eq 'OEBPS/content.opf') {
            if ($left.normalizedSha256 -cne $right.normalizedSha256) { return $false }
        } elseif ($expected.ContainsKey($left.path)) {
            $oldHash = if ($left.path -eq 'OEBPS/toc.ncx') { $left.normalizedSha256 } else { $left.sha256 }
            $newHash = if ($left.path -eq 'OEBPS/toc.ncx') { $right.normalizedSha256 } else { $right.sha256 }
            if ($oldHash -cne $expected[$left.path][0] -or $newHash -cne $expected[$left.path][1]) {
                return $false
            }
        } elseif ($left.length -ne $right.length -or $left.sha256 -cne $right.sha256) {
            return $false
        }
    }
    return $true
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
        } elseif ($Probe -in @('reading-shelf', 'reading-shelf-after-progress')) {
            $result.data = @($data | ForEach-Object { Format-Book $_ })
        } elseif ($Probe -eq 'reading-book-info') {
            $result.data = Format-Book $data
        } else {
            $result.data = $data
        }
    } else {
        $result.byteLength = $Response.bytes.Length
        $result.sha256 = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($Response.bytes))
        $result.disposition = $Response.disposition
        $result.cacheControl = $Response.cacheControl
        if ($Probe -eq 'export-local-txt-as-epub') {
            $result.epubEntries = @(Format-Epub $Response.bytes)
        }
    }
    return [pscustomobject]$result
}

function Wait-Reader([Net.Http.HttpClient]$Client, [string]$BaseUri, [Diagnostics.Process]$Process) {
    $deadline = [DateTime]::UtcNow.AddSeconds(60)
    while ([DateTime]::UtcNow -lt $deadline) {
        if ($Process.HasExited) { throw "Reader exited during startup with code $($Process.ExitCode)" }
        try {
            $response = Invoke-Api $Client $BaseUri "GET" "/reader3/getSystemInfo"
            if ($response.status -eq 200 -and $response.value.isSuccess) { return }
        }
        catch { }
        Start-Sleep -Milliseconds 500
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

        $readingText = "第一章 开始`n这是第一章的正文。`n第二章 继续`n这是第二章的正文。"
        $readingPath = "/reading/readable.txt"
        $readingSave = Invoke-Api $clientA $baseUri "POST" "/reader3/file/save" `
            @{ home = "__HOME__"; path = $readingPath; content = $readingText }
        Assert-Success $readingSave "reading-save"
        $results += Format-Response $readingSave "reading-save"
        $readingImport = Invoke-Api $clientA $baseUri "GET" "/reader3/file/parse?home=__HOME__&path=/reading&import=1"
        Assert-Success $readingImport "reading-import"
        $results += Format-Response $readingImport "reading-import"
        $shelf = Invoke-Api $clientA $baseUri "GET" "/reader3/getBookshelf"
        Assert-Success $shelf "reading-shelf"
        if ($shelf.value.data.Count -ne 1) { throw "reading-shelf did not contain exactly one imported book" }
        $results += Format-Response $shelf "reading-shelf"
        $bookUrl = [uri]::EscapeDataString([string]$shelf.value.data[0].bookUrl)
        $bookInfo = Invoke-Api $clientA $baseUri "GET" "/reader3/getBookInfo?url=$bookUrl"
        if ($bookInfo.value.isSuccess -and
            $bookInfo.value.data.bookUrl -cne $shelf.value.data[0].bookUrl) {
            throw "reading-book-info returned a different book"
        }
        $results += Format-Response $bookInfo "reading-book-info"
        $chapters = Invoke-Api $clientA $baseUri "GET" "/reader3/getChapterList?url=$bookUrl"
        Assert-Success $chapters "reading-chapters"
        if ($chapters.value.data.Count -ne 2) { throw "reading-chapters did not return two chapters" }
        $results += Format-Response $chapters "reading-chapters"
        $readingContent = Invoke-Api $clientA $baseUri "GET" "/reader3/getBookContent?url=$bookUrl&index=0"
        Assert-Success $readingContent "reading-content"
        if ($readingContent.value.data -cne "第一章 开始`n这是第一章的正文。`n") {
            throw "reading-content differs from the original TXT chapter format"
        }
        $results += Format-Response $readingContent "reading-content"
        $results += Format-Response (Invoke-Api $anonymous $baseUri "GET" "/reader3/exportBook?url=$bookUrl") "export-anonymous"
        $results += Format-Response (Invoke-Api $clientA $baseUri "GET" "/reader3/exportBook") "export-missing-url"
        $originalTxtBytes = [Text.Encoding]::UTF8.GetBytes($readingText)
        foreach ($exportProbe in @(
            @{ name = 'export-local-txt-get'; method = 'GET'; path = "/reader3/exportBook?url=$bookUrl&isEpub=0"; body = $null },
            @{ name = 'export-local-txt-post'; method = 'POST'; path = '/reader3/exportBook'; body = @{ url = [string]$shelf.value.data[0].bookUrl; isEpub = 0 } }
        )) {
            $export = Invoke-Api $clientA $baseUri $exportProbe.method $exportProbe.path $exportProbe.body
            if ($export.status -ne 200 -or $export.bytes.Length -ne $originalTxtBytes.Length -or
                [Convert]::ToHexString($export.bytes) -cne [Convert]::ToHexString($originalTxtBytes) -or
                -not $export.disposition.StartsWith('attachment;')) {
                throw "$($exportProbe.name) did not return the original local TXT bytes as an attachment"
            }
            $results += Format-Response $export $exportProbe.name
        }
        $epubExport = Invoke-Api $clientA $baseUri 'GET' "/reader3/exportBook?url=$bookUrl&isEpub=1"
        if ($epubExport.status -ne 200 -or -not $epubExport.disposition.StartsWith('attachment;') -or
            $epubExport.bytes.Length -lt 100) {
            throw 'export-local-txt-as-epub did not return an EPUB attachment'
        }
        $results += Format-Response $epubExport 'export-local-txt-as-epub'
        $progressSave = Invoke-Api $clientA $baseUri "POST" "/reader3/saveBookProgress" `
            @{ url = [string]$shelf.value.data[0].bookUrl; index = 0 }
        Assert-Success $progressSave "reading-progress-save"
        $results += Format-Response $progressSave "reading-progress-save"
        $progressShelf = Invoke-Api $clientA $baseUri "GET" "/reader3/getBookshelf"
        Assert-Success $progressShelf "reading-shelf-after-progress"
        if ($progressShelf.value.data[0].durChapterIndex -ne 0 -or
            $progressShelf.value.data[0].totalChapterNum -ne 2) {
            throw "reading progress was not persisted on the bookshelf"
        }
        $results += Format-Response $progressShelf "reading-shelf-after-progress"
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
        $leftJson = ConvertTo-Json $left -Compress -Depth 25
        $rightJson = ConvertTo-Json $right -Compress -Depth 25
        $equal = $leftJson -ceq $rightJson
        $accepted = $false
        if (-not $equal -and $left.probe -eq 'reading-book-info') {
            $accepted = $left.status -eq 200 -and -not $left.isSuccess -and
                $left.errorMsg -eq '未配置书源' -and $right.status -eq 200 -and
                $right.isSuccess -and $right.data.origin -eq 'loc_book'
        }
        if (-not $equal -and $left.probe -eq 'reading-shelf-after-progress') {
            $titlePattern = '"(latestChapterTitle|durChapterTitle)":"[^"]*"'
            $leftWithoutTitles = [regex]::Replace($leftJson, $titlePattern, '"$1":"<normalized-title>"')
            $rightWithoutTitles = [regex]::Replace($rightJson, $titlePattern, '"$1":"<normalized-title>"')
            $accepted = $leftWithoutTitles -ceq $rightWithoutTitles -and
                $right.data[0].latestChapterTitle -ceq '第二章 继续' -and
                $right.data[0].durChapterTitle -ceq '第一章 开始'
        }
        if (-not $equal -and $left.probe -eq 'export-local-txt-as-epub') {
            $accepted = Test-Epub-Encoding-Divergence $left $right
        }
        [pscustomobject]@{
            probe = $left.probe
            semanticEqual = $equal
            acceptedDivergence = $accepted
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
    $comparisons | Select-Object probe, semanticEqual, acceptedDivergence | Format-Table -AutoSize
    Write-Host "Report: $OutputPath"
    Write-Host "Isolated run data: $runRoot"
    if (@($comparisons | Where-Object { -not $_.semanticEqual -and -not $_.acceptedDivergence }).Count -ne 0) {
        throw "Unreviewed file or reading lifecycle difference from original JAR"
    }
    if ((ConvertTo-Json $original.storage -Compress -Depth 10) -cne
        (ConvertTo-Json $restored.storage -Compress -Depth 10)) {
        throw "File storage layout differs from original JAR"
    }
}
finally {
    $password = $null
}
