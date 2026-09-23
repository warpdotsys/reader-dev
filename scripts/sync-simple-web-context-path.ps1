$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$simpleWebDir = Join-Path $root "src\main\resources\simple-web"
$oldTag = '<script type="text/javascript" src="assets/js/common-eebd186870.js"></script>'
$newTag = $oldTag + '<script type="text/javascript" src="assets/js/context-path-20260923.js"></script>'

foreach ($name in @("index.html", "reader.html", "search.html", "rss.html")) {
    $path = Join-Path $simpleWebDir $name
    $html = [IO.File]::ReadAllText($path, [Text.Encoding]::UTF8)
    if ($html.Contains($newTag)) { continue }
    if (-not $html.Contains($oldTag)) { throw "Expected common bundle reference missing: $path" }
    $html = $html.Replace($oldTag, $newTag)
    [IO.File]::WriteAllText($path, $html, [Text.UTF8Encoding]::new($false))
}
