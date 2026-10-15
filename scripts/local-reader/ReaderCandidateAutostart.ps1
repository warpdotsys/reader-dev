<#
.SYNOPSIS
    Plan, inspect, or (only when explicitly enabled) manage the logon task for one
    Reader Pro candidate profile.

.DESCRIPTION
    This script deliberately has no default side effect.  It only accepts a
    profile whose candidate id and D: work directory are bound to the candidate
    task name.  It never creates a boot task, a SYSTEM task, or a highest-
    privilege task.  A user must separately call Install/Uninstall with both
    -Execute and -ConfirmCandidateId.  Plan and Inspect do not invoke any task
    registration command.
#>
[CmdletBinding()]
param(
    [ValidateSet('Plan', 'Inspect', 'Install', 'Uninstall')]
    [string]$Action = 'Plan',

    [Parameter(Mandatory = $true)]
    [string]$ProfilePath,

    [switch]$Execute,

    [string]$ConfirmCandidateId,

    # Test-only adapter: production calls are rejected unless the caller opted
    # into the isolated test mode.  It exists so tests never touch Task
    # Scheduler while still executing Plan/Install/Uninstall code paths.
    [string]$TestSchedulerAdapterPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Fail([string]$Message) { throw "ReaderCandidateAutostart: $Message" }

function Get-Sha256Hex([string]$Path) {
    $hash = [Security.Cryptography.SHA256]::Create()
    $stream = $null
    try {
        $stream = [IO.File]::OpenRead($Path)
        return ([BitConverter]::ToString($hash.ComputeHash($stream))).Replace('-', '').ToLowerInvariant()
    }
    finally {
        if ($null -ne $stream) { $stream.Dispose() }
        $hash.Dispose()
    }
}

function Get-AbsoluteWindowsPath([string]$Path, [string]$Label) {
    if (-not [IO.Path]::IsPathRooted($Path)) { Fail "$Label 必须是绝对 Windows 路径。" }
    $full = [IO.Path]::GetFullPath($Path)
    if ($full -notmatch '^[A-Za-z]:\\') { Fail "$Label 必须是带盘符的绝对 Windows 路径。" }
    return $full
}

function Test-ReparseFreePath([string]$Path) {
    $resolved = Get-AbsoluteWindowsPath $Path '路径'
    $current = [IO.Path]::GetPathRoot($resolved)
    $relative = $resolved.Substring($current.Length).TrimStart('\\')
    foreach ($piece in $relative.Split([char]'\', [StringSplitOptions]::RemoveEmptyEntries)) {
        $current = Join-Path $current $piece
        if (Test-Path -LiteralPath $current) {
            $item = Get-Item -LiteralPath $current -Force
            if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
                Fail "拒绝重解析点路径：$current"
            }
        }
    }
    return $resolved
}

function Get-CandidateBinding([string]$RawProfilePath) {
    $RawProfilePath = Get-AbsoluteWindowsPath $RawProfilePath 'ProfilePath'
    if (-not (Test-Path -LiteralPath $RawProfilePath -PathType Leaf)) { Fail '候选 profile 不存在。' }
    $profileFile = Test-ReparseFreePath $RawProfilePath
    $profileDirectory = Split-Path -Parent $profileFile
    $profileData = Get-Content -LiteralPath $profileFile -Raw -Encoding UTF8 | ConvertFrom-Json
    if ($profileData.schemaVersion -ne 1) { Fail '只接受 schemaVersion=1 的候选 profile。' }
    $candidateId = [string]$profileData.candidateId
    # Candidate profile grammar is shared with candidate-profile.py.  The
    # externally visible task identity is derived as reader-candidate-<id>.
    if ($candidateId -notmatch '^[a-z][a-z0-9-]{2,62}$') { Fail 'candidateId 格式不符合候选 profile。' }
    if ($null -eq $profileData.paths -or [string]::IsNullOrWhiteSpace([string]$profileData.paths.workDir) -or [string]::IsNullOrWhiteSpace([string]$profileData.paths.mountRoot)) { Fail 'profile 缺少 paths.workDir 或 paths.mountRoot。' }
    $expectedRoot = "D:\Reader-Pro-Candidate-$candidateId"
    $workDir = [string]$profileData.paths.workDir
    $expectedWslWorkDir = "/mnt/d/Reader-Pro-Candidate-$candidateId"
    $expectedMountRoot = "/mnt/reader-candidate-$candidateId"
    if ($workDir -cne $expectedWslWorkDir) { Fail "paths.workDir 必须精确为 $expectedWslWorkDir。" }
    if ([string]$profileData.paths.mountRoot -cne $expectedMountRoot) { Fail "paths.mountRoot 必须精确为 $expectedMountRoot。" }
    if ($profileDirectory -cne $expectedRoot) { Fail "profile 必须直接位于 $expectedRoot。" }
    if ($profileFile -cne (Join-Path $expectedRoot 'candidate-profile.json')) { Fail 'profile 文件名必须为 candidate-profile.json。' }
    $root = Test-ReparseFreePath $expectedRoot
    $launcher = Join-Path $root 'runtime-software\source\ReaderCandidate.ps1'
    if (-not (Test-Path -LiteralPath $launcher -PathType Leaf)) { Fail '候选启动器不存在；不会创建无效登录任务。' }
    $launcher = Test-ReparseFreePath $launcher
    if ($launcher -cne (Join-Path $root 'runtime-software\source\ReaderCandidate.ps1')) { Fail '候选启动器路径不匹配。' }
    if ($null -eq $profileData.artifacts -or [string]$profileData.artifacts.softwareManifestSha256 -notmatch '^[0-9a-f]{64}$') { Fail 'profile 缺少 artifacts.softwareManifestSha256。' }
    $launcherShaPath = "$launcher.sha256"
    if (-not (Test-Path -LiteralPath $launcherShaPath -PathType Leaf)) { Fail '候选启动器缺少固定 SHA-256 旁车文件；不会注册。' }
    $launcherShaPath = Test-ReparseFreePath $launcherShaPath
    $declaredLauncherSha = (Get-Content -LiteralPath $launcherShaPath -Raw -Encoding ASCII).Trim().ToLowerInvariant()
    if ($declaredLauncherSha -notmatch '^[0-9a-f]{64}$') { Fail '启动器 SHA-256 旁车文件格式错误。' }
    $actualLauncherSha = Get-Sha256Hex $launcher
    if ($actualLauncherSha -cne $declaredLauncherSha) { Fail '启动器 SHA-256 与旁车文件不一致。' }
    $powerShellPath = (Get-Command powershell.exe -CommandType Application -ErrorAction Stop).Source
    if (-not (Test-Path -LiteralPath $powerShellPath -PathType Leaf)) {
        Fail '无法确认绝对的 Windows PowerShell 可执行文件路径。'
    }
    $powerShellPath = Get-AbsoluteWindowsPath $powerShellPath 'Windows PowerShell 路径'
    $powerShellPath = Test-ReparseFreePath $powerShellPath
    [PSCustomObject]@{
        CandidateId = $candidateId
        InstanceName = "reader-candidate-$candidateId"
        Root = $root
        ProfilePath = $profileFile
        LauncherPath = $launcher
        LauncherSha256 = $actualLauncherSha
        TaskName = "ReaderPro-reader-candidate-$candidateId-Logon"
        # Keep the task at the root because Register-ScheduledTask does not
        # safely create arbitrary nested folders.  Its full name remains bound
        # to this exact candidate id.
        TaskPath = '\'
        TaskFullName = "\ReaderPro-reader-candidate-$candidateId-Logon"
        CurrentUserSid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
        PowerShellPath = $powerShellPath
    }
}

function New-CandidateTaskXml($Binding) {
    $arguments = '-NoLogo -NoProfile -NonInteractive -WindowStyle Hidden -File "{0}" -Action Start -CandidateId {1}' -f $Binding.LauncherPath, $Binding.CandidateId
    $escapedCommand = [Security.SecurityElement]::Escape($Binding.PowerShellPath)
    $escapedArgs = [Security.SecurityElement]::Escape($arguments)
    $escapedWork = [Security.SecurityElement]::Escape((Split-Path -Parent $Binding.LauncherPath))
    $escapedId = [Security.SecurityElement]::Escape($Binding.InstanceName)
    $escapedSid = [Security.SecurityElement]::Escape($Binding.CurrentUserSid)
    @"
<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.4" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo><Description>ReaderProCandidateAutostart; candidate=$escapedId; managed-by=ReaderCandidateAutostart.ps1</Description></RegistrationInfo>
  <Triggers><LogonTrigger><Enabled>true</Enabled><UserId>$escapedSid</UserId></LogonTrigger></Triggers>
  <Principals><Principal id="ReaderProCurrentUser"><UserId>$escapedSid</UserId><LogonType>InteractiveToken</LogonType><RunLevel>LeastPrivilege</RunLevel></Principal></Principals>
  <Settings><MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy><DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries><StopIfGoingOnBatteries>false</StopIfGoingOnBatteries><AllowHardTerminate>true</AllowHardTerminate><StartWhenAvailable>false</StartWhenAvailable><RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable><AllowStartOnDemand>false</AllowStartOnDemand><Enabled>true</Enabled><Hidden>true</Hidden><ExecutionTimeLimit>PT5M</ExecutionTimeLimit><Priority>7</Priority></Settings>
  <Actions Context="ReaderProCurrentUser"><Exec><Command>$escapedCommand</Command><Arguments>$escapedArgs</Arguments><WorkingDirectory>$escapedWork</WorkingDirectory></Exec></Actions>
</Task>
"@
}

function Initialize-TestSchedulerAdapter($Binding) {
    if ([string]::IsNullOrWhiteSpace($TestSchedulerAdapterPath)) { return }
    if ($env:READER_CANDIDATE_AUTOSTART_TEST_MODE -cne '1') { Fail '测试计划任务适配器只能在显式隔离测试模式下使用。' }
    $adapterPath = Test-ReparseFreePath (Get-AbsoluteWindowsPath $TestSchedulerAdapterPath 'TestSchedulerAdapterPath')
    if ((Split-Path -Parent $adapterPath) -cne $Binding.Root -or (Split-Path -Leaf $adapterPath) -cne 'candidate-test-scheduler-adapter.ps1') {
        Fail '测试适配器必须是候选根目录内固定命名的生成文件。'
    }
    $adapter = & $adapterPath
    if ($adapter -isnot [Collections.IDictionary]) { Fail '测试适配器必须返回函数表。' }
    foreach ($name in 'Get', 'Export', 'Register', 'Unregister') {
        if ($adapter[$name] -isnot [ScriptBlock]) { Fail "测试适配器缺少 $name 函数。" }
    }
    $script:TestSchedulerAdapter = $adapter
    $script:UsingTestSchedulerAdapter = $true
}

function Test-NotFoundError($ErrorRecord) {
    $supportedType = ($ErrorRecord.Exception -is [Management.Automation.ItemNotFoundException] -or
                      $ErrorRecord.Exception -is [Microsoft.Management.Infrastructure.CimException])
    return ($ErrorRecord.CategoryInfo.Category -eq [Management.Automation.ErrorCategory]::ObjectNotFound -and $supportedType)
}

function Get-ExistingCandidateTask($Binding) {
    try {
        if ($script:UsingTestSchedulerAdapter) { return & $script:TestSchedulerAdapter['Get'] $Binding.TaskName $Binding.TaskPath }
        return Get-ScheduledTask -TaskName $Binding.TaskName -TaskPath $Binding.TaskPath -ErrorAction Stop
    }
    catch {
        if (Test-NotFoundError $_) { return $null }
        throw
    }
}

function Export-CandidateTask($Binding) {
    if ($script:UsingTestSchedulerAdapter) { return & $script:TestSchedulerAdapter['Export'] $Binding.TaskName $Binding.TaskPath }
    return Export-ScheduledTask -TaskName $Binding.TaskName -TaskPath $Binding.TaskPath -ErrorAction Stop
}

function Register-CandidateTask($Binding, [string]$Xml) {
    if ($script:UsingTestSchedulerAdapter) { & $script:TestSchedulerAdapter['Register'] $Binding.TaskName $Binding.TaskPath $Xml; return }
    Register-ScheduledTask -TaskName $Binding.TaskName -TaskPath $Binding.TaskPath -Xml $Xml | Out-Null
}

function Unregister-CandidateTask($Binding) {
    if ($script:UsingTestSchedulerAdapter) { & $script:TestSchedulerAdapter['Unregister'] $Binding.TaskName $Binding.TaskPath; return }
    Unregister-ScheduledTask -TaskName $Binding.TaskName -TaskPath $Binding.TaskPath -Confirm:$false
}

function Test-CandidateTaskIdentity($Task, $Binding, [string]$ExpectedXml) {
    if ($null -eq $Task) { return $false }
    [xml]$xml = Export-CandidateTask $Binding
    $ns = New-Object Xml.XmlNamespaceManager($xml.NameTable)
    $ns.AddNamespace('t', 'http://schemas.microsoft.com/windows/2004/02/mit/task')
    $arguments = '-NoLogo -NoProfile -NonInteractive -WindowStyle Hidden -File "{0}" -Action Start -CandidateId {1}' -f $Binding.LauncherPath, $Binding.CandidateId
    # Scheduler is allowed to normalize XML declaration/order.  Deletion and
    # overwrite require every security-relevant managed field to still match.
    $marker = 'ReaderProCandidateAutostart; candidate={0}; managed-by=ReaderCandidateAutostart.ps1' -f $Binding.InstanceName
    $text = {
        param([string]$Path)
        $node = $xml.SelectSingleNode($Path, $ns)
        if ($null -eq $node) { return $null }
        return $node.InnerText
    }
    $settings = $xml.SelectSingleNode('/t:Task/t:Settings', $ns)
    return (
        $xml.SelectNodes('/t:Task/t:RegistrationInfo/t:Description', $ns).Count -eq 1 -and
        (& $text '/t:Task/t:RegistrationInfo/t:Description') -ceq $marker -and
        $xml.SelectNodes('/t:Task/t:Triggers/t:LogonTrigger', $ns).Count -eq 1 -and
        $xml.SelectNodes('/t:Task/t:Triggers/*', $ns).Count -eq 1 -and
        $xml.SelectNodes('/t:Task/t:Triggers/t:LogonTrigger/*', $ns).Count -eq 2 -and
        (& $text '/t:Task/t:Triggers/t:LogonTrigger/t:Enabled') -ceq 'true' -and
        (& $text '/t:Task/t:Triggers/t:LogonTrigger/t:UserId') -ceq $Binding.CurrentUserSid -and
        $xml.SelectNodes('/t:Task/t:Principals', $ns).Count -eq 1 -and
        $xml.SelectNodes('/t:Task/t:Principals/t:Principal', $ns).Count -eq 1 -and
        $xml.SelectSingleNode('/t:Task/t:Principals/t:Principal', $ns).GetAttribute('id') -ceq 'ReaderProCurrentUser' -and
        (& $text '/t:Task/t:Principals/t:Principal/t:UserId') -ceq $Binding.CurrentUserSid -and
        (& $text '/t:Task/t:Principals/t:Principal/t:LogonType') -ceq 'InteractiveToken' -and
        (& $text '/t:Task/t:Principals/t:Principal/t:RunLevel') -ceq 'LeastPrivilege' -and
        $xml.SelectNodes('/t:Task/t:Actions', $ns).Count -eq 1 -and
        $xml.SelectSingleNode('/t:Task/t:Actions', $ns).GetAttribute('Context') -ceq 'ReaderProCurrentUser' -and
        $xml.SelectNodes('/t:Task/t:Actions/t:Exec', $ns).Count -eq 1 -and
        $xml.SelectNodes('/t:Task/t:Actions/*', $ns).Count -eq 1 -and
        (& $text '/t:Task/t:Actions/t:Exec/t:Command') -ceq $Binding.PowerShellPath -and
        (& $text '/t:Task/t:Actions/t:Exec/t:Arguments') -ceq $arguments -and
        (& $text '/t:Task/t:Actions/t:Exec/t:WorkingDirectory') -ceq (Split-Path -Parent $Binding.LauncherPath) -and
        $xml.SelectNodes('/t:Task/t:Settings', $ns).Count -eq 1 -and
        $xml.SelectNodes('/t:Task/t:Settings/*', $ns).Count -eq 11 -and
        (& $text '/t:Task/t:Settings/t:MultipleInstancesPolicy') -ceq 'IgnoreNew' -and
        (& $text '/t:Task/t:Settings/t:DisallowStartIfOnBatteries') -ceq 'false' -and
        (& $text '/t:Task/t:Settings/t:StopIfGoingOnBatteries') -ceq 'false' -and
        (& $text '/t:Task/t:Settings/t:AllowHardTerminate') -ceq 'true' -and
        (& $text '/t:Task/t:Settings/t:StartWhenAvailable') -ceq 'false' -and
        (& $text '/t:Task/t:Settings/t:RunOnlyIfNetworkAvailable') -ceq 'false' -and
        (& $text '/t:Task/t:Settings/t:AllowStartOnDemand') -ceq 'false' -and
        (& $text '/t:Task/t:Settings/t:Enabled') -ceq 'true' -and
        (& $text '/t:Task/t:Settings/t:Hidden') -ceq 'true' -and
        (& $text '/t:Task/t:Settings/t:ExecutionTimeLimit') -ceq 'PT5M' -and
        (& $text '/t:Task/t:Settings/t:Priority') -ceq '7'
    )
}

function Get-PlanObject($Binding, [string]$Xml, [string]$Mode, [bool]$Existing, [bool]$ExactMatch) {
    [PSCustomObject]@{
        action = $Mode; executeRequested = [bool]$Execute; candidateId = $Binding.CandidateId; instanceName = $Binding.InstanceName
        taskFullName = $Binding.TaskFullName; launcherPath = $Binding.LauncherPath
        launcherSha256 = $Binding.LauncherSha256; principal = 'current-user InteractiveToken / LeastPrivilege'
        trigger = 'LogonTrigger only'; bootTrigger = $false; systemTask = $false; highestPrivilege = $false
        hiddenWindow = $true; existingTask = $Existing; exactManagedMatch = $ExactMatch
        wouldRegister = ($Mode -eq 'Install' -and -not $Existing)
        wouldRemove = ($Mode -eq 'Uninstall' -and $ExactMatch)
        taskXml = $Xml
    }
}

$script:UsingTestSchedulerAdapter = $false
$script:TestSchedulerAdapter = $null
$binding = Get-CandidateBinding $ProfilePath
Initialize-TestSchedulerAdapter $binding
$xml = New-CandidateTaskXml $binding

if ($Action -eq 'Plan') {
    Get-PlanObject $binding $xml 'Plan' $false $false | ConvertTo-Json -Depth 4
    return
}

$existing = Get-ExistingCandidateTask $binding
$exact = Test-CandidateTaskIdentity $existing $binding $xml
if ($Action -eq 'Inspect') {
    Get-PlanObject $binding $xml 'Inspect' ($null -ne $existing) $exact | ConvertTo-Json -Depth 4
    return
}

if (-not $Execute) { Fail "$Action 默认只允许 Plan；若确认执行，必须同时提供 -Execute 和 -ConfirmCandidateId $($binding.CandidateId)。" }
if ($ConfirmCandidateId -cne $binding.CandidateId) { Fail 'ConfirmCandidateId 必须精确匹配 profile 的 candidateId。' }

$plan = Get-PlanObject $binding $xml $Action ($null -ne $existing) $exact
if ($Action -eq 'Install') {
    if ($null -ne $existing) {
        if (-not $exact) { Fail '已有同名任务不完全匹配本工具身份；拒绝覆盖。' }
        $plan | ConvertTo-Json -Depth 4
        return
    }
    # No -Force: a task appearing in the check/register race must fail closed,
    # never overwrite a task we did not inspect.
    Register-CandidateTask $binding $xml
    $after = Get-ExistingCandidateTask $binding
    if (-not (Test-CandidateTaskIdentity $after $binding $xml)) { Fail '计划任务创建后身份核验失败；未将其视为已启用。' }
    $plan | Add-Member NoteProperty applied $true
    $plan | ConvertTo-Json -Depth 4
    return
}

if ($Action -eq 'Uninstall') {
    if ($null -eq $existing) { $plan | ConvertTo-Json -Depth 4; return }
    if (-not $exact) { Fail '已有任务不完全匹配本工具身份；拒绝删除。' }
    Unregister-CandidateTask $binding
    if ($null -ne (Get-ExistingCandidateTask $binding)) { Fail '计划任务删除后仍存在；未将其视为已移除。' }
    $plan | Add-Member NoteProperty applied $true
    $plan | ConvertTo-Json -Depth 4
}
