param(
    [string]$JavaHome = ""
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot

if ([string]::IsNullOrWhiteSpace($JavaHome)) {
    $JavaHome = Join-Path $projectRoot ".tools\jdk-11.0.8"
}

$javaExecutable = Join-Path $JavaHome "bin\java.exe"
if (-not (Test-Path -LiteralPath $javaExecutable)) {
    throw "JDK not found at $JavaHome. Pass -JavaHome with a JDK 11 path."
}

$env:JAVA_HOME = (Resolve-Path -LiteralPath $JavaHome).Path
$env:PATH = (Join-Path $env:JAVA_HOME "bin") + ";" + $env:PATH

Push-Location $projectRoot
try {
    & .\gradlew.bat clean test bootJar --no-daemon
    if ($LASTEXITCODE -ne 0) {
        throw "Gradle build failed with exit code $LASTEXITCODE"
    }

    $artifact = Join-Path $projectRoot "build\libs\reader-4.0.7.jar"
    if (-not (Test-Path -LiteralPath $artifact)) {
        throw "Expected artifact was not produced: $artifact"
    }

    Get-Item -LiteralPath $artifact | Select-Object FullName, Length, LastWriteTime
    Get-FileHash -Algorithm SHA256 -LiteralPath $artifact
}
finally {
    Pop-Location
}
