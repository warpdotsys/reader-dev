# 前端交互流程模拟（登录 → 保存书源 → 搜索 → 详情 → 目录 → 正文）
# 与浏览器实际交互 API 序列一致
$ErrorActionPreference = 'Continue'
$readerPort = 18094

# 启动 mock 书源 + reader
$mock = Start-Process -FilePath 'powershell.exe' -ArgumentList '-File','C:\Users\chong\reader-dev-legacy\tests\mock_server.ps1' -PassThru -WindowStyle Hidden
Start-Sleep 2
$proc = Start-Process -FilePath 'D:\rust\target\debug\reader.exe' -ArgumentList "--port=$readerPort", '--workdir=C:\Users\chong\reader-dev-legacy' -WorkingDirectory 'C:\Users\chong\reader-dev-legacy' -PassThru -RedirectStandardOutput 'front_out.log' -RedirectStandardError 'front_err.log'

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
    if (-not $ready) { Report 'server' $false 'not up'; exit 1 }
    Report '页面加载' $true 'index.html 200'

    # 1. 登录（前端自动登录 admin/admin123；首次为注册语义，需干净用户文件）
    Remove-Item storage\data\users.json, storage\data\default\users.json, storage\data\.users.key -Force -ErrorAction SilentlyContinue
    $r = Invoke-WebRequest -Uri "$base/reader3/login" -Method POST -Body '{"username":"admin","password":"admin123"}' -ContentType 'application/json' -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    Report '登录' ($j.isSuccess -eq $true -and $null -ne $j.data.accessToken) "token=$($j.data.accessToken.Substring(0,8))..."

    # 2. 进入书源管理 → 保存书源（UTF-8）
    $src = @{
        bookSourceUrl = 'http://localhost:18999'
        bookSourceName = '本地测试'
        bookSourceType = 0
        searchUrl = 'http://localhost:18999/search?key={{key}}&page={{page}}'
        exploreUrl = 'http://localhost:18999/explore'
        ruleSearch = @{ bookList='$.books'; name='$.name'; author='$.author'; bookUrl='$.bookUrl'; intro='$.intro' }
        ruleExplore = @{ bookList='$.books'; name='$.name'; author='$.author'; bookUrl='$.bookUrl'; intro='$.intro' }
        ruleBookInfo = @{ name='$.name'; author='$.author'; intro='$.intro'; tocUrl='$.tocUrl' }
        ruleToc = @{ chapterList='$.chapters'; chapterName='$.title'; chapterUrl='$.url' }
        ruleContent = @{ content='div.content' }
    } | ConvertTo-Json -Depth 8
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($src)
    $r = Invoke-WebRequest -Uri "$base/reader3/saveBookSource" -Method POST -Body $bytes -ContentType 'application/json' -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    Report '保存书源' ($j.isSuccess -eq $true) ''

    # 3. 书源列表（前端书源管理页）
    $r = Invoke-WebRequest -Uri "$base/reader3/getBookSources" -Method POST -Body '{}' -ContentType 'application/json' -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    $n = if ($null -ne $j.data) { @($j.data).Count } else { 0 }
    Report '书源列表' ($n -ge 1) "书源数=$n"

    # 3.5 简单书源列表（书源管理页，仅 3 字段 + 有 exploreUrl 的）
    $r = Invoke-WebRequest -Uri "$base/reader3/getBookSources?simple=1" -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    $simple = @()
    if ($null -ne $j.data) { $simple = @($j.data | ForEach-Object { $_ }) }
    $keys = if ($simple.Count -ge 1) { @($simple[0].PSObject.Properties.Name | ForEach-Object { $_ }) } else { @() }
    $allowed = @('bookSourceGroup','bookSourceName','bookSourceUrl','exploreUrl')
    $simpleOk = ($simple.Count -ge 1) -and (($keys | Where-Object { $_ -notin $allowed }).Count -eq 0) -and ($simple[0].exploreUrl -eq $true)
    Report '书源简单列表' $simpleOk "条数=$($simple.Count) 字段=$($keys -join ',') exploreUrl=$($simple[0].exploreUrl)"

    # 3.6 发现页（书源 exploreUrl）
    $eu = [uri]::EscapeDataString('http://localhost:18999/explore')
    $r = Invoke-WebRequest -Uri "$base/reader3/exploreBook?ruleFindUrl=$eu&bookSourceUrl=http%3A%2F%2Flocalhost%3A18999" -UseBasicParsing -TimeoutSec 30
    $j = $r.Content | ConvertFrom-Json
    $explore = @()
    if ($null -ne $j.data) { $explore = @($j.data | ForEach-Object { $_ }) }
    Report '发现页' ($j.isSuccess -eq $true -and $explore.Count -ge 1 -and $explore[0].name -eq '发现之书') "书名=$($explore[0].name)"

    # 4. 书架 → 搜索（前端搜索页输入关键词）
    $r = Invoke-WebRequest -Uri "$base/reader3/getBookshelf" -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    Report '书架加载' ($j.isSuccess -eq $true) ''

    $kw = [uri]::EscapeDataString('测试')
    $r = Invoke-WebRequest -Uri "$base/reader3/searchBook?key=$kw&bookSourceUrl=http%3A%2F%2Flocalhost%3A18999" -UseBasicParsing -TimeoutSec 30
    $j = $r.Content | ConvertFrom-Json
    $books = if ($null -ne $j.data) { @($j.data) } else { @() }
    Report '搜索书籍' ($books.Count -ge 2) "结果=$($books.Count) 本"

    # 4.5 多书源并发搜索（前端首页搜索默认模式）
    $r = Invoke-WebRequest -Uri "$base/reader3/searchBookMulti?key=$kw&lastIndex=-1&searchSize=20" -UseBasicParsing -TimeoutSec 60
    $j = $r.Content | ConvertFrom-Json
    $multi = @()
    if ($null -ne $j.data -and $null -ne $j.data.list) { $multi = @($j.data.list | ForEach-Object { $_ }) }
    $multiOk = ($j.isSuccess -eq $true) -and ($multi.Count -ge 1) -and ($multi[0].name -eq '测试之书')
    Report '多书源搜索' $multiOk "结果=$($multi.Count) 本 raw=$($r.Content.Substring(0, [Math]::Min(300, $r.Content.Length)))"

    # 5. 点击书籍 → 详情
    $book = $books[0]
    $bu = [uri]::EscapeDataString($book.bookUrl)
    $r = Invoke-WebRequest -Uri "$base/reader3/getBookInfo?url=$bu&bookSourceUrl=http%3A%2F%2Flocalhost%3A18999" -UseBasicParsing -TimeoutSec 30
    $j = $r.Content | ConvertFrom-Json
    Report '书籍详情' ($j.isSuccess -eq $true -and $j.data.name -eq '测试之书') "书名=$($j.data.name)"

    # 6. 进入阅读 → 目录
    $r = Invoke-WebRequest -Uri "$base/reader3/getChapterList?url=$bu&bookSourceUrl=http%3A%2F%2Flocalhost%3A18999" -UseBasicParsing -TimeoutSec 30
    $j = $r.Content | ConvertFrom-Json
    $chapters = if ($null -ne $j.data) { @($j.data) } else { @() }
    Report '获取目录' ($chapters.Count -ge 2) "章节=$($chapters.Count)"

    # 7. 阅读正文
    $cu = [uri]::EscapeDataString($chapters[0].url)
    $r = Invoke-WebRequest -Uri "$base/reader3/getBookContent?url=$bu&chapterUrl=$cu&bookSourceUrl=http%3A%2F%2Flocalhost%3A18999&index=-1" -UseBasicParsing -TimeoutSec 30
    $j = $r.Content | ConvertFrom-Json
    Report '阅读正文' ($j.isSuccess -eq $true -and "$($j.data)".Contains('正文内容段落')) "正文=$($j.data)"

    # 8. 加入书架（前端书架页收藏）
    $bookShelfItem = $book | Select-Object @{n='name';e={$_.name}}, @{n='author';e={$_.author}}, @{n='bookUrl';e={$_.bookUrl}}, @{n='origin';e={$_.origin}}, @{n='intro';e={$_.intro}}
    $r = Invoke-WebRequest -Uri "$base/reader3/saveBook" -Method POST -Body ([System.Text.Encoding]::UTF8.GetBytes(($bookShelfItem | ConvertTo-Json))) -ContentType 'application/json' -UseBasicParsing -TimeoutSec 20
    $j = $r.Content | ConvertFrom-Json
    Report '加入书架' ($j.isSuccess -eq $true) $r.Content

    # 9. 书架刷新（应含新书）
    $r = Invoke-WebRequest -Uri "$base/reader3/getBookshelf" -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    $shelf = @()
    if ($null -ne $j.data) { $shelf = @($j.data | ForEach-Object { $_ }) }
    Report '书架含新书' ($shelf.Count -ge 1) "书架=$($shelf.Count) 本"

    # 10. 刷新书架（书架下拉刷新，触发网络拉取并更新书籍信息）
    $r = Invoke-WebRequest -Uri "$base/reader3/getBookshelf?refresh=1" -UseBasicParsing -TimeoutSec 60
    $j = $r.Content | ConvertFrom-Json
    $shelf = @()
    if ($null -ne $j.data) { $shelf = @($j.data | ForEach-Object { $_ }) }
    $refreshed = ($shelf.Count -ge 1) -and ($null -ne $shelf[0].latestChapterTitle) -and ($shelf[0].totalChapterNum -ge 2)
    Report '刷新书架' $refreshed "最新章节=$($shelf[0].latestChapterTitle) 章节=$($shelf[0].totalChapterNum)"

    # 10. 保存阅读进度（读到第2章）
    $progress = @{ url = $book.bookUrl; index = 1 } | ConvertTo-Json
    $r = Invoke-WebRequest -Uri "$base/reader3/saveBookProgress" -Method POST -Body ([System.Text.Encoding]::UTF8.GetBytes($progress)) -ContentType 'application/json' -UseBasicParsing -TimeoutSec 20
    $j = $r.Content | ConvertFrom-Json
    $savedRes = $r.Content
    $r = Invoke-WebRequest -Uri "$base/reader3/getBookshelf" -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    $shelf = @()
    if ($null -ne $j.data) { $shelf = @($j.data | ForEach-Object { $_ }) }
    $saved = $false
    if ($shelf.Count -ge 1) { $saved = ($shelf[0].durChapterIndex -eq 1) -and ($shelf[0].durChapterTitle -eq $chapters[1].title) }
    Report '保存阅读进度' $saved "进度=$($shelf[0].durChapterIndex)/$($shelf[0].durChapterTitle) save=$savedRes"

    # 11. 阅读配置（阅读页设置 PDF 图片宽度）
    $cfg = @{ bookUrl = $book.bookUrl; pdfImageWidth = 0.6 } | ConvertTo-Json
    $r = Invoke-WebRequest -Uri "$base/reader3/book/saveBookConfig" -Method POST -Body ([System.Text.Encoding]::UTF8.GetBytes($cfg)) -ContentType 'application/json' -UseBasicParsing -TimeoutSec 20
    $j = $r.Content | ConvertFrom-Json
    Report '阅读配置' ($j.isSuccess -eq $true) "isSuccess=$($j.isSuccess)"

    # 12. 单书查询（书架点击进入）
    $su = [uri]::EscapeDataString($book.bookUrl)
    $r = Invoke-WebRequest -Uri "$base/reader3/getShelfBook?url=$su" -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    Report '单书查询' ($j.isSuccess -eq $true -and $j.data.name -eq '测试之书') "书名=$($j.data.name)"

    # 11. 移动到分组（前端书架分组菜单）
    $group = @{ bookUrl = $book.bookUrl; groupId = 1 } | ConvertTo-Json
    $r = Invoke-WebRequest -Uri "$base/reader3/saveBookGroupId" -Method POST -Body ([System.Text.Encoding]::UTF8.GetBytes($group)) -ContentType 'application/json' -UseBasicParsing -TimeoutSec 20
    $j = $r.Content | ConvertFrom-Json
    $r = Invoke-WebRequest -Uri "$base/reader3/getBookshelf" -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    $shelf = @()
    if ($null -ne $j.data) { $shelf = @($j.data | ForEach-Object { $_ }) }
    $grouped = ($shelf.Count -ge 1) -and ($shelf[0].group -eq 1)
    Report '移动到分组' ($j.isSuccess -eq $true -and $grouped) "group=$($shelf[0].group)"

    # 12. 保存书签（阅读页划线/书签）
    $bm = @{ bookName = '测试之书'; bookAuthor = '作者甲'; chapterIndex = 0; chapterPos = 10; chapterName = '第一章'; bookText = '这是一段正文'; content = '书签内容'; time = 2024010100001 } | ConvertTo-Json
    $r = Invoke-WebRequest -Uri "$base/reader3/saveBookmark" -Method POST -Body ([System.Text.Encoding]::UTF8.GetBytes($bm)) -ContentType 'application/json' -UseBasicParsing -TimeoutSec 20
    $j = $r.Content | ConvertFrom-Json
    $r = Invoke-WebRequest -Uri "$base/reader3/getBookmarks" -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    $bms = @()
    if ($null -ne $j.data) { $bms = @($j.data | ForEach-Object { $_ }) }
    $bmOk = ($bms.Count -ge 1) -and ($bms[0].bookName -eq '测试之书')
    Report '保存书签' ($j.isSuccess -eq $true -and $bmOk) "书签=$($bms.Count) raw=$($r.Content)"

    # 13. 删除书签（checker 按 time 匹配）
    $r = Invoke-WebRequest -Uri "$base/reader3/deleteBookmark" -Method POST -Body ([System.Text.Encoding]::UTF8.GetBytes($bm)) -ContentType 'application/json' -UseBasicParsing -TimeoutSec 20
    $j = $r.Content | ConvertFrom-Json
    $r = Invoke-WebRequest -Uri "$base/reader3/getBookmarks" -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    $bms = @()
    if ($null -ne $j.data) { $bms = @($j.data | ForEach-Object { $_ }) }
    Report '删除书签' ($j.isSuccess -eq $true -and $bms.Count -eq 0) "剩余=$($bms.Count)"

    # 14. 替换净化规则（阅读页净化管理）
    $rr1 = @{ name='去广告'; pattern='广告'; replacement=''; scopeContent=$true; isEnabled=$true; isRegex=$false; timeoutMillisecond=3000; order=0 } | ConvertTo-Json
    $r = Invoke-WebRequest -Uri "$base/reader3/saveReplaceRule" -Method POST -Body ([System.Text.Encoding]::UTF8.GetBytes($rr1)) -ContentType 'application/json' -UseBasicParsing -TimeoutSec 20
    $j = $r.Content | ConvertFrom-Json
    $rr2 = @{ name='净化空格'; pattern='\s+'; replacement=''; scopeContent=$true; isEnabled=$true; isRegex=$true; timeoutMillisecond=3000; order=1 } | ConvertTo-Json
    $r = Invoke-WebRequest -Uri "$base/reader3/saveReplaceRule" -Method POST -Body ([System.Text.Encoding]::UTF8.GetBytes($rr2)) -ContentType 'application/json' -UseBasicParsing -TimeoutSec 20
    $j = $r.Content | ConvertFrom-Json
    $r = Invoke-WebRequest -Uri "$base/reader3/getReplaceRules" -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    $rrs = @()
    if ($null -ne $j.data) { $rrs = @($j.data | ForEach-Object { $_ }) }
    Report '保存替换规则' ($j.isSuccess -eq $true -and $rrs.Count -ge 2) "规则=$($rrs.Count)"

    # 15. 删除替换规则（checker 按 name 匹配）
    $r = Invoke-WebRequest -Uri "$base/reader3/deleteReplaceRule" -Method POST -Body ([System.Text.Encoding]::UTF8.GetBytes($rr1)) -ContentType 'application/json' -UseBasicParsing -TimeoutSec 20
    $j = $r.Content | ConvertFrom-Json
    $r = Invoke-WebRequest -Uri "$base/reader3/getReplaceRules" -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    $rrs = @()
    if ($null -ne $j.data) { $rrs = @($j.data | ForEach-Object { $_ }) }
    $rrLeft = ($rrs.Count -eq 1) -and ($rrs[0].name -eq '净化空格')
    Report '删除替换规则' ($j.isSuccess -eq $true -and $rrLeft) "剩余=$($rrs.Count)"

    # 16. RSS 源 CRUD
    $rs = @{ sourceUrl='http://localhost:18999/rss.xml'; sourceName='本地RSS'; enabled=$true } | ConvertTo-Json
    $r = Invoke-WebRequest -Uri "$base/reader3/saveRssSource" -Method POST -Body ([System.Text.Encoding]::UTF8.GetBytes($rs)) -ContentType 'application/json' -UseBasicParsing -TimeoutSec 20
    $j = $r.Content | ConvertFrom-Json
    $r = Invoke-WebRequest -Uri "$base/reader3/getRssSources" -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    $rss = @()
    if ($null -ne $j.data) { $rss = @($j.data | ForEach-Object { $_ }) }
    Report '保存RSS源' ($j.isSuccess -eq $true -and $rss.Count -ge 1 -and $rss[0].sourceName -eq '本地RSS') "RSS=$($rss.Count)"

    # 17. RSS 文章列表（getRssArticles → XML 解析链路）
    $r = Invoke-WebRequest -Uri "$base/reader3/getRssArticles?sourceUrl=http%3A%2F%2Flocalhost%3A18999%2Frss.xml" -UseBasicParsing -TimeoutSec 30
    $j = $r.Content | ConvertFrom-Json
    $arts = @()
    if ($null -ne $j.data -and $null -ne $j.data.first) { $arts = @($j.data.first | ForEach-Object { $_ }) }
    $artOk = ($j.isSuccess -eq $true) -and ($arts.Count -ge 2) -and ($arts[0].title -match '文章一')
    Report 'RSS文章列表' $artOk "文章=$($arts.Count)"

    # 17. 删除 RSS 源
    $r = Invoke-WebRequest -Uri "$base/reader3/deleteRssSource" -Method POST -Body ([System.Text.Encoding]::UTF8.GetBytes($rs)) -ContentType 'application/json' -UseBasicParsing -TimeoutSec 20
    $j = $r.Content | ConvertFrom-Json
    $r = Invoke-WebRequest -Uri "$base/reader3/getRssSources" -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    $rss = @()
    if ($null -ne $j.data) { $rss = @($j.data | ForEach-Object { $_ }) }
    Report '删除RSS源' ($j.isSuccess -eq $true -and $rss.Count -eq 0) "剩余=$($rss.Count)"

    # 18. 书源批量保存（saveBookSources）
    $srcList = "[$src]"
    $r = Invoke-WebRequest -Uri "$base/reader3/saveBookSources" -Method POST -Body ([System.Text.Encoding]::UTF8.GetBytes($srcList)) -ContentType 'application/json' -UseBasicParsing -TimeoutSec 20
    $j = $r.Content | ConvertFrom-Json
    $r = Invoke-WebRequest -Uri "$base/reader3/getBookSources" -Method POST -Body '{}' -ContentType 'application/json' -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    $n = if ($null -ne $j.data) { @($j.data).Count } else { 0 }
    Report '书源批量保存' ($j.isSuccess -eq $true -and $n -ge 1) "书源数=$n"

    # 19. WebDAV 备份（生成备份 zip）
    $r = Invoke-WebRequest -Uri "$base/reader3/backupToWebdav" -Method POST -Body '{}' -ContentType 'application/json' -UseBasicParsing -TimeoutSec 60
    $j = $r.Content | ConvertFrom-Json
    $backupDir = "$PSScriptRoot\..\storage\data\default\webdav"
    $backupZip = Get-ChildItem -Path $backupDir -Recurse -Filter '*.zip' -ErrorAction SilentlyContinue | Select-Object -First 1
    Report 'WebDAV备份' ($j.isSuccess -eq $true -and $null -ne $backupZip) "zip=$($backupZip.Name)"

    # 20. WebDAV 文件操作（MKCOL/PUT/PROPFIND/GET/DELETE）
    function Invoke-WebDav($uri, $method, $body) {
        try {
            $client = New-Object System.Net.Http.HttpClient
            $client.Timeout = [TimeSpan]::FromSeconds(15)
            $req = New-Object System.Net.Http.HttpRequestMessage([System.Net.Http.HttpMethod]::new($method), $uri)
            if ($null -ne $body) {
                $req.Content = New-Object System.Net.Http.StringContent($body, [System.Text.Encoding]::UTF8, 'text/plain')
            }
            $resp = $client.SendAsync($req).Result
            $content = $resp.Content.ReadAsStringAsync().Result
            $status = [int]$resp.StatusCode
            $client.Dispose()
            return @{ Status = $status; Content = $content }
        } catch {
            return @{ Status = 0; Content = $_.Exception.Message }
        }
    }
    $null = Invoke-WebDav "$base/reader3/webdav/legado" 'MKCOL' $null
    $null = Invoke-WebDav "$base/reader3/webdav/legado/test.txt" 'PUT' 'webdav test content'
    $wd = Invoke-WebDav "$base/reader3/webdav/legado" 'PROPFIND' $null
    $wdOk = $wd.Status -eq 207 -and $wd.Content -match 'test\.txt'
    Report 'WebDAV上传' $wdOk "status=$($wd.Status)"
    $wd = Invoke-WebDav "$base/reader3/webdav/legado/test.txt" 'GET' $null
    Report 'WebDAV下载' ($wd.Status -eq 200 -and $wd.Content -match 'webdav test content') "status=$($wd.Status)"
    $wd = Invoke-WebDav "$base/reader3/webdav/legado/test.txt" 'DELETE' $null
    Report 'WebDAV删除' ($wd.Status -eq 200 -or $wd.Status -eq 204) "status=$($wd.Status)"

    # 21. 远程书源导入（saveFromRemoteSource）
    $imp = @{ url = 'http://localhost:18999/remote-source.json' } | ConvertTo-Json
    $r = Invoke-WebRequest -Uri "$base/reader3/saveFromRemoteSource" -Method POST -Body ([System.Text.Encoding]::UTF8.GetBytes($imp)) -ContentType 'application/json' -UseBasicParsing -TimeoutSec 20
    $j = $r.Content | ConvertFrom-Json
    $impData = @()
    if ($null -ne $j.data) { $impData = @($j.data | ForEach-Object { $_ }) }
    $impOk = ($j.isSuccess -eq $true) -and ($impData.Count -ge 1) -and ($impData[0] -match '远程导入源')
    Report '远程书源导入' $impOk "raw=$($r.Content)"

    # 22. 封面下载（get_book_cover → launch 封面缓存链路）
    $cv = [uri]::EscapeDataString('http://localhost:18999/cover.jpg')
    $r = Invoke-WebRequest -Uri "$base/reader3/cover?path=$cv" -UseBasicParsing -TimeoutSec 20 -ErrorAction SilentlyContinue
    $coverOk = ($r.StatusCode -eq 200) -and ($r.RawContentLength -gt 0)
    Report '封面下载' $coverOk "status=$($r.StatusCode) len=$($r.RawContentLength)"

    # 15. 书源启禁用（保存 enabled=false 模拟禁用，读回验证）
    $srcObj = $src | ConvertFrom-Json
    $srcObj | Add-Member -NotePropertyName enabled -NotePropertyValue $false -Force
    $srcDisabled = $srcObj | ConvertTo-Json -Depth 8 -Compress
    $r = Invoke-WebRequest -Uri "$base/reader3/saveBookSource" -Method POST -Body ([System.Text.Encoding]::UTF8.GetBytes($srcDisabled)) -ContentType 'application/json' -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    $r = Invoke-WebRequest -Uri "$base/reader3/getBookSources" -Method POST -Body '{}' -ContentType 'application/json' -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    $n = @()
    if ($null -ne $j.data) { $n = @($j.data | ForEach-Object { $_ }) }
    $disabledOk = ($n.Count -ge 1) -and ($n[0].enabled -eq $false)
    Report '书源禁用' ($j.isSuccess -eq $true -and $disabledOk) "enabled=$($n[0].enabled)"

    # 16. 恢复启用（enabled=true 保存回写）
    $r = Invoke-WebRequest -Uri "$base/reader3/saveBookSource" -Method POST -Body ([System.Text.Encoding]::UTF8.GetBytes($src)) -ContentType 'application/json' -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    $r = Invoke-WebRequest -Uri "$base/reader3/getBookSources" -Method POST -Body '{}' -ContentType 'application/json' -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    $n = @()
    if ($null -ne $j.data) { $n = @($j.data | ForEach-Object { $_ }) }
    $enabledOk = ($n.Count -ge 1) -and ($n[0].enabled -eq $true)
    Report '书源恢复启用' ($j.isSuccess -eq $true -and $enabledOk) "enabled=$($n[0].enabled)"

    # 17. 删除书源（书源管理页长按删除）
    $r = Invoke-WebRequest -Uri "$base/reader3/deleteBookSource" -Method POST -Body ([System.Text.Encoding]::UTF8.GetBytes($src)) -ContentType 'application/json' -UseBasicParsing -TimeoutSec 20
    $j = $r.Content | ConvertFrom-Json
    $r = Invoke-WebRequest -Uri "$base/reader3/getBookSources" -Method POST -Body '{}' -ContentType 'application/json' -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    $n = if ($null -ne $j.data) { @($j.data).Count } else { 0 }
    Report '删除书源' ($j.isSuccess -eq $true -and $n -eq 0) "书源数=$n"

    # 11. 移除书架（前端书架长按删除）
    $r = Invoke-WebRequest -Uri "$base/reader3/deleteBook" -Method POST -Body ([System.Text.Encoding]::UTF8.GetBytes(($bookShelfItem | ConvertTo-Json))) -ContentType 'application/json' -UseBasicParsing -TimeoutSec 20
    $j = $r.Content | ConvertFrom-Json
    $r = Invoke-WebRequest -Uri "$base/reader3/getBookshelf" -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    $shelf = @()
    if ($null -ne $j.data) { $shelf = @($j.data | ForEach-Object { $_ }) }
    Report '移除书架' ($j.isSuccess -eq $true -and $shelf.Count -eq 0) "书架=$($shelf.Count) 本"

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
if ($fails.Count -gt 0) { $fails | ForEach-Object { "  FAIL: $($_.Step) :: $($_.Detail)" } }
