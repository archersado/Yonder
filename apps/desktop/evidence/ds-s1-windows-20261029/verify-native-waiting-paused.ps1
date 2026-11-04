$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class YondaWaitingPausedProbe {
    private delegate bool EnumWindowsProc(IntPtr handle, IntPtr state);
    [DllImport("user32.dll")] private static extern bool EnumWindows(EnumWindowsProc callback, IntPtr state);
    [DllImport("user32.dll")] private static extern uint GetWindowThreadProcessId(IntPtr handle, out uint processId);
    [DllImport("user32.dll")] private static extern bool IsWindowVisible(IntPtr handle);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] private static extern int GetWindowText(IntPtr handle, StringBuilder text, int count);
    [DllImport("user32.dll")] private static extern bool GetWindowRect(IntPtr handle, out Rect rect);
    [DllImport("user32.dll")] private static extern bool SetWindowPos(IntPtr handle, IntPtr insertAfter, int x, int y, int width, int height, uint flags);
    [DllImport("user32.dll")] private static extern uint GetDpiForWindow(IntPtr handle);
    [StructLayout(LayoutKind.Sequential)] private struct Rect { public int Left, Top, Right, Bottom; }

    private static IntPtr FindHandle(int expectedProcessId) {
        IntPtr result = IntPtr.Zero;
        EnumWindows(delegate(IntPtr handle, IntPtr state) {
            uint processId;
            GetWindowThreadProcessId(handle, out processId);
            if (processId != expectedProcessId || !IsWindowVisible(handle)) return true;
            var title = new StringBuilder(128);
            GetWindowText(handle, title, title.Capacity);
            Rect rect;
            if (!title.ToString().StartsWith("Yonda") || !GetWindowRect(handle, out rect) || rect.Right - rect.Left != 200 || rect.Bottom - rect.Top != 200) return true;
            result = handle;
            return false;
        }, IntPtr.Zero);
        return result;
    }
    public static int[] Find(int expectedProcessId) {
        var handle = FindHandle(expectedProcessId);
        Rect rect;
        if (handle == IntPtr.Zero || !GetWindowRect(handle, out rect)) return null;
        return new[] { rect.Left, rect.Top, rect.Right - rect.Left, rect.Bottom - rect.Top };
    }
    public static bool Move(int expectedProcessId, int x, int y) {
        var handle = FindHandle(expectedProcessId);
        return handle != IntPtr.Zero && SetWindowPos(handle, IntPtr.Zero, x, y, 0, 0, 0x0001 | 0x0004 | 0x0010);
    }
    public static uint Dpi(int expectedProcessId) {
        var handle = FindHandle(expectedProcessId);
        return handle == IntPtr.Zero ? 0 : GetDpiForWindow(handle);
    }
}
'@

$desktopExe = 'F:\workspace\yonder\target\debug\yonder-desktop.exe'
$registerAgentExe = 'F:\workspace\yonder\target\debug\examples\register-agent.exe'
$pipeName = 'com.yonder.desktop.agent'
$requestNumber = 0
$ownedProcess = $null
$session = $null
$runRoot = $null
$result = [ordered]@{ platform='windows'; scenarios=[ordered]@{}; cua_executed=$false }

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
            return @{ Pipe=$pipe; Reader=$reader; Writer=$writer }
        } catch { $pipe.Dispose(); Start-Sleep -Milliseconds 200 }
    } while ([datetime]::UtcNow -lt $until)
    throw 'Yonder Named Pipe未在期限内就绪'
}

function Send-Request([hashtable]$Connection, [string]$AgentId, [string]$Method, [string]$Capability, [hashtable]$Parameters) {
    $script:requestNumber++
    $params = [ordered]@{ agent_id=$AgentId; capability=$Capability; deadline=Get-Deadline }
    foreach ($entry in $Parameters.GetEnumerator()) { $params[$entry.Key]=$entry.Value }
    $request = [ordered]@{ jsonrpc='2.0'; id="ds-s1-state-$script:requestNumber"; method=$Method; params=$params }
    $Connection.Writer.WriteLine(($request | ConvertTo-Json -Compress -Depth 12))
    $line = $Connection.Reader.ReadLine()
    if ([string]::IsNullOrWhiteSpace($line)) { throw "Gateway未响应：$Method" }
    $response = $line | ConvertFrom-Json
    if ($response.id -ne $request.id) { throw "Gateway响应ID不匹配：$Method" }
    if ($null -ne $response.error) { throw ("Gateway调用失败 {0}: {1}" -f $Method, ($response.error | ConvertTo-Json -Compress)) }
    $response.result
}

function Close-Session {
    if (-not $script:session) { return }
    $script:session.Writer.Dispose(); $script:session.Reader.Dispose(); $script:session.Pipe.Dispose()
    $script:session = $null
}

function Stop-OwnedProcess {
    Close-Session
    if ($script:ownedProcess -and -not $script:ownedProcess.HasExited) { $script:ownedProcess.Kill(); $script:ownedProcess.WaitForExit() }
    $script:ownedProcess = $null
}

function Capture-Pet([string]$Name) {
    $rect = [YondaWaitingPausedProbe]::Find($script:ownedProcess.Id)
    if (-not $rect) { throw "截图时Yonda窗口不可用：$Name" }
    $dpi = [YondaWaitingPausedProbe]::Dpi($script:ownedProcess.Id)
    $scale = $dpi / 96.0
    $x=[int][Math]::Round($rect[0]*$scale); $y=[int][Math]::Round($rect[1]*$scale)
    $width=[int][Math]::Round($rect[2]*$scale); $height=[int][Math]::Round($rect[3]*$scale)
    $bitmap = New-Object Drawing.Bitmap($width,$height,[Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.CopyFromScreen($x,$y,0,0,(New-Object Drawing.Size($width,$height)),[Drawing.CopyPixelOperation]::SourceCopy)
        $path = Join-Path $PSScriptRoot "$Name.png"
        $bitmap.Save($path,[Drawing.Imaging.ImageFormat]::Png)
        [ordered]@{ file="$Name.png"; sha256=(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant(); logical=[ordered]@{x=$rect[0];y=$rect[1];width=$rect[2];height=$rect[3]}; physical=[ordered]@{x=$x;y=$y;width=$width;height=$height}; dpi=$dpi }
    } finally { $graphics.Dispose(); $bitmap.Dispose() }
}

function Finish-EgoSpace([string]$ExternalRef, [string]$TemporaryRoot) {
    $spaceId = [int]($ExternalRef -replace '^ego:','')
    if ($spaceId -le 0) { throw 'ego引用无效' }
    $base = Join-Path $env:ProgramFiles 'Citro Labs\ego lite\Application'
    $ego = Get-ChildItem -LiteralPath $base -Filter 'ego-browser.exe' -Recurse -File |
        Sort-Object { $_.Directory.Name -as [version] } -Descending |
        Select-Object -First 1 -ExpandProperty FullName
    if (-not $ego) { throw 'ego-browser.exe不可用' }
    $scriptPath = Join-Path $TemporaryRoot 'finish-space.mjs'
    [IO.File]::WriteAllText($scriptPath,"const task = await taskSpace($spaceId); await task.finish({ keep: [] });",[Text.UTF8Encoding]::new($false))
    $entry = 'file:///' + ($scriptPath -replace '\\','/')
    & $ego nodejs -e "await import('$entry')"
    if ($LASTEXITCODE -ne 0) { throw "ego空间关闭失败：$ExternalRef" }
    $true
}

function Invoke-StateScenario([string]$State) {
    $agentId = "ds-s1-$State-" + [guid]::NewGuid().ToString('N').Substring(0,10)
    $script:runRoot = Join-Path ([IO.Path]::GetTempPath()) ("yonda-ds-s1-$State-" + [guid]::NewGuid().ToString('N'))
    $appData = Join-Path $script:runRoot 'Roaming'
    $localAppData = Join-Path $script:runRoot 'Local'
    $applicationData = Join-Path $appData 'com.yonder.desktop'
    New-Item -ItemType Directory -Path $applicationData,$localAppData -Force | Out-Null
    & $registerAgentExe $applicationData $agentId
    if ($LASTEXITCODE -ne 0) { throw "隔离Agent登记失败：$State" }

    $start = New-Object Diagnostics.ProcessStartInfo
    $start.FileName=$desktopExe; $start.WorkingDirectory=Split-Path -Parent $desktopExe
    $start.UseShellExecute=$false; $start.RedirectStandardError=$true
    $start.Environment['APPDATA']=$appData; $start.Environment['LOCALAPPDATA']=$localAppData
    $script:ownedProcess=[Diagnostics.Process]::Start($start)
    $watch=[Diagnostics.Stopwatch]::StartNew(); $rect=$null
    while($watch.Elapsed.TotalSeconds -lt 30 -and -not $rect){Start-Sleep -Milliseconds 250;$rect=[YondaWaitingPausedProbe]::Find($script:ownedProcess.Id)}
    if(-not $rect){throw "30秒内未找到Yonda窗口：$State"}
    Start-Sleep -Seconds 2
    if(-not [YondaWaitingPausedProbe]::Move($script:ownedProcess.Id,640,400)){throw "无法移动Yonda窗口：$State"}
    Start-Sleep -Milliseconds 500

    $script:session=Connect-Gateway
    $hello=Send-Request $script:session $agentId 'gateway.hello' 'task.read' @{protocol_version=[ordered]@{major=1;minor=19}}
    $browser=@($hello.capabilities|Where-Object{$_.name -eq 'browser.execute'})
    if($browser.Count -ne 1 -or $browser[0].availability -ne 'available'){throw 'browser.execute不可用'}
    $created=Send-Request $script:session $agentId 'task.create' 'task.create' @{idempotency_key="ds-s1-$State-$([guid]::NewGuid().ToString('N'))";description="验证Windows真实$State展示";name="DS-S1 Windows $State"}
    $task=$created.task
    $declared=Send-Request $script:session $agentId 'task.step.declare' 'task.step.declare' @{task_id=$task.task_id;expected_sequence=$task.sequence;step_id='open-space';label='创建Browser Task Space'}
    $opened=Send-Request $script:session $agentId 'browser.execute' 'browser.execute' @{task_id=$task.task_id;expected_sequence=$declared.task.sequence;operation='create'}
    if($opened.task.status -ne 'running' -or $opened.reference.external_task_ref -notmatch '^ego:[1-9][0-9]*$'){throw "Browser空间创建失败：$State"}

    if($State -eq 'paused'){
        $stateResult=Send-Request $script:session $agentId 'task.control' 'task.control' @{task_id=$task.task_id;expected_sequence=$opened.task.sequence;kind='pause'}
        if($stateResult.task.status -ne 'paused' -or $stateResult.control.phase -ne 'stopped'){throw 'Pause未在已Observe边界停止'}
    } else {
        $advanced=Send-Request $script:session $agentId 'task.step.advance' 'task.step.advance' @{task_id=$task.task_id;expected_sequence=$opened.task.sequence}
        $stateResult=Send-Request $script:session $agentId 'task.wait_for_user' 'task.wait-for-user' @{task_id=$task.task_id;expected_sequence=$advanced.task.sequence;reason='请确认验证夹具状态'}
        if($stateResult.task.status -ne 'waiting-for-user'){throw 'WaitForUser未进入等待状态'}
    }
    Start-Sleep -Milliseconds 180
    $capture=Capture-Pet "presentation-$State"
    $externalFinished=Finish-EgoSpace $opened.reference.external_task_ref $script:runRoot
    [ordered]@{agent_id=$agentId;task_id=$task.task_id;task_status=$stateResult.task.status;external_task_ref=$opened.reference.external_task_ref;external_space_finished=$externalFinished;capture=$capture;isolated_app_data=$true}
}

if(-not(Test-Path -LiteralPath $desktopExe -PathType Leaf)-or-not(Test-Path -LiteralPath $registerAgentExe -PathType Leaf)){throw '缺少Windows桌面或Agent注册夹具'}
if(Get-Process yonder-desktop -ErrorAction SilentlyContinue){throw '检测到现有Yonda进程，拒绝干扰'}
$backdrop=New-Object Windows.Forms.Form
$backdrop.FormBorderStyle=[Windows.Forms.FormBorderStyle]::None; $backdrop.StartPosition=[Windows.Forms.FormStartPosition]::Manual
$backdrop.Bounds=New-Object Drawing.Rectangle(520,280,600,600); $backdrop.BackColor=[Drawing.Color]::FromArgb(255,220,230,242); $backdrop.ShowInTaskbar=$false

try {
    $backdrop.Show();$backdrop.Refresh();[Windows.Forms.Application]::DoEvents()
    foreach($state in @('paused','waiting_for_user')){
        try { $result.scenarios[$state]=Invoke-StateScenario $state }
        finally {
            Stop-OwnedProcess
            if($script:runRoot -and (Test-Path -LiteralPath $script:runRoot)){Remove-Item -LiteralPath $script:runRoot -Recurse -Force}
            $script:runRoot=$null
            Start-Sleep -Milliseconds 400
        }
    }
    $hashes=@($result.scenarios.Values|ForEach-Object{$_.capture.sha256}|Select-Object -Unique)
    $result.distinct_capture_hashes=$hashes.Count
    $result.isolated_data_removed=$true
    $result.passed=$result.scenarios.paused.task_status -eq 'paused' -and $result.scenarios.waiting_for_user.task_status -eq 'waiting-for-user' -and $result.scenarios.paused.external_space_finished -and $result.scenarios.waiting_for_user.external_space_finished -and $hashes.Count -eq 2
    if(-not $result.passed){throw 'waiting/paused原生展示门禁未通过'}
} catch {
    $result.passed=$false; $result.error=$_.Exception.Message
    throw
} finally {
    Stop-OwnedProcess
    if($script:runRoot -and (Test-Path -LiteralPath $script:runRoot)){Remove-Item -LiteralPath $script:runRoot -Recurse -Force}
    $backdrop.Close();$backdrop.Dispose()
    $result|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $PSScriptRoot 'native-waiting-paused-result.json') -Encoding UTF8
}

$result|ConvertTo-Json -Depth 10 -Compress
