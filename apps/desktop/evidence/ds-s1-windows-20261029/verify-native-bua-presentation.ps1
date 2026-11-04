$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class YondaPresentationProbe {
    private delegate bool EnumWindowsProc(IntPtr handle, IntPtr state);
    [DllImport("user32.dll")]
    private static extern bool EnumWindows(EnumWindowsProc callback, IntPtr state);
    [DllImport("user32.dll")]
    private static extern uint GetWindowThreadProcessId(IntPtr handle, out uint processId);
    [DllImport("user32.dll")]
    private static extern bool IsWindowVisible(IntPtr handle);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    private static extern int GetWindowText(IntPtr handle, StringBuilder text, int count);
    [DllImport("user32.dll")]
    private static extern bool GetWindowRect(IntPtr handle, out Rect rect);
    [DllImport("user32.dll")]
    private static extern bool SetWindowPos(IntPtr handle, IntPtr insertAfter, int x, int y, int width, int height, uint flags);
    [DllImport("user32.dll")]
    private static extern uint GetDpiForWindow(IntPtr handle);
    [StructLayout(LayoutKind.Sequential)]
    private struct Rect { public int Left, Top, Right, Bottom; }

    private static IntPtr FindPetHandle(int expectedProcessId) {
        IntPtr result = IntPtr.Zero;
        EnumWindows(delegate(IntPtr handle, IntPtr state) {
            uint processId;
            GetWindowThreadProcessId(handle, out processId);
            if (processId != expectedProcessId || !IsWindowVisible(handle)) return true;
            var title = new StringBuilder(128);
            GetWindowText(handle, title, title.Capacity);
            if (!title.ToString().StartsWith("Yonda")) return true;
            Rect rect;
            if (!GetWindowRect(handle, out rect) || rect.Right - rect.Left != 200 || rect.Bottom - rect.Top != 200) return true;
            result = handle;
            return false;
        }, IntPtr.Zero);
        return result;
    }

    public static int[] FindPet(int expectedProcessId) {
        var handle = FindPetHandle(expectedProcessId);
        Rect rect;
        if (handle == IntPtr.Zero || !GetWindowRect(handle, out rect)) return null;
        return new[] { rect.Left, rect.Top, rect.Right - rect.Left, rect.Bottom - rect.Top };
    }
    public static bool MovePetWithoutActivation(int expectedProcessId, int x, int y) {
        var handle = FindPetHandle(expectedProcessId);
        return handle != IntPtr.Zero && SetWindowPos(handle, IntPtr.Zero, x, y, 0, 0, 0x0001 | 0x0004 | 0x0010);
    }
    public static uint PetDpi(int expectedProcessId) {
        var handle = FindPetHandle(expectedProcessId);
        return handle == IntPtr.Zero ? 0 : GetDpiForWindow(handle);
    }
}
'@

$desktopExe = 'F:\workspace\yonder\target\debug\yonder-desktop.exe'
$registerAgentExe = 'F:\workspace\yonder\target\debug\examples\register-agent.exe'
$applicationData = Join-Path $env:APPDATA 'com.yonder.desktop'
$agentId = 'ds-s1-win-' + [guid]::NewGuid().ToString('N').Substring(0, 12)
$pipeName = 'com.yonder.desktop.agent'
$requestNumber = 0
$ownedProcess = $null
$session = $null
$registered = $false
$finished = $false
$result = [ordered]@{ platform = 'windows'; flow = @('idle','listening','executing','success','idle') }

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
            return @{ Pipe = $pipe; Reader = $reader; Writer = $writer }
        } catch { $pipe.Dispose(); Start-Sleep -Milliseconds 200 }
    } while ([datetime]::UtcNow -lt $until)
    throw 'Yonder Named Pipe未在期限内就绪'
}

function New-Request([string]$Method, [string]$Capability, [hashtable]$Parameters) {
    $script:requestNumber++
    $params = [ordered]@{ agent_id = $agentId; capability = $Capability; deadline = Get-Deadline }
    foreach ($entry in $Parameters.GetEnumerator()) { $params[$entry.Key] = $entry.Value }
    [ordered]@{ jsonrpc = '2.0'; id = "ds-s1-$script:requestNumber"; method = $Method; params = $params }
}

function Complete-Response([hashtable]$Connection, [hashtable]$Request, [string]$Line) {
    if ([string]::IsNullOrWhiteSpace($Line)) { throw "Gateway未响应：$($Request.method)" }
    $response = $Line | ConvertFrom-Json
    if ($response.id -ne $Request.id) { throw "Gateway响应ID不匹配：$($Request.method)" }
    if ($null -ne $response.error) { throw ("Gateway调用失败 {0}: {1}" -f $Request.method, ($response.error | ConvertTo-Json -Compress)) }
    return $response.result
}

function Send-Request([hashtable]$Connection, [string]$Method, [string]$Capability, [hashtable]$Parameters) {
    $request = New-Request $Method $Capability $Parameters
    $Connection.Writer.WriteLine(($request | ConvertTo-Json -Compress -Depth 12))
    Complete-Response $Connection $request $Connection.Reader.ReadLine()
}

function Capture-Pet([string]$Name) {
    $rect = [YondaPresentationProbe]::FindPet($ownedProcess.Id)
    if (-not $rect) { throw "截图时Yonda窗口不可用：$Name" }
    $dpi = [YondaPresentationProbe]::PetDpi($ownedProcess.Id)
    $scale = $dpi / 96.0
    $x = [int][Math]::Round($rect[0] * $scale)
    $y = [int][Math]::Round($rect[1] * $scale)
    $width = [int][Math]::Round($rect[2] * $scale)
    $height = [int][Math]::Round($rect[3] * $scale)
    $bitmap = New-Object Drawing.Bitmap($width, $height, [Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.CopyFromScreen($x, $y, 0, 0, (New-Object Drawing.Size($width, $height)), [Drawing.CopyPixelOperation]::SourceCopy)
        $path = Join-Path $PSScriptRoot "$Name.png"
        $bitmap.Save($path, [Drawing.Imaging.ImageFormat]::Png)
        [ordered]@{ file = "$Name.png"; sha256 = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant(); logical = [ordered]@{ x=$rect[0]; y=$rect[1]; width=$rect[2]; height=$rect[3] }; physical = [ordered]@{ x=$x; y=$y; width=$width; height=$height }; dpi=$dpi }
    } finally { $graphics.Dispose(); $bitmap.Dispose() }
}

function Send-RequestWithExecutingCapture([hashtable]$Connection, [string]$Method, [string]$Capability, [hashtable]$Parameters, [string]$Name) {
    $request = New-Request $Method $Capability $Parameters
    $Connection.Writer.WriteLine(($request | ConvertTo-Json -Compress -Depth 12))
    $read = $Connection.Reader.ReadLineAsync()
    Start-Sleep -Milliseconds 120
    $inFlight = -not $read.IsCompleted
    $capture = Capture-Pet $Name
    $line = $read.GetAwaiter().GetResult()
    [ordered]@{ result = Complete-Response $Connection $request $line; capture = $capture; request_in_flight_at_capture = $inFlight }
}

function Close-Session {
    if (-not $session) { return }
    $session.Writer.Dispose(); $session.Reader.Dispose(); $session.Pipe.Dispose()
    $script:session = $null
}

if (-not (Test-Path -LiteralPath $desktopExe -PathType Leaf) -or -not (Test-Path -LiteralPath $registerAgentExe -PathType Leaf)) { throw '缺少Windows桌面或Agent注册夹具' }
if (Get-Process yonder-desktop -ErrorAction SilentlyContinue) { throw '检测到现有Yonda进程，拒绝干扰' }

$backgroundColor = [Drawing.Color]::FromArgb(255, 220, 230, 242)
$backdrop = New-Object Windows.Forms.Form
$backdrop.FormBorderStyle = [Windows.Forms.FormBorderStyle]::None
$backdrop.StartPosition = [Windows.Forms.FormStartPosition]::Manual
$backdrop.Bounds = New-Object Drawing.Rectangle(520, 280, 600, 600)
$backdrop.BackColor = $backgroundColor
$backdrop.ShowInTaskbar = $false

try {
    & $registerAgentExe $applicationData $agentId
    if ($LASTEXITCODE -ne 0) { throw 'Agent登记失败' }
    $registered = $true
    $backdrop.Show(); $backdrop.Refresh(); [Windows.Forms.Application]::DoEvents()

    $start = New-Object Diagnostics.ProcessStartInfo
    $start.FileName = $desktopExe
    $start.WorkingDirectory = Split-Path -Parent $desktopExe
    $start.UseShellExecute = $false
    $start.RedirectStandardError = $true
    $ownedProcess = [Diagnostics.Process]::Start($start)
    $result.process_id = $ownedProcess.Id

    $watch = [Diagnostics.Stopwatch]::StartNew()
    $rect = $null
    while ($watch.Elapsed.TotalSeconds -lt 30 -and -not $rect) { Start-Sleep -Milliseconds 250; $rect = [YondaPresentationProbe]::FindPet($ownedProcess.Id) }
    if (-not $rect) { throw '30秒内没有找到Yonda窗口' }
    Start-Sleep -Seconds 5
    if (-not [YondaPresentationProbe]::MovePetWithoutActivation($ownedProcess.Id, 640, 400)) { throw '无法把Yonda移到受控背景' }
    Start-Sleep -Milliseconds 500

    $session = Connect-Gateway
    $hello = Send-Request $session 'gateway.hello' 'task.read' @{ protocol_version = [ordered]@{ major=1; minor=19 } }
    $browser = @($hello.capabilities | Where-Object { $_.name -eq 'browser.execute' })
    if ($browser.Count -ne 1 -or $browser[0].availability -ne 'available') { throw 'Windows browser.execute不可用' }

    $captures = [ordered]@{}
    $captures.idle_before = Capture-Pet 'presentation-idle-before'
    $created = Send-Request $session 'task.create' 'task.create' @{ idempotency_key="ds-s1-$([guid]::NewGuid().ToString('N'))"; description='验证Windows真实BUA桌宠展示'; name='DS-S1 Windows展示验证' }
    Start-Sleep -Milliseconds 100
    $captures.listening = Capture-Pet 'presentation-listening'
    $task = $created.task

    $declared = Send-Request $session 'task.step.declare' 'task.step.declare' @{ task_id=$task.task_id; expected_sequence=$task.sequence; step_id='open-space'; label='创建Browser Task Space' }
    $openedCall = Send-RequestWithExecutingCapture $session 'browser.execute' 'browser.execute' @{ task_id=$task.task_id; expected_sequence=$declared.task.sequence; operation='create' } 'presentation-executing'
    $captures.executing_create = $openedCall.capture
    $opened = $openedCall.result
    if ($opened.task.status -ne 'running' -or $opened.reference.external_task_ref -notmatch '^ego:[1-9][0-9]*$') { throw 'Browser创建未返回运行中引用' }

    $advanced = Send-Request $session 'task.step.advance' 'task.step.advance' @{ task_id=$task.task_id; expected_sequence=$opened.task.sequence }
    $declaredFinish = Send-Request $session 'task.step.declare' 'task.step.declare' @{ task_id=$task.task_id; expected_sequence=$advanced.task.sequence; step_id='finish-space'; label='完成Browser Task Space' }
    $finishCall = Send-RequestWithExecutingCapture $session 'browser.execute' 'browser.execute' @{ task_id=$task.task_id; expected_sequence=$declaredFinish.task.sequence; operation='finish' } 'presentation-executing-finish'
    $captures.executing_finish = $finishCall.capture
    $completed = $finishCall.result
    if ($completed.task.status -ne 'completed' -or -not $completed.reference.finished) { throw 'Browser finish未完成任务' }
    $finished = $true
    Start-Sleep -Milliseconds 100
    $captures.success = Capture-Pet 'presentation-success'
    Start-Sleep -Milliseconds 2000
    $captures.idle_after = Capture-Pet 'presentation-idle-after'

    $hashes = @($captures.Values | ForEach-Object { $_.sha256 } | Select-Object -Unique)
    $result.agent_id = $agentId
    $result.protocol = [ordered]@{ major=$hello.protocol_version.major; minor=$hello.protocol_version.minor }
    $result.task_id = $task.task_id
    $result.external_task_ref = $opened.reference.external_task_ref
    $result.final_status = $completed.task.status
    $result.reference_finished = $completed.reference.finished
    $result.executing_create_request_in_flight = $openedCall.request_in_flight_at_capture
    $result.executing_finish_request_in_flight = $finishCall.request_in_flight_at_capture
    $result.captures = $captures
    $result.distinct_capture_hashes = $hashes.Count
    $result.cua_executed = $false
    $result.passed = $finished -and $openedCall.request_in_flight_at_capture -and $finishCall.request_in_flight_at_capture -and $captures.Count -eq 6 -and $captures.success.sha256 -ne $captures.idle_before.sha256 -and $captures.success.sha256 -ne $captures.idle_after.sha256 -and $hashes.Count -ge 4
    if (-not $result.passed) { throw '真实BUA展示证据门禁未通过' }
} catch {
    $result.passed = $false
    $result.error = $_.Exception.Message
    throw
} finally {
    Close-Session
    if ($ownedProcess -and -not $ownedProcess.HasExited) { $ownedProcess.Kill(); $ownedProcess.WaitForExit() }
    if ($ownedProcess) { $result.diagnostic_log = @($ownedProcess.StandardError.ReadToEnd() -split "`r?`n" | Where-Object { $_ }) }
    if ($registered) { & $registerAgentExe $applicationData $agentId disable | Out-Null }
    $backdrop.Close(); $backdrop.Dispose()
    $result | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'native-bua-presentation-result.json') -Encoding UTF8
}

$result | ConvertTo-Json -Depth 10 -Compress
