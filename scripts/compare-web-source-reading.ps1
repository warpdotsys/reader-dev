param(
    [string]$JavaHome = "",
    [string]$OriginalJar = "",
    [string]$RestoredJar = "",
    [string]$OutputPath = ""
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
if (-not $JavaHome) { $JavaHome = Join-Path $root ".tools\jdk-11.0.8" }
if (-not $OriginalJar) { $OriginalJar = Join-Path $root "reference\original\reader-pro-3.2.14.original.jar" }
if (-not $RestoredJar) { $RestoredJar = Join-Path $root "build\libs\reader-4.0.7.jar" }
if (-not $OutputPath) { $OutputPath = Join-Path $root "reports\web-source-reading-diff-latest.json" }
$java = Join-Path $JavaHome "bin\java.exe"
$python = (Get-Command python -ErrorAction Stop).Source
foreach ($path in @($java, $python, $OriginalJar, $RestoredJar)) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Required file not found: $path" }
}

function Get-FreePort {
    $listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
    $listener.Start()
    try { return ([Net.IPEndPoint]$listener.LocalEndpoint).Port }
    finally { $listener.Stop() }
}

function Invoke-Json([Net.Http.HttpClient]$Client, [string]$Base, [string]$Path, [object]$Body = $null) {
    $method = if ($null -eq $Body) { [Net.Http.HttpMethod]::Get } else { [Net.Http.HttpMethod]::Post }
    $request = [Net.Http.HttpRequestMessage]::new($method, $Base + $Path)
    try {
        if ($null -ne $Body) {
            $request.Content = [Net.Http.StringContent]::new(
                (ConvertTo-Json -InputObject $Body -Compress -Depth 15), [Text.Encoding]::UTF8, "application/json")
        }
        $response = $Client.SendAsync($request).GetAwaiter().GetResult()
        try {
            $raw = $response.Content.ReadAsStringAsync().GetAwaiter().GetResult()
            return [pscustomobject]@{
                status = [int]$response.StatusCode
                contentType = [string]$response.Content.Headers.ContentType
                value = $raw | ConvertFrom-Json -Depth 50
            }
        }
        finally { $response.Dispose() }
    }
    finally { $request.Dispose() }
}

function New-Client {
    $handler = [Net.Http.HttpClientHandler]::new()
    $handler.UseCookies = $true
    $handler.CookieContainer = [Net.CookieContainer]::new()
    $client = [Net.Http.HttpClient]::new($handler)
    $client.Timeout = [TimeSpan]::FromSeconds(25)
    return $client
}

function Assert-Success([object]$Response, [string]$Probe) {
    if ($Response.status -ne 200 -or -not $Response.value.isSuccess) {
        throw "$Probe failed: HTTP $($Response.status), $($Response.value.errorMsg)"
    }
}

function Wait-Ready([Net.Http.HttpClient]$Client, [string]$Base, [Diagnostics.Process]$Process) {
    $deadline = [DateTime]::UtcNow.AddSeconds(60)
    while ([DateTime]::UtcNow -lt $deadline) {
        if ($Process.HasExited) { throw "Reader exited during startup with code $($Process.ExitCode)" }
        try {
            $response = Invoke-Json $Client $Base "/reader3/getSystemInfo"
            if ($response.status -eq 200 -and $response.value.isSuccess) { return }
        } catch { }
        Start-Sleep -Milliseconds 500
    }
    throw "Reader did not become ready: $Base"
}

function Get-FieldShape([object]$Item) {
    if ($null -eq $Item) { return $null }
    $shape = [ordered]@{}
    foreach ($property in @($Item.PSObject.Properties | Sort-Object Name)) {
        $value = $property.Value
        $shape[$property.Name] = if ($null -eq $value) { 'null' }
            elseif ($value -is [string] -and $value.Length -eq 0) { 'empty-string' }
            elseif ($value -is [bool]) { "bool:$value" }
            elseif ($value -is [ValueType] -and $value -eq 0) { 'zero' }
            elseif ($value -is [array] -and $value.Count -eq 0) { 'empty-array' }
            else { 'non-default' }
    }
    return $shape
}

function Select-Result([object]$Response, [string]$Name) {
    $data = $Response.value.data
    $fieldShape = $null
    if ($Name -in @('search', 'chapter-list', 'shelf')) {
        $fieldShape = @($data | ForEach-Object { Get-FieldShape $_ })
    } elseif ($Name -in @('book-info', 'book-save')) {
        $fieldShape = Get-FieldShape $data
    }
    if ($Name -eq 'source-save') {
        $data = [string]$data
    } elseif ($Name -eq 'search') {
        $data = @($data | ForEach-Object {
            [ordered]@{ name = $_.name; author = $_.author; bookUrl = $_.bookUrl; origin = $_.origin }
        })
    } elseif ($Name -eq 'book-info') {
        $data = [ordered]@{
            name = $data.name; author = $data.author; bookUrl = $data.bookUrl
            tocUrl = $data.tocUrl; origin = $data.origin
        }
    } elseif ($Name -eq 'chapter-list') {
        $data = @($data | ForEach-Object {
            [ordered]@{ title = $_.title; url = $_.url; index = $_.index }
        })
    } elseif ($Name -eq 'shelf') {
        $data = @($data | ForEach-Object {
            [ordered]@{
                name = $_.name; author = $_.author; bookUrl = $_.bookUrl
                tocUrl = $_.tocUrl; origin = $_.origin
            }
        })
    } elseif ($Name -eq 'book-save') {
        $data = [ordered]@{
            name = $data.name; author = $data.author; bookUrl = $data.bookUrl
            tocUrl = $data.tocUrl; origin = $data.origin
        }
    }
    return [pscustomobject]@{
        probe = $Name
        status = $Response.status
        contentType = $Response.contentType
        isSuccess = [bool]$Response.value.isSuccess
        errorMsg = [string]$Response.value.errorMsg
        data = $data
        fieldShape = $fieldShape
    }
}

function Run-Reader([string]$Jar, [string]$Dir, [int]$Port, [object]$Source,
    [string]$User, [string]$Password, [string]$BookUrl) {
    New-Item -ItemType Directory -Path $Dir -Force | Out-Null
    $base = "http://127.0.0.1:$Port"
    $arguments = @(
        '-jar', ('"' + (Resolve-Path -LiteralPath $Jar).Path + '"'),
        "--reader.app.workDir=$Dir", "--reader.server.port=$Port",
        '--reader.app.secure=true', '--reader.app.licenseCheckEnabled=false',
        '--spring.profiles.active=prod'
    )
    $process = Start-Process -FilePath $java -ArgumentList $arguments -PassThru -WindowStyle Hidden `
        -RedirectStandardOutput (Join-Path $Dir 'stdout.log') `
        -RedirectStandardError (Join-Path $Dir 'stderr.log')
    $client = New-Client
    try {
        Wait-Ready $client $base $process
        Assert-Success (Invoke-Json $client $base '/reader3/login' `
            @{ username = $User; password = $Password; isLogin = $false }) 'register'
        Assert-Success (Invoke-Json $client $base '/reader3/login' `
            @{ username = $User; password = $Password; isLogin = $true }) 'login'
        $results = @()
        $sourceSave = Invoke-Json $client $base '/reader3/saveBookSource' $Source
        Assert-Success $sourceSave 'source-save'
        $results += Select-Result $sourceSave 'source-save'
        $search = Invoke-Json $client $base '/reader3/searchBook' `
            @{ key = '差分'; page = 1; bookSourceUrl = $Source.bookSourceUrl }
        Assert-Success $search 'search'
        if (@($search.value.data).Count -ne 1 -or
            $search.value.data[0].name -cne '差分测试书' -or
            $search.value.data[0].bookUrl -cne $BookUrl) {
            throw 'Search result did not match the fixture'
        }
        $results += Select-Result $search 'search'
        $info = Invoke-Json $client $base '/reader3/getBookInfo' `
            @{ url = $BookUrl; bookSource = $Source }
        Assert-Success $info 'book-info'
        if ($info.value.data.name -cne '差分测试书' -or
            $info.value.data.tocUrl -cne ($BookUrl -replace '/book$', '/toc')) {
            throw 'Book information did not match the fixture'
        }
        $results += Select-Result $info 'book-info'
        $save = Invoke-Json $client $base '/reader3/saveBook' $info.value.data
        Assert-Success $save 'book-save'
        $results += Select-Result $save 'book-save'
        $shelf = Invoke-Json $client $base '/reader3/getBookshelf'
        Assert-Success $shelf 'shelf'
        if (@($shelf.value.data).Count -ne 1) { throw 'Shelf count was not one' }
        $results += Select-Result $shelf 'shelf'
        $chapters = Invoke-Json $client $base '/reader3/getChapterList' `
            @{ url = $BookUrl; refresh = 1 }
        Assert-Success $chapters 'chapter-list'
        if (@($chapters.value.data).Count -ne 2 -or
            $chapters.value.data[0].title -cne '第一章 起点' -or
            $chapters.value.data[1].title -cne '第二章 继续') {
            throw 'Chapter list did not match the fixture'
        }
        $results += Select-Result $chapters 'chapter-list'
        foreach ($index in @(0, 1)) {
            $content = Invoke-Json $client $base '/reader3/getBookContent' `
                @{ url = $BookUrl; index = $index; refresh = 1 }
            Assert-Success $content "content-$index"
            $expected = if ($index -eq 0) { '第一段，中文与 UTF-8。' } else { '终章内容固定。' }
            if (-not ([string]$content.value.data).Contains($expected)) {
                throw "Content $index did not match the fixture"
            }
            $results += Select-Result $content "content-$index"
        }
        return ,$results
    }
    finally {
        $client.Dispose()
        if (-not $process.HasExited) { Stop-Process -Id $process.Id -Force }
        $process.Dispose()
    }
}

$runRoot = Join-Path $root ('.tools\web-source-diff-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $runRoot -Force | Out-Null
$fixturePort = Get-FreePort
$originalPort = Get-FreePort
$restoredPort = Get-FreePort
while (@($fixturePort, $originalPort, $restoredPort | Select-Object -Unique).Count -ne 3) {
    $fixturePort = Get-FreePort; $originalPort = Get-FreePort; $restoredPort = Get-FreePort
}
$fixtureBase = "http://127.0.0.1:$fixturePort"
$bookUrl = "$fixtureBase/book"
$source = @{
    bookSourceUrl = $fixtureBase
    bookSourceName = 'Deterministic differential fixture'
    searchUrl = "$fixtureBase/search?key={{key}}"
    ruleSearch = @{ bookList = '.book'; name = '.name@text'; author = '.author@text'; bookUrl = 'a@href' }
    ruleBookInfo = @{ name = 'h1@text'; author = '.author@text'; tocUrl = '.toc@href' }
    ruleToc = @{ chapterList = '.chapter'; chapterName = 'a@text'; chapterUrl = 'a@href' }
    ruleContent = @{ content = '.content@html' }
}
$user = 'webprobe' + [guid]::NewGuid().ToString('N').Substring(0, 10)
$password = 'Probe-' + [guid]::NewGuid().ToString('N')
$fixture = Start-Process -FilePath $python -ArgumentList @('-B', ('"' + (Join-Path $PSScriptRoot 'mock-book-source.py') + '"'), '--port', "$fixturePort") `
    -PassThru -WindowStyle Hidden -RedirectStandardOutput (Join-Path $runRoot 'fixture.stdout.log') `
    -RedirectStandardError (Join-Path $runRoot 'fixture.stderr.log')
try {
    $deadline = [DateTime]::UtcNow.AddSeconds(10)
    while ([DateTime]::UtcNow -lt $deadline) {
        if ($fixture.HasExited) { throw 'Book-source fixture exited during startup' }
        try { if ((Invoke-WebRequest -Uri "$fixtureBase/health" -TimeoutSec 2).StatusCode -eq 200) { break } }
        catch { }
        Start-Sleep -Milliseconds 200
    }
    if ([DateTime]::UtcNow -ge $deadline) { throw 'Book-source fixture did not start' }
    $original = Run-Reader $OriginalJar (Join-Path $runRoot 'original') $originalPort $source $user $password $bookUrl
    $restored = Run-Reader $RestoredJar (Join-Path $runRoot 'restored') $restoredPort $source $user $password $bookUrl
    $comparisons = for ($index = 0; $index -lt $original.Count; $index++) {
        $left = $original[$index]; $right = $restored[$index]
        $equal = (ConvertTo-Json $left -Compress -Depth 30) -ceq (ConvertTo-Json $right -Compress -Depth 30)
        $accepted = $false
        if (-not $equal -and $left.probe -eq 'shelf') {
            $accepted = $left.status -eq 200 -and $left.isSuccess -and
                $right.status -eq 200 -and $right.isSuccess -and
                $left.contentType -ceq $right.contentType -and
                $left.errorMsg -ceq $right.errorMsg -and
                (ConvertTo-Json $left.fieldShape -Compress -Depth 20) -ceq
                    (ConvertTo-Json $right.fieldShape -Compress -Depth 20) -and
                $left.data.Count -eq 1 -and $right.data.Count -eq 1 -and
                $left.data[0].name -cne '差分测试书' -and
                $left.data[0].author -cne '测试作者' -and
                $right.data[0].name -ceq '差分测试书' -and
                $right.data[0].author -ceq '测试作者' -and
                $left.data[0].bookUrl -ceq $right.data[0].bookUrl -and
                $left.data[0].tocUrl -ceq $right.data[0].tocUrl -and
                $left.data[0].origin -ceq $right.data[0].origin
        }
        if (-not $equal -and $left.probe -in @('content-0', 'content-1')) {
            $accepted = $left.status -eq 200 -and $right.status -eq 200 -and
                $left.isSuccess -and $right.isSuccess -and
                $left.contentType -ceq $right.contentType -and
                $left.errorMsg -ceq $right.errorMsg -and
                $left.data -ceq ($right.data + "`n")
        }
        [pscustomobject]@{
            probe = $left.probe
            semanticEqual = $equal
            acceptedDivergence = $accepted
            original = $left; restored = $right
        }
    }
    [ordered]@{
        originalJarSha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $OriginalJar).Hash
        restoredJarSha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $RestoredJar).Hash
        fixtureBase = $fixtureBase
        comparisons = @($comparisons)
    } | ConvertTo-Json -Depth 35 | Set-Content -LiteralPath $OutputPath -Encoding utf8
    $comparisons | Select-Object probe, semanticEqual, acceptedDivergence | Format-Table -AutoSize
    Write-Host "Report: $OutputPath"
    Write-Host "Isolated run data: $runRoot"
    if (@($comparisons | Where-Object { -not $_.semanticEqual -and -not $_.acceptedDivergence }).Count -ne 0) {
        throw 'Unreviewed remote book-source reading difference from original JAR'
    }
}
finally {
    $password = $null
    if (-not $fixture.HasExited) { Stop-Process -Id $fixture.Id -Force }
    $fixture.Dispose()
}
