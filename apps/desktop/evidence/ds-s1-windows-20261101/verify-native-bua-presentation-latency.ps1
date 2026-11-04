$ErrorActionPreference = 'Stop'

$desktopExe = 'F:\workspace\yonder\target\debug\yonder-desktop.exe'
$registerAgentExe = 'F:\workspace\yonder\target\debug\examples\register-agent.exe'
$applicationData = Join-Path $env:APPDATA 'com.yonder.desktop'
$agentId = 'ds-s1-bua-latency-' + [guid]::NewGuid().ToString('N').Substring(0, 10)
$pipeName = 'com.yonder.desktop.agent'
$requestNumber = 0
$ownedProcess = $null
$session = $null
$registered = $false
$finished = $false
$stderrRead = $null
$diagnosticLines = [Collections.Generic.List[object]]::new()
$result = [ordered]@{ platform='windows'; target_ms=300; cua_executed=$false; ego_lite_executed=$true }

function Get-Deadline { [long](([datetime]::UtcNow - [datetime]'1970-01-01').TotalMilliseconds + 60000) }

function Connect-Gateway {
    $until = [datetime]::UtcNow.AddSeconds(30)
    do {
        $pipe = [IO.Pipes.NamedPipeClientStream]::new('.', $pipeName, [IO.Pipes.PipeDirection]::InOut, [IO.Pipes.PipeOptions]::Asynchronous)
        try {
            $pipe.Connect(500)
            $encoding = [Text.UTF8Encoding]::new($false)
            $reader = [IO.StreamReader]::new($pipe, $encoding, $false, 65536, $true)
            $writer = [IO.StreamWriter]::new($pipe, $encoding, 65536, $true)
            $writer.AutoFlush = $true
            return @{Pipe=$pipe;Reader=$reader;Writer=$writer}
        } catch {
            $pipe.Dispose()
            Start-Sleep -Milliseconds 200
        }
    } while ([datetime]::UtcNow -lt $until)
    throw 'Yonder Named Pipe未在期限内就绪'
}

function New-Request([string]$Method, [string]$Capability, [hashtable]$Parameters) {
    $script:requestNumber++
    $params = [ordered]@{agent_id=$agentId;capability=$Capability;deadline=Get-Deadline}
    foreach ($entry in $Parameters.GetEnumerator()) { $params[$entry.Key] = $entry.Value }
    [ordered]@{jsonrpc='2.0';id="ds-s1-bua-latency-$script:requestNumber";method=$Method;params=$params}
}

function Complete-Response([hashtable]$Request, [string]$Line) {
    if ([string]::IsNullOrWhiteSpace($Line)) { throw "Gateway未响应：$($Request.method)" }
    $response = $Line | ConvertFrom-Json
    if ($response.id -ne $Request.id) { throw "Gateway响应ID不匹配：$($Request.method)" }
    if ($null -ne $response.error) { throw ("Gateway调用失败 {0}: {1}" -f $Request.method, ($response.error | ConvertTo-Json -Compress)) }
    $response.result
}

function Send-Request([hashtable]$Connection, [string]$Method, [string]$Capability, [hashtable]$Parameters) {
    $request = New-Request $Method $Capability $Parameters
    $Connection.Writer.WriteLine(($request | ConvertTo-Json -Compress -Depth 12))
    Complete-Response $request $Connection.Reader.ReadLine()
}

function Wait-PresentationChange([string]$State, [Diagnostics.Stopwatch]$Watch, $ResponseTask) {
    while ($Watch.Elapsed.TotalSeconds -lt 15) {
        if ($script:stderrRead.IsCompleted) {
            $line = $script:stderrRead.GetAwaiter().GetResult()
            if ($null -eq $line) { throw '桌面诊断流提前关闭' }
            $observed = [Math]::Round($Watch.Elapsed.TotalMilliseconds, 3)
            $script:diagnosticLines.Add([ordered]@{elapsed_ms=$observed;line=$line})
            $script:stderrRead = $script:ownedProcess.StandardError.ReadLineAsync()
            if ($line -match 'phase=presentation-change' -and $line -match "task_state=$([regex]::Escape($State))(,|$)") {
                return [ordered]@{state=$State;latency_ms_upper_bound=$observed;response_in_flight=($ResponseTask -and -not $ResponseTask.IsCompleted);diagnostic_line=$line}
            }
        } else {
            Start-Sleep -Milliseconds 2
        }
    }
    throw "15秒内未观察到展示状态：$State"
}

function Send-RequestWithLatency([hashtable]$Connection, [string]$Method, [string]$Capability, [hashtable]$Parameters, [string]$State) {
    $request = New-Request $Method $Capability $Parameters
    $watch = [Diagnostics.Stopwatch]::StartNew()
    $Connection.Writer.WriteLine(($request | ConvertTo-Json -Compress -Depth 12))
    $responseTask = $Connection.Reader.ReadLineAsync()
    $observation = Wait-PresentationChange $State $watch $responseTask
    $line = $responseTask.GetAwaiter().GetResult()
    [ordered]@{result=(Complete-Response $request $line);observation=$observation;request_duration_ms=[Math]::Round($watch.Elapsed.TotalMilliseconds,3)}
}

function Close-Session {
    if (-not $session) { return }
    $session.Writer.Dispose()
    $session.Reader.Dispose()
    $session.Pipe.Dispose()
    $script:session = $null
}

if (-not (Test-Path -LiteralPath $desktopExe -PathType Leaf) -or -not (Test-Path -LiteralPath $registerAgentExe -PathType Leaf)) { throw '缺少Windows桌面或Agent注册夹具' }
if (Get-Process yonder-desktop -ErrorAction SilentlyContinue) { throw '检测到现有Yonda进程，拒绝干扰' }

try {
    & $registerAgentExe $applicationData $agentId
    if ($LASTEXITCODE -ne 0) { throw 'Agent登记失败' }
    $registered = $true

    $start = New-Object Diagnostics.ProcessStartInfo
    $start.FileName = $desktopExe
    $start.WorkingDirectory = Split-Path -Parent $desktopExe
    $start.UseShellExecute = $false
    $start.RedirectStandardError = $true
    $ownedProcess = [Diagnostics.Process]::Start($start)
    $stderrRead = $ownedProcess.StandardError.ReadLineAsync()
    $result.process_id = $ownedProcess.Id

    $session = Connect-Gateway
    $hello = Send-Request $session 'gateway.hello' 'task.read' @{protocol_version=[ordered]@{major=1;minor=19}}
    $browser = @($hello.capabilities | Where-Object { $_.name -eq 'browser.execute' })
    if ($browser.Count -ne 1 -or $browser[0].availability -ne 'available') { throw 'Windows browser.execute不可用' }
    $result.agent_id = $agentId
    $result.protocol = [ordered]@{major=$hello.protocol_version.major;minor=$hello.protocol_version.minor}
    Start-Sleep -Seconds 4

    $created = Send-Request $session 'task.create' 'task.create' @{idempotency_key="ds-s1-bua-latency-$([guid]::NewGuid().ToString('N'))";description='验证Windows BUA核心展示状态延迟';name='DS-S1 Windows BUA状态延迟验证'}
    $task = $created.task
    $result.task_id = $task.task_id
    $declared = Send-Request $session 'task.step.declare' 'task.step.declare' @{task_id=$task.task_id;expected_sequence=$task.sequence;step_id='open-space';label='创建Browser Task Space'}
    $openedCall = Send-RequestWithLatency $session 'browser.execute' 'browser.execute' @{task_id=$task.task_id;expected_sequence=$declared.task.sequence;operation='create'} 'executing'
    $opened = $openedCall.result
    if ($opened.task.status -ne 'running' -or $opened.reference.external_task_ref -notmatch '^ego:[1-9][0-9]*$') { throw 'Browser创建未返回运行中引用' }
    $result.external_task_ref = $opened.reference.external_task_ref
    $result.executing = $openedCall.observation

    $advanced = Send-Request $session 'task.step.advance' 'task.step.advance' @{task_id=$task.task_id;expected_sequence=$opened.task.sequence}
    $declaredFinish = Send-Request $session 'task.step.declare' 'task.step.declare' @{task_id=$task.task_id;expected_sequence=$advanced.task.sequence;step_id='finish-space';label='完成Browser Task Space'}
    $finishedCall = Send-RequestWithLatency $session 'browser.execute' 'browser.execute' @{task_id=$task.task_id;expected_sequence=$declaredFinish.task.sequence;operation='finish'} 'success'
    $completed = $finishedCall.result
    if ($completed.task.status -ne 'completed' -or -not $completed.reference.finished) { throw 'Browser finish未完成任务或关闭空间' }
    $finished = $true
    $result.success_from_request = $finishedCall.observation

    $resumeWatch = [Diagnostics.Stopwatch]::StartNew()
    $result.resume = Wait-PresentationChange 'paused' $resumeWatch $null
    $result.final_status = $completed.task.status
    $result.reference_finished = $completed.reference.finished
    $result.request_durations_ms = [ordered]@{browser_create=$openedCall.request_duration_ms;browser_finish=$finishedCall.request_duration_ms}
    $result.diagnostic_observations = $diagnosticLines
    $result.optimization_target_met = $openedCall.observation.latency_ms_upper_bound -lt 300
    $result.success_latency_note = 'success从browser.finish请求写入计时，包含外部Task Space完成耗时，不与300毫秒UI状态应用目标比较。'
    $result.resume_latency_note = 'resume从finish响应完成后计时，包含既有1.8秒终态展示脉冲，不与300毫秒UI状态应用目标比较。'
    $result.passed = $finished -and $result.optimization_target_met -and $result.resume.state -eq 'paused'
    if (-not $result.passed) { throw 'BUA核心展示状态延迟证据门禁未通过' }
} catch {
    $result.passed = $false
    $result.error = $_.Exception.Message
    throw
} finally {
    Close-Session
    if ($ownedProcess -and -not $ownedProcess.HasExited) { $ownedProcess.Kill(); $ownedProcess.WaitForExit() }
    if ($registered) { & $registerAgentExe $applicationData $agentId disable | Out-Null }
    $result.diagnostic_observations = $diagnosticLines
    $result.finished = $finished
    $result | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'native-bua-presentation-latency-result.json') -Encoding UTF8
}

$result | ConvertTo-Json -Depth 10 -Compress
