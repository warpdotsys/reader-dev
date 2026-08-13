# secure 模式用户系统验证：登录/会话/自动登录/users.json 持久化
$ErrorActionPreference = 'Continue'
$readerPort = 18111
Remove-Item storage\data\default\users*.json, storage\data\default\.users.key -Force -ErrorAction SilentlyContinue
$env:READER_APP_SECURE = 'true'
$env:READER_APP_SECUREKEY = 'adminpwd'
$proc = Start-Process -FilePath 'D:\rust\target\debug\reader.exe' -ArgumentList "--port=$readerPort", '--workdir=C:\Users\chong\reader-dev-legacy' -WorkingDirectory 'C:\Users\chong\reader-dev-legacy' -PassThru -RedirectStandardOutput 'D:\rust\sec_out.log' -RedirectStandardError 'D:\rust\sec_err.log'
Start-Sleep 4
$base = "http://localhost:$readerPort"
$ok = 0; $fail = 0
function RSec($name, $okv, $d) { $script:ok += [int]$okv; $script:fail += [int](-not $okv); Write-Output ("[{0}] {1} :: {2}" -f $(if($okv){'PASS'}else{'FAIL'}), $name, $d) }

try {
    # 未登录访问受保护接口 → NEED_LOGIN
    $r = Invoke-WebRequest -Uri "$base/reader3/getBookshelf" -UseBasicParsing -TimeoutSec 10 -ErrorAction SilentlyContinue
    $j = $r.Content | ConvertFrom-Json
    RSec '未登录拦截' ($j.isSuccess -eq $false -and $j.data -eq 'NEED_LOGIN') "data=$($j.data) err=$($j.errorMsg)"

    # 登录（自动注册 admin）
    $r = Invoke-WebRequest -Uri "$base/reader3/login" -Method POST -Body '{"username":"admin","password":"admin123"}' -ContentType 'application/json' -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    $token = $j.data.accessToken
    RSec '登录成功' ($j.isSuccess -eq $true -and $null -ne $token) "token=$($token.Substring(0, [Math]::Min(10, $token.Length)))..."

    # users.json 持久化（应含 password/salt/token，非空 map）
    $usersPath = "$PWD\storage\data\users.json"
    if (-not (Test-Path $usersPath)) { $usersPath = "$PWD\storage\data\default\users.json" }
    $usersRaw = [System.IO.File]::ReadAllText($usersPath)
    $users = $usersRaw | ConvertFrom-Json
    $admin = $users.admin
    $persisted = ($null -ne $admin) -and ($admin.password.Length -gt 0) -and ($admin.salt.Length -gt 0) -and ($admin.token.Length -gt 0)
    RSec 'users.json 持久化' $persisted "password=$($admin.password.Length)盐=$($admin.salt.Length)"

    # accessToken 自动登录（模拟前端带 token 访问）
    $h = @{ Authorization = "Bearer $token" }
    $r = Invoke-WebRequest -Uri "$base/reader3/getBookshelf?accessToken=$([uri]::EscapeDataString($token))" -UseBasicParsing -TimeoutSec 10 -ErrorAction SilentlyContinue
    $j = $r.Content | ConvertFrom-Json
    RSec '自动登录' ($j.isSuccess -eq $true) "isSuccess=$($j.isSuccess)"

    # 登录后再次登录（已有用户 + isLogin=true 走密码校验）
    $r = Invoke-WebRequest -Uri "$base/reader3/login" -Method POST -Body '{"username":"admin","password":"admin123","isLogin":true}' -ContentType 'application/json' -UseBasicParsing -TimeoutSec 10
    $j = $r.Content | ConvertFrom-Json
    RSec '二次登录' ($j.isSuccess -eq $true) "isSuccess=$($j.isSuccess) err=$($j.errorMsg)"

    # 错误密码
    $r = Invoke-WebRequest -Uri "$base/reader3/login" -Method POST -Body '{"username":"admin","password":"wrong"}' -ContentType 'application/json' -UseBasicParsing -TimeoutSec 10 -ErrorAction SilentlyContinue
    $j = $r.Content | ConvertFrom-Json
    RSec '错误密码拒绝' ($j.isSuccess -eq $false) "isSuccess=$($j.isSuccess)"

    # 管理密码校验（getUserList 需要 secureKey）
    $r = Invoke-WebRequest -Uri "$base/reader3/getUserList?accessToken=$([uri]::EscapeDataString($token))" -UseBasicParsing -TimeoutSec 10 -ErrorAction SilentlyContinue
    $j = $r.Content | ConvertFrom-Json
    RSec '用户列表' ($j.isSuccess -eq $true -and $null -ne $j.data) "isSuccess=$($j.isSuccess) users=$($j.data.Count)"
} catch {
    RSec 'exception' $false $_.Exception.Message
} finally {
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    Remove-Item Env:\READER_APP_SECURE -ErrorAction SilentlyContinue
    Remove-Item Env:\READER_APP_SECUREKEY -ErrorAction SilentlyContinue
}
Write-Output ""
Write-Output "=== 汇总 ==="
Write-Output "通过: $ok  失败: $fail"
