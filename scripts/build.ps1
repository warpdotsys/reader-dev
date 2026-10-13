param(
    [string]$JavaHome = ""
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot

. (Join-Path $PSScriptRoot 'reader-project-version.ps1')
$buildVersion = Get-ReaderBuildVersion -ProjectRoot $projectRoot

if ([string]::IsNullOrWhiteSpace($JavaHome)) {
    $JavaHome = Join-Path $projectRoot ".tools\jdk-11.0.8"
}

$javaExecutable = Join-Path $JavaHome "bin\java.exe"
if (-not (Test-Path -LiteralPath $javaExecutable)) {
    throw "JDK not found at $JavaHome. Pass -JavaHome with a JDK 11 path."
}

$env:JAVA_HOME = (Resolve-Path -LiteralPath $JavaHome).Path
$env:PATH = (Join-Path $env:JAVA_HOME "bin") + [IO.Path]::PathSeparator + $env:PATH

Push-Location $projectRoot
try {
    # build/ also contains authorized private test inputs and saved receipts.
    # Rerun task outputs without recursively deleting that shared directory.
    # CI proves exact-byte reproducibility with two clean builds in its own checkout.
    & .\gradlew.bat test bootJar --rerun-tasks --max-workers=2 --no-build-cache --no-daemon
    if ($LASTEXITCODE -ne 0) {
        throw "Gradle build failed with exit code $LASTEXITCODE"
    }

    if ((Get-ReaderBuildVersion -ProjectRoot $projectRoot) -ne $buildVersion) {
        throw 'Project version changed during the build; refusing a stale artifact.'
    }
    $artifact = Join-Path $projectRoot "build\libs\reader-$buildVersion.jar"
    if (-not (Test-Path -LiteralPath $artifact -PathType Leaf) -or (Get-Item -LiteralPath $artifact).Length -eq 0) {
        throw "Expected artifact was not produced: $artifact"
    }

    $artifactInfo = Get-Item -LiteralPath $artifact
    $artifactHash = Get-FileHash -Algorithm SHA256 -LiteralPath $artifact
    # One consistent object prevents PowerShell's mixed-object table formatter
    # from silently hiding the SHA-256 after printing file metadata.
    [PSCustomObject]@{
        FullName = $artifactInfo.FullName
        Length = $artifactInfo.Length
        LastWriteTime = $artifactInfo.LastWriteTime
        Algorithm = $artifactHash.Algorithm
        SHA256 = $artifactHash.Hash
    }
}
finally {
    Pop-Location
}
