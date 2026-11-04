param(
    [Parameter(Mandatory = $true)]
    [string]$DesktopExe
)

$ErrorActionPreference = 'Stop'
$agentId = 'bu-s2-win-' + [guid]::NewGuid().ToString('N').Substring(0, 12)
$pipeName = 'com.yonder.desktop.agent'
$runRoot = Join-Path ([IO.Path]::GetTempPath()) ("yonda-bu-s2-" + [guid]::NewGuid().ToString('N'))
$appData = Join-Path $runRoot 'Roaming'
$localAppData = Join-Path $runRoot 'Local'
$originalAppData = $env:APPDATA
$originalLocalAppData = $env:LOCALAPPDATA
$applicationData = Join-Path $originalAppData 'com.yonder.desktop'
$registerAgentExe = Join-Path (Split-Path -Parent $DesktopExe) 'examples\register-agent.exe'
$ownedProcesses = [Collections.Generic.List[Diagnostics.Process]]::new()
$requestNumber = 0

function Get-Deadline {
    return [long](([datetime]::UtcNow - [datetime]'1970-01-01').TotalMilliseconds) + 60000
}

function Stop-OwnedDesktop {
    param([Diagnostics.Process]$Process)
    if ($null -eq $Process -or $Process.HasExited) { return }
    [void]$Process.CloseMainWindow()
    if (-not $Process.WaitForExit(3000)) {
        $Process.Kill()
        $Process.WaitForExit()
    }
}

function Start-OwnedDesktop {
    $process = Start-Process -FilePath $DesktopExe -WorkingDirectory (Split-Path -Parent $DesktopExe) -PassThru
    $ownedProcesses.Add($process)
    return $process
}

function Connect-Gateway {
    param([int]$TimeoutSeconds = 30)
    $until = [datetime]::UtcNow.AddSeconds($TimeoutSeconds)
    do {
        $pipe = [IO.Pipes.NamedPipeClientStream]::new('.', $pipeName, [IO.Pipes.PipeDirection]::InOut, [IO.Pipes.PipeOptions]::Asynchronous)
        try {
            $pipe.Connect(500)
            $encoding = [Text.UTF8Encoding]::new($false)
            $reader = [IO.StreamReader]::new($pipe, $encoding, $false, 65536, $true)
            $writer = [IO.StreamWriter]::new($pipe, $encoding, 65536, $true)
            $writer.AutoFlush = $true
            return @{ Pipe = $pipe; Reader = $reader; Writer = $writer }
        } catch {
            $pipe.Dispose()
            Start-Sleep -Milliseconds 200
        }
    } while ([datetime]::UtcNow -lt $until)
    throw 'Yonder Named Pipe未在期限内就绪'
}

function Send-Request {
    param(
        [hashtable]$Connection,
        [string]$Method,
        [string]$Capability,
        [hashtable]$Parameters
    )
    $script:requestNumber++
    $params = [ordered]@{
        agent_id = $agentId
        capability = $Capability
        deadline = Get-Deadline
    }
    foreach ($entry in $Parameters.GetEnumerator()) { $params[$entry.Key] = $entry.Value }
    $request = [ordered]@{
        jsonrpc = '2.0'
        id = "windows-$script:requestNumber"
        method = $Method
        params = $params
    }
    $Connection.Writer.WriteLine(($request | ConvertTo-Json -Compress -Depth 12))
    $line = $Connection.Reader.ReadLine()
    if ([string]::IsNullOrWhiteSpace($line)) { throw "Gateway未响应：$Method" }
    $response = $line | ConvertFrom-Json
    if ($response.id -ne $request.id) { throw "Gateway响应ID不匹配：$Method" }
    if ($null -ne $response.error) { throw ("Gateway调用失败 {0}: {1}" -f $Method, ($response.error | ConvertTo-Json -Compress)) }
    return $response.result
}

function Open-Session {
    $connection = Connect-Gateway
    $hello = Send-Request $connection 'gateway.hello' 'task.read' @{
        protocol_version = [ordered]@{ major = 1; minor = 18 }
    }
    if ($hello.protocol_version.major -ne 1 -or $hello.protocol_version.minor -lt 15) { throw 'Gateway协议版本不足' }
    $browserCapability = @($hello.capabilities | Where-Object { $_.name -eq 'browser.execute' })
    if ($browserCapability.Count -ne 1 -or $browserCapability[0].availability -ne 'available') { throw 'Windows browser.execute不可用' }
    return @{ Connection = $connection; Hello = $hello }
}

function Close-Session {
    param([hashtable]$Session)
    if ($null -eq $Session) { return }
    $Session.Connection.Writer.Dispose()
    $Session.Connection.Reader.Dispose()
    $Session.Connection.Pipe.Dispose()
}

function Declare-Step {
    param([hashtable]$Connection, [string]$TaskId, [string]$Sequence, [string]$StepId, [string]$Label)
    return Send-Request $Connection 'task.step.declare' 'task.step.declare' @{
        task_id = $TaskId; expected_sequence = $Sequence; step_id = $StepId; label = $Label
    }
}

function Execute-Browser {
    param([hashtable]$Connection, [string]$TaskId, [string]$Sequence, [string]$Operation)
    return Send-Request $Connection 'browser.execute' 'browser.execute' @{
        task_id = $TaskId; expected_sequence = $Sequence; operation = $Operation
    }
}

function Advance-Step {
    param([hashtable]$Connection, [string]$TaskId, [string]$Sequence)
    return Send-Request $Connection 'task.step.advance' 'task.step.advance' @{
        task_id = $TaskId; expected_sequence = $Sequence
    }
}

function Read-BrowserReference {
    param([hashtable]$Connection, [string]$TaskId)
    return Send-Request $Connection 'task.browser.get' 'task.read' @{ task_id = $TaskId }
}

New-Item -ItemType Directory -Path $appData, $localAppData -Force | Out-Null
$env:APPDATA = $appData
$env:LOCALAPPDATA = $localAppData
$session = $null
$reopenedSession = $null

try {
    if (-not (Test-Path -LiteralPath $registerAgentExe -PathType Leaf)) { throw '缺少register-agent验证夹具' }
    & $registerAgentExe $applicationData $agentId
    if ($LASTEXITCODE -ne 0) { throw '隔离数据库Agent登记失败' }
    $desktop = Start-OwnedDesktop
    $session = Open-Session
    $connection = $session.Connection
    $created = Send-Request $connection 'task.create' 'task.create' @{
        idempotency_key = "bu-s2-windows-$([guid]::NewGuid().ToString('N'))"
        description = '验证Windows Browser引用、控制权与重启恢复'
        name = 'BU-S2 Windows验证'
    }
    $task = $created.task
    $declared = Declare-Step $connection $task.task_id $task.sequence 'create-space' '创建Browser Task Space'
    $opened = Execute-Browser $connection $task.task_id $declared.task.sequence 'create'
    if ($opened.task.status -ne 'running' -or $opened.reference.finished -or $opened.reference.external_task_ref -notmatch '^ego:[1-9][0-9]*$') { throw 'Browser创建结果不符合契约' }
    $externalRef = $opened.reference.external_task_ref
    $readAfterCreate = Read-BrowserReference $connection $task.task_id
    if ($readAfterCreate.reference.external_task_ref -ne $externalRef -or $readAfterCreate.reference.ownership -ne 'agent') { throw '创建后的引用读取不一致' }

    $advanced = Advance-Step $connection $task.task_id $opened.task.sequence
    $declared = Declare-Step $connection $task.task_id $advanced.task.sequence 'hand-off' '交给用户'
    $handed = Execute-Browser $connection $task.task_id $declared.task.sequence 'hand-off'
    if ($handed.reference.external_task_ref -ne $externalRef -or $handed.reference.ownership -ne 'agentDelegatedToUser') { throw 'hand-off未保持引用或控制权' }

    $advanced = Advance-Step $connection $task.task_id $handed.task.sequence
    $declared = Declare-Step $connection $task.task_id $advanced.task.sequence 'take-over' 'Agent恢复控制'
    $taken = Execute-Browser $connection $task.task_id $declared.task.sequence 'take-over'
    if ($taken.reference.external_task_ref -ne $externalRef -or $taken.reference.ownership -ne 'agent') { throw 'take-over未恢复同一引用' }

    $advanced = Advance-Step $connection $task.task_id $taken.task.sequence
    $declared = Declare-Step $connection $task.task_id $advanced.task.sequence 'finish-space' '完成Browser Task Space'
    $finished = Execute-Browser $connection $task.task_id $declared.task.sequence 'finish'
    if ($finished.task.status -ne 'completed' -or -not $finished.reference.finished -or $finished.reference.external_task_ref -ne $externalRef) { throw 'finish未完成任务或保留引用审计' }
    $events = Send-Request $connection 'task.events' 'task.read' @{ task_id = $task.task_id; after_sequence = '0'; limit = 100 }
    $observedAttempts = @($events.events | Where-Object { $_.attempt_result.phase -eq 'observed' }).Count
    if ($observedAttempts -ne 4) { throw "Observed attempt数量错误：$observedAttempts" }
    Close-Session $session
    $session = $null
    Stop-OwnedDesktop $desktop

    $desktop = Start-OwnedDesktop
    $reopenedSession = Open-Session
    $afterRestart = Read-BrowserReference $reopenedSession.Connection $task.task_id
    if (-not $afterRestart.reference.finished -or $afterRestart.reference.external_task_ref -ne $externalRef) { throw '桌面重启后引用未恢复' }

    [ordered]@{
        story = 'BU-S2'
        platform = 'windows'
        protocol = [ordered]@{ major = $reopenedSession.Hello.protocol_version.major; minor = $reopenedSession.Hello.protocol_version.minor }
        browser_capability = 'available'
        task_id = $task.task_id
        external_task_ref = $externalRef
        ownership_path = @('agent', 'agentDelegatedToUser', 'agent')
        final_status = $finished.task.status
        final_sequence = $finished.task.sequence
        events = @($events.events).Count
        observed_attempts = $observedAttempts
        persisted_after_desktop_restart = $true
        finished_reference_retained = $afterRestart.reference.finished
        cua_executed = $false
        passed = $true
    } | ConvertTo-Json -Depth 8
} finally {
    Close-Session $session
    Close-Session $reopenedSession
    foreach ($process in $ownedProcesses) { Stop-OwnedDesktop $process }
    if (Test-Path -LiteralPath $registerAgentExe -PathType Leaf) {
        & $registerAgentExe $applicationData $agentId disable
    }
    $env:APPDATA = $originalAppData
    $env:LOCALAPPDATA = $originalLocalAppData
    if (Test-Path -LiteralPath $runRoot) { Remove-Item -LiteralPath $runRoot -Recurse -Force }
}
