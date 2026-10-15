# Read project metadata only. Never select the newest or an old JAR by glob.
function Get-ReaderBuildVersion {
    param([Parameter(Mandatory = $true)][string]$ProjectRoot)
    $source = Get-Content -LiteralPath (Join-Path $ProjectRoot 'build.gradle.kts') -Raw
    $matches = [regex]::Matches($source, '(?m)^version = "((?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*))"\r?$')
    if ($matches.Count -ne 1) {
        throw 'Expected exactly one stable project version in build.gradle.kts.'
    }
    return $matches[0].Groups[1].Value
}
