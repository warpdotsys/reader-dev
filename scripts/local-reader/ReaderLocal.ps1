param(
    [ValidateSet('Start', 'Stop', 'Status')]
    [string]$Action = 'Status',
    [switch]$GeneratedNetwork
)
$ErrorActionPreference = 'Stop'
$taskRoot = 'D:\Reader-Pro-Local-Full-610-20261015-a'
$serviceSource = Join-Path $taskRoot 'runtime-software\source\service.py'
if (-not (Test-Path -LiteralPath (Join-Path $taskRoot 'prepare-receipt.json') -PathType Leaf)) {
    throw '本机镜像尚未完成首次准备；不会自动格式化任何磁盘。'
}
$serviceLinux = (& wsl.exe -d Ubuntu -u root --exec wslpath -a $serviceSource).Trim()
if ($LASTEXITCODE -ne 0 -or $serviceLinux -cne '/mnt/d/Reader-Pro-Local-Full-610-20261015-a/runtime-software/source/service.py') {
    throw '无法确认本机启动脚本路径。'
}
if ($Action -eq 'Stop') {
    & wsl.exe -d Ubuntu -u root --exec python3 $serviceLinux stop --execute
    if ($LASTEXITCODE -ne 0) { throw '服务未正常停止；没有删除数据。' }
    return
}
if ($Action -eq 'Status') {
    & wsl.exe -d Ubuntu -u root --exec python3 $serviceLinux status
    if ($LASTEXITCODE -ne 0) { throw '无法读取本机状态。' }
    return
}
$current = & wsl.exe -d Ubuntu -u root --exec python3 $serviceLinux status
if ($LASTEXITCODE -ne 0) { throw '无法确认旧服务状态；未启动第二份。' }
$currentState = $current | ConvertFrom-Json
if ($currentState.unitActive -eq 'active') {
    $current
    return
}
$stamp = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfff')
$arguments = '-d Ubuntu -u root --exec python3 "' + $serviceLinux + '" start --execute'
if ($GeneratedNetwork) { $arguments += ' --generated-network' }
$controller = Start-Process -FilePath 'wsl.exe' -ArgumentList $arguments -WindowStyle Hidden -PassThru `
    -RedirectStandardOutput (Join-Path $taskRoot ('controller-' + $stamp + '.out.log')) `
    -RedirectStandardError (Join-Path $taskRoot ('controller-' + $stamp + '.err.log'))
$deadline = [DateTime]::UtcNow.AddSeconds(100)
while ([DateTime]::UtcNow -lt $deadline) {
    $controller.Refresh()
    if ($controller.HasExited) { throw '本机启动进程已退出；请检查本次 controller 日志。' }
    try {
        $response = Invoke-RestMethod -Uri 'http://127.0.0.1:18963/reader3/getSystemInfo' -TimeoutSec 3 -NoProxy
        if ($response.isSuccess -eq $true) {
            [PSCustomObject]@{ status = 'ready'; url = 'http://127.0.0.1:18963/'; controllerPid = $controller.Id; dataRetained = $true } | ConvertTo-Json
            return
        }
    } catch { }
    Start-Sleep -Milliseconds 500
}
throw '尚未确认 Windows 本机入口可用；服务未被自动终止，数据保持不变。'
