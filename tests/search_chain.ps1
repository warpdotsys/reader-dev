# 搜索链路端到端验证：mock 书源服务器（独立进程）+ reader + 完整流程
$ErrorActionPreference = 'Continue'
$readerPort = 18093

# 启动 mock（独立进程）
Remove-Item "C:\Users\chong\AppData\Local\Temp\opencode\mock_requests.log" -Force -ErrorAction SilentlyContinue
$mock = Start-Process -FilePath 'powershell.exe' -ArgumentList '-File','C:\Users\chong\reader-dev-legacy\tests\mock_server.ps1' -PassThru -WindowStyle Hidden
Start-Sleep 2

# 启动 reader
$proc = Start-Process -FilePath 'D:\rust\target\debug\reader.exe' -ArgumentList "--port=$readerPort", '--workdir=C:\Users\chong\reader-dev-legacy' -WorkingDirectory 'C:\Users\chong\reader-dev-legacy' -PassThru -RedirectStandardOutput 'chain_out.log' -RedirectStandardError 'chain_err.log'

$results = @()
function Report($name, $ok, $detail) {
    $script:results += [pscustomobject]@{ Step = $name; OK = $ok; Detail = $detail }
    Write-Output ("[{0}] {1} :: {2}" -f ($(if($ok){'PASS'}else{'FAIL'})), $name, $detail)
}

try {
    Start-Sleep -Seconds 3
    $base = "http://localhost:$readerPort"
    $ready = $false
    for ($i = 0; $i -lt 20; $i++) {
        try { $null = Invoke-WebRequest -Uri "$base/" -UseBasicParsing -TimeoutSec 3; $ready = $true; break } catch { Start-Sleep -Milliseconds 500 }
    }
    if (-not $ready) { Report 'server-ready' $false 'reader not up'; exit 1 }
    Report 'server-ready' $true 'reader up'

    # 保存书源（UTF-8）
    $src = @{
        bookSourceUrl = 'http://localhost:18999'
        bookSourceName = '本地测试'
        bookSourceType = 0
        searchUrl = 'http://localhost:18999/search?key={{key}}&page={{page}}'
        ruleSearch = @{ bookList='$.books'; name='$.name'; author='$.author'; bookUrl='$.bookUrl'; intro='$.intro' }
        ruleBookInfo = @{ name='$.name'; author='$.author'; intro='$.intro'; tocUrl='$.tocUrl' }
        ruleToc = @{ chapterList='$.chapters'; chapterName='$.title'; chapterUrl='$.url' }
        ruleContent = @{ content='div.content' }
    } | ConvertTo-Json -Depth 8
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($src)
    $r = Invoke-WebRequest -Uri "$base/reader3/saveBookSource" -Method POST -Body $bytes -ContentType 'application/json' -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    Report 'saveBookSource' ($j.isSuccess -eq $true) $r.Content

    # 搜索
    $r = Invoke-WebRequest -Uri "$base/reader3/searchBook?key=%E6%B5%8B%E8%AF%95&bookSourceUrl=http%3A%2F%2Flocalhost%3A18999" -UseBasicParsing -TimeoutSec 30
    $j = $r.Content | ConvertFrom-Json
    $bookCount = 0
    if ($j.isSuccess -eq $true -and $null -ne $j.data) { $bookCount = @($j.data).Count }
    Report 'searchBook' ($bookCount -ge 2) ("found $bookCount books")

    # 详情
    if ($bookCount -ge 1) {
        $book = @($j.data)[0]
        $bu = [uri]::EscapeDataString($book.bookUrl)
        $r = Invoke-WebRequest -Uri "$base/reader3/getBookInfo?url=$bu&bookSourceUrl=http%3A%2F%2Flocalhost%3A18999" -UseBasicParsing -TimeoutSec 30
        $j2 = $r.Content | ConvertFrom-Json
        $nameOk = $null -ne $j2.data -and $j2.data.name -eq '测试之书'
        Report 'getBookInfo' ($j2.isSuccess -eq $true -and $nameOk) ("name=$($j2.data.name)")

        # 目录
        $r = Invoke-WebRequest -Uri "$base/reader3/getChapterList?url=$bu&bookSourceUrl=http%3A%2F%2Flocalhost%3A18999" -UseBasicParsing -TimeoutSec 30
        $j3 = $r.Content | ConvertFrom-Json
        $chapterCount = 0
        if ($null -ne $j3.data) { $chapterCount = @($j3.data).Count }
        Report 'getChapterList' ($chapterCount -ge 2) ("found $chapterCount chapters")

        # 正文
        if ($chapterCount -ge 1) {
            $cu = [uri]::EscapeDataString(@($j3.data)[0].url)
            $r = Invoke-WebRequest -Uri "$base/reader3/getBookContent?url=$bu&chapterUrl=$cu&bookSourceUrl=http%3A%2F%2Flocalhost%3A18999&index=-1" -UseBasicParsing -TimeoutSec 30
            $j4 = $r.Content | ConvertFrom-Json
            $hasContent = $j4.isSuccess -eq $true -and $null -ne $j4.data -and "$($j4.data)".Contains('正文内容段落')
            Report 'getBookContent' $hasContent ("content=$($j4.data)")
        } else {
            Report 'getBookContent' $false 'no chapters'
        }
    }
} catch {
    Report 'exception' $false $_.Exception.Message
} finally {
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    Stop-Process -Id $mock.Id -Force -ErrorAction SilentlyContinue
    Write-Output "cleanup done"
}

Write-Output ""
Write-Output "=== 汇总 ==="
$fails = @($results | Where-Object { -not $_.OK })
Write-Output "总步骤: $($results.Count)  失败: $($fails.Count)"
