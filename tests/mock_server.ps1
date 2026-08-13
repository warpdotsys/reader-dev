# Mock 书源服务器（独立进程）
$port = 18999
$listener = New-Object System.Net.HttpListener
$listener.Prefixes.Add("http://localhost:$port/")
try { $listener.Start() } catch { Write-Output "mock start failed: $($_.Exception.Message)"; exit 1 }
Add-Content -Path "C:\Users\chong\AppData\Local\Temp\opencode\mock_run.log" -Value "mock started"
while ($true) {
    try { $ctx = $listener.GetContext() } catch { break }
    $path = $ctx.Request.Url.AbsolutePath
    $query = $ctx.Request.Url.Query
    Add-Content -Path "C:\Users\chong\AppData\Local\Temp\opencode\mock_requests.log" -Value "$path$query"
    $body = ""
    try {
        if ($path -eq '/search') {
            $body = '{"books":[{"name":"测试之书","author":"作者甲","bookUrl":"http://localhost:18999/book/1","intro":"一本用于测试的书","coverUrl":"http://localhost:18999/cover.jpg"},{"name":"测试第二本","author":"作者乙","bookUrl":"http://localhost:18999/book/2","intro":"第二本测试书"}]}'
        } elseif ($path -eq '/remote-source.json') {
            $body = '[{"bookSourceUrl":"http://localhost:18999","bookSourceName":"远程导入源","bookSourceType":0,"searchUrl":"http://localhost:18999/search?key={{key}}","ruleSearch":{"bookList":"$.books","name":"$.name","author":"$.author","bookUrl":"$.bookUrl"}}]'
        } elseif ($path -eq '/explore') {
            $body = '{"books":[{"name":"发现之书","author":"作者丙","bookUrl":"http://localhost:18999/book/3","intro":"发现页书籍"}]}'
        } elseif ($path -eq '/book/1') {
            $body = '{"name":"测试之书","author":"作者甲","intro":"一本用于测试的书","tocUrl":"http://localhost:18999/toc/1"}'
        } elseif ($path -eq '/book/2') {
            $body = '{"name":"测试第二本","author":"作者乙","intro":"第二本测试书","tocUrl":"http://localhost:18999/toc/2"}'
        } elseif ($path -eq '/rss.xml') {
            $body = '<?xml version="1.0"?><rss version="2.0"><channel><title>本地RSS</title><item><title>文章一</title><link>http://localhost:18999/article/1</link><description>第一篇文章内容</description></item><item><title>文章二</title><link>http://localhost:18999/article/2</link><description>第二篇文章内容</description></item></channel></rss>'
        } elseif ($path -eq '/cover.jpg') {
            $body = 'COVERBINARYDATA123'
        } elseif ($path -eq '/toc/1') {
            $body = '{"chapters":[{"title":"第一章 开始","url":"http://localhost:18999/content/1"},{"title":"第二章 继续","url":"http://localhost:18999/content/2"}]}'
        } elseif ($path -eq '/content/1' -or $path -eq '/content/2') {
            $body = '<html><body><div class="content">这是正文内容段落，来自 mock 书源。</div></body></html>'
        } else {
            $ctx.Response.StatusCode = 404
        }
    } catch { $ctx.Response.StatusCode = 500 }
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($body)
    $ctx.Response.ContentType = 'application/json; charset=utf-8'
    $ctx.Response.ContentLength64 = $bytes.Length
    $ctx.Response.OutputStream.Write($bytes, 0, $bytes.Length)
    $ctx.Response.Close()
}
