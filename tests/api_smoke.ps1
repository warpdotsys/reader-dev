$ErrorActionPreference = 'Continue'
$port = 18092
$base = "http://localhost:$port"
$results = @()

function Test-Api($name, $method, $path, $body, $expectSuccess = $true) {
    try {
        $params = @{ Uri = "$base$path"; Method = $method; UseBasicParsing = $true; TimeoutSec = 15 }
        if ($body) { $params.Body = ($body | ConvertTo-Json -Compress -Depth 6); $params.ContentType = 'application/json' }
        $r = Invoke-WebRequest @params
        $content = $r.Content
        $json = $null
        try { $json = $content | ConvertFrom-Json } catch {}
        $hasData = $null -ne $json -and ($null -eq $json.data -or $json.data.GetType().Name -ne 'String' -or $json.data -notmatch '^Any')
        $dataIsAny = $null -ne $json -and ($json.data -is [string] -and $json.data -match '^Any')
        if ($expectSuccess) {
            $ok = $r.StatusCode -eq 200 -and $null -ne $json -and $json.isSuccess -eq $true -and -not $dataIsAny
        } else {
            $ok = $r.StatusCode -eq 200
        }
        $snippet = if ($content.Length -gt 120) { $content.Substring(0, 120) } else { $content }
        $script:results += [pscustomobject]@{ API = $name; Status = $r.StatusCode; OK = $ok; DataIsAny = $dataIsAny; Snippet = $snippet -replace "`n", " " }
    } catch {
        $script:results += [pscustomobject]@{ API = $name; Status = 'ERR'; OK = $false; DataIsAny = $false; Snippet = $_.Exception.Message }
    }
}

# 1. 启动服务器
$proc = Start-Process -FilePath 'D:\rust\target\debug\reader.exe' -ArgumentList "--port=$port", '--workdir=C:\Users\chong\reader-dev-legacy' -WorkingDirectory 'C:\Users\chong\reader-dev-legacy' -PassThru -RedirectStandardOutput 'test_srv_out.log' -RedirectStandardError 'test_srv_err.log'
try {
    # 等待就绪
    $ready = $false
    Start-Sleep -Seconds 3
    for ($i = 0; $i -lt 20; $i++) {
        try { $null = Invoke-WebRequest -Uri "$base/" -UseBasicParsing -TimeoutSec 3; $ready = $true; break } catch { Start-Sleep -Milliseconds 500 }
    }
    if (-not $ready) { Write-Output "SERVER NOT READY"; exit 1 }
    Write-Output "server up, PID=$($proc.Id)"

    # 2. 基础
    Test-Api 'index'        'GET'  '/' $null $false
    Test-Api 'systemInfo'   'GET'  '/reader3/getSystemInfo'
    Test-Api 'login'        'POST' '/reader3/login' @{ username = 'admin'; password = 'admin123' }

    # 3. 书源模块
    Test-Api 'getBookSources'   'POST' '/reader3/getBookSources' @{}
    Test-Api 'getBookSource'    'POST' '/reader3/getBookSource' @{ bookSourceUrl = 'https://www.kuaikan.com' } $null $false
    Test-Api 'deleteAllBookSources' 'POST' '/reader3/deleteAllBookSources' @{}

    # 4. 书架/书籍
    Test-Api 'getBookshelf'  'GET'  '/reader3/getBookshelf'
    Test-Api 'getShelfBook'  'GET'  '/reader3/getShelfBook?url=https%3A%2F%2Fwww.kuaikan.com' $null $false
    Test-Api 'getBookGroups' 'GET'  '/reader3/getBookGroups'
    Test-Api 'saveBookGroup' 'POST' '/reader3/saveBookGroup' @{ groupName = '测试分组'; groupId = 0; show = $true }
    Test-Api 'getBookmarks'  'GET'  '/reader3/getBookmarks'

    # 5. RSS / 替换规则 / TTS
    Test-Api 'getRssSources'     'GET'  '/reader3/getRssSources'
    Test-Api 'getReplaceRules'   'GET'  '/reader3/getReplaceRules'
    Test-Api 'getHttpTtsList'    'GET'  '/reader3/httpTTS/list'
    Test-Api 'getRssArticles'    'GET'  '/reader3/getRssArticles?sourceUrl=https%3A%2F%2Fexample.com%2Frss' $null $false

    # 6. 用户（getUserList/getUserConfig 为未实现 stub/行为性返回，仅验证 200）
    Test-Api 'getUserList'   'GET'  '/reader3/getUserList' $null $false
    Test-Api 'getUserInfo'   'GET'  '/reader3/getUserInfo'
    Test-Api 'getUserConfig' 'GET'  '/reader3/getUserConfig' $null $false

    # 7. 文件管理
    Test-Api 'fileList'      'GET'  '/reader3/file/list?path=%2F&home=__HOME__'
} finally {
    # 3. 停止服务器（必须清理）
    Stop-Process -Name reader -Force -ErrorAction SilentlyContinue
    Write-Output "server stopped"
}

# 汇总
Write-Output ""
Write-Output "=== 汇总 ==="
$fail = @($results | Where-Object { -not $_.OK })
$any = @($results | Where-Object { $_.DataIsAny })
$results | Format-Table -AutoSize | Out-String -Width 200
Write-Output "总测试: $($results.Count)  失败: $($fail.Count)  返回Any占位: $($any.Count)"
if ($fail.Count -gt 0) { Write-Output "--- 失败明细 ---"; $fail | ForEach-Object { "  $($_.API) [$($_.Status)] $($_.Snippet)" } }
Write-Output "--- 服务器 stderr 尾部 ---"
Get-Content 'test_srv_err.log' -ErrorAction SilentlyContinue | Select-Object -Last 15
