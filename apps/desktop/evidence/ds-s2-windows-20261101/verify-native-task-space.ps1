$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class YondaTaskSpaceProbe {
    [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint x, uint y, uint data, UIntPtr extra);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindow(string className, string windowName);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr handle, out Rect rect);
    public static int[] MenuRect() {
        var handle = FindWindow("#32768", null); Rect rect;
        if (handle == IntPtr.Zero || !GetWindowRect(handle, out rect)) return null;
        return new[] { rect.Left, rect.Top, rect.Right - rect.Left, rect.Bottom - rect.Top };
    }
}
'@

$desktopExe = 'F:\workspace\yonder\target\debug\yonder-desktop.exe'
$registerAgentExe = 'F:\workspace\yonder\target\debug\examples\register-agent.exe'
$pipeName = 'com.yonder.desktop.agent'
$agentId = 'ds-s2-win-' + [guid]::NewGuid().ToString('N').Substring(0, 12)
$applicationData = Join-Path $env:APPDATA 'com.yonder.desktop'
$resultPath = Join-Path $PSScriptRoot 'native-task-space-result.json'
$requestNumber = 0
$paginationScan = [Collections.Generic.List[object]]::new()
$ownedProcess = $null
$session = $null
$registered = $false
$browserTask = $null
$uiCancelTask = $null
$browserFinished = $false
$forcedCleanup = $false
$createdTasks = @()
$result = [ordered]@{ platform='windows'; cua_executed=$false; fixture='real-gateway-and-ego-lite-with-dedicated-agent' }

function Get-Deadline { [long](([datetime]::UtcNow - [datetime]'1970-01-01').TotalMilliseconds + 60000) }

function Connect-Gateway {
    $until = [datetime]::UtcNow.AddSeconds(30)
    do {
        $pipe = [IO.Pipes.NamedPipeClientStream]::new('.', $pipeName, [IO.Pipes.PipeDirection]::InOut, [IO.Pipes.PipeOptions]::Asynchronous)
        try {
            $pipe.Connect(500)
            $encoding = [Text.UTF8Encoding]::new($false)
            $reader = [IO.StreamReader]::new($pipe, $encoding, $false, 65536, $true)
            $writer = [IO.StreamWriter]::new($pipe, $encoding, 65536, $true); $writer.AutoFlush = $true
            return @{Pipe=$pipe;Reader=$reader;Writer=$writer}
        } catch { $pipe.Dispose(); Start-Sleep -Milliseconds 200 }
    } while ([datetime]::UtcNow -lt $until)
    throw 'Yonder Named Pipe未在期限内就绪'
}

function Send-Request([string]$Method, [string]$Capability, [hashtable]$Parameters) {
    $script:requestNumber++
    $params = [ordered]@{agent_id=$agentId;capability=$Capability;deadline=Get-Deadline}
    foreach ($entry in $Parameters.GetEnumerator()) { $params[$entry.Key] = $entry.Value }
    $request = [ordered]@{jsonrpc='2.0';id="ds-s2-win-$script:requestNumber";method=$Method;params=$params}
    $session.Writer.WriteLine(($request | ConvertTo-Json -Compress -Depth 12))
    $line = $session.Reader.ReadLine()
    if ([string]::IsNullOrWhiteSpace($line)) { throw "Gateway未响应：$Method" }
    $response = $line | ConvertFrom-Json
    if ($response.id -ne $request.id) { throw "Gateway响应ID不匹配：$Method" }
    if ($null -ne $response.error) { throw ("Gateway调用失败 {0}: {1}" -f $Method, ($response.error | ConvertTo-Json -Compress)) }
    return $response.result
}

function Close-Session {
    if (-not $session) { return }
    $session.Writer.Dispose(); $session.Reader.Dispose(); $session.Pipe.Dispose(); $script:session = $null
}

function Find-Element([string]$Name, [string]$ClassName = '') {
    $all = [Windows.Automation.AutomationElement]::RootElement.FindAll([Windows.Automation.TreeScope]::Descendants, [Windows.Automation.Condition]::TrueCondition)
    foreach ($element in $all) {
        try {
            if ($element.Current.Name -eq $Name -and (!$ClassName -or $element.Current.ClassName -eq $ClassName)) { return $element }
        } catch {}
    }
    return $null
}

function Find-Descendant([Windows.Automation.AutomationElement]$Root, [string]$Name, [switch]$Contains) {
    $all = $Root.FindAll([Windows.Automation.TreeScope]::Descendants, [Windows.Automation.Condition]::TrueCondition)
    foreach ($element in $all) {
        try {
            if (($Contains -and $element.Current.Name.Contains($Name)) -or (!$Contains -and $element.Current.Name -eq $Name)) { return $element }
        } catch {}
    }
    return $null
}

function Wait-Element([Windows.Automation.AutomationElement]$Root, [string]$Name, [switch]$Contains, [int]$Seconds = 15) {
    $until = [datetime]::UtcNow.AddSeconds($Seconds)
    do {
        $found = Find-Descendant $Root $Name -Contains:$Contains
        if ($found) { return $found }
        Start-Sleep -Milliseconds 150
    } while ([datetime]::UtcNow -lt $until)
    throw "未找到原生UI元素：$Name"
}

function Invoke-Element([Windows.Automation.AutomationElement]$Element) {
    if ($Element.Current.IsOffscreen) {
        try { $Element.GetCurrentPattern([Windows.Automation.ScrollItemPattern]::Pattern).ScrollIntoView(); Start-Sleep -Milliseconds 300 } catch {}
    }
    $rect=$Element.Current.BoundingRectangle
    if (-not $Element.Current.IsOffscreen -and $rect.Width -gt 0 -and $rect.Height -gt 0 -and -not [double]::IsNaN($rect.Left) -and -not [double]::IsNaN($rect.Top)) {
        [void][YondaTaskSpaceProbe]::SetCursorPos([int]($rect.Left+$rect.Width/2),[int]($rect.Top+$rect.Height/2))
        [YondaTaskSpaceProbe]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero)
        [YondaTaskSpaceProbe]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
    } else {
        $Element.GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern).Invoke()
    }
    Start-Sleep -Milliseconds 700
}

function Find-YondaTray {
    $icon = Find-Element 'Yonda' 'SystemTray.NormalButton'
    if ($icon) {
        $rect = $icon.Current.BoundingRectangle
        if (-not $icon.Current.IsOffscreen -and $rect.Width -gt 0 -and $rect.Height -gt 0 -and -not [double]::IsNaN($rect.Left) -and -not [double]::IsNaN($rect.Top)) { return $icon }
    }
    $hidden = Find-Element '显示隐藏的图标'
    if (-not $hidden) { throw 'Windows隐藏图标入口不可用' }
    Invoke-Element $hidden
    $icon = Find-Element 'Yonda' 'SystemTray.NormalButton'
    if (-not $icon -or $icon.Current.IsOffscreen) { throw '隐藏托盘区未找到可见Yonda图标' }
    $rect = $icon.Current.BoundingRectangle
    if ($rect.Width -le 0 -or $rect.Height -le 0 -or [double]::IsNaN($rect.Left) -or [double]::IsNaN($rect.Top)) { throw 'Yonda托盘图标没有有效原生坐标' }
    return $icon
}

function Open-Task-Space {
    $pet=Find-Element '轻点 Yonda 小龙，按住移动可拖动'
    if ($pet) {
        try {
            $pet.SetFocus()
            [Windows.Forms.SendKeys]::SendWait('{ENTER}')
            $until=[datetime]::UtcNow.AddSeconds(8)
            do {
                Start-Sleep -Milliseconds 200
                $window=Find-Element 'Yonda · Task Space'
                if ($window) { return $window }
            } while ([datetime]::UtcNow -lt $until)
        } catch {}
    }
    $icon = Find-YondaTray
    $rect = $icon.Current.BoundingRectangle
    [void][YondaTaskSpaceProbe]::SetCursorPos([int]($rect.Left + $rect.Width / 2), [int]($rect.Top + $rect.Height / 2))
    [YondaTaskSpaceProbe]::mouse_event(0x0008,0,0,0,[UIntPtr]::Zero); [YondaTaskSpaceProbe]::mouse_event(0x0010,0,0,0,[UIntPtr]::Zero)
    Start-Sleep -Milliseconds 500
    $menu = [YondaTaskSpaceProbe]::MenuRect()
    if (-not $menu) { throw 'Yonda原生托盘菜单未打开' }
    $x = [int]($menu[0] + $menu[2] / 2); $y = [int]($menu[1] + ($menu[3] / 7.0) * 1.5)
    [void][YondaTaskSpaceProbe]::SetCursorPos($x,$y)
    [YondaTaskSpaceProbe]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero); [YondaTaskSpaceProbe]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
    $until = [datetime]::UtcNow.AddSeconds(15)
    do {
        Start-Sleep -Milliseconds 200
        $window = Find-Element 'Yonda · Task Space'
        if ($window) { return $window }
    } while ([datetime]::UtcNow -lt $until)
    throw '任务总览窗口未打开'
}

function Capture-Window([Windows.Automation.AutomationElement]$Window, [string]$FileName) {
    $path = Join-Path $PSScriptRoot $FileName; $rect = $Window.Current.BoundingRectangle
    $width = [int][Math]::Round($rect.Width); $height = [int][Math]::Round($rect.Height)
    $bitmap = New-Object Drawing.Bitmap($width,$height,[Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    try { $graphics.CopyFromScreen([int]$rect.Left,[int]$rect.Top,0,0,(New-Object Drawing.Size($width,$height))); $bitmap.Save($path,[Drawing.Imaging.ImageFormat]::Png) }
    finally { $graphics.Dispose(); $bitmap.Dispose() }
    return [ordered]@{file=$FileName;width=$width;height=$height;sha256=(Get-FileHash $path -Algorithm SHA256).Hash.ToLowerInvariant()}
}

function Snapshot-Tree([Windows.Automation.AutomationElement]$Window) {
    $values = [Collections.Generic.List[object]]::new()
    $all = $Window.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.Condition]::TrueCondition)
    foreach ($element in $all) {
        try {
            if ($element.Current.Name) { $values.Add([ordered]@{name=$element.Current.Name;type=$element.Current.ControlType.ProgrammaticName;enabled=$element.Current.IsEnabled}) }
        } catch {}
    }
    return @($values)
}

function Find-Task-Button([Windows.Automation.AutomationElement]$Window, [string]$TaskName) {
    return Wait-Element $Window $TaskName -Contains
}

function Find-Nearest-Button([Windows.Automation.AutomationElement]$Window, [Windows.Automation.AutomationElement]$Reference, [string]$Name) {
    $referenceRect = $Reference.Current.BoundingRectangle; $best = $null; $distance = [double]::MaxValue
    $all = $Window.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.Condition]::TrueCondition)
    foreach ($element in $all) {
        try {
            if ($element.Current.Name -eq $Name -and $element.Current.ControlType -eq [Windows.Automation.ControlType]::Button) {
                $rect = $element.Current.BoundingRectangle
                $candidate = [Math]::Abs(($rect.Top + $rect.Height / 2) - ($referenceRect.Top + $referenceRect.Height / 2))
                if ($candidate -lt $distance) { $distance = $candidate; $best = $element }
            }
        } catch {}
    }
    if (-not $best -or $distance -gt 80) { throw "未找到任务同行按钮：$Name" }
    return $best
}

function Go-To-Second-Page([Windows.Automation.AutomationElement]$Window) {
    $next = Wait-Element $Window '下一页'
    Invoke-Element $next
    [void](Wait-Element $Window '第 2 页' -Contains)
}

function Go-To-Task([Windows.Automation.AutomationElement]$Window, [string]$TaskName, [int]$PageNumber) {
    while ($PageNumber -le 50) {
        $seen=@($Window.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.Condition]::TrueCondition) | ForEach-Object { try { $_.Current.Name } catch {} } | Where-Object { $_ -like 'DS2W-*' })
        $script:paginationScan.Add([ordered]@{page=$PageNumber;target=$TaskName;seen=$seen})
        $card=Find-Descendant $Window $TaskName -Contains
        if ($card) {
            if ($card.Current.IsOffscreen) { try { $card.GetCurrentPattern([Windows.Automation.ScrollItemPattern]::Pattern).ScrollIntoView(); Start-Sleep -Milliseconds 300 } catch {} }
            return [ordered]@{card=$card;page=$PageNumber}
        }
        $next=Find-Descendant $Window '下一页'
        if (-not $next) { throw "遍历到末页仍未找到任务：$TaskName" }
        Invoke-Element $next
        $PageNumber++
        [void](Wait-Element $Window "第 $PageNumber 页" -Contains)
    }
    throw "任务分页超过安全上限：$TaskName"
}

function Create-Task([string]$Index) {
    $tag=$agentId.Substring('ds-s2-win-'.Length)
    return Send-Request 'task.create' 'task.create' @{idempotency_key="ds-s2-$Index-$([guid]::NewGuid().ToString('N'))";description='Windows任务总览过滤、分页与时间线验证';name="DS2W-$tag-$Index"}
}

function Declare-Step([object]$Task, [string]$Id, [string]$Label) {
    return Send-Request 'task.step.declare' 'task.step.declare' @{task_id=$Task.task_id;expected_sequence=$Task.sequence;step_id=$Id;label=$Label}
}

function Execute-Browser([object]$Task, [string]$Operation) {
    return Send-Request 'browser.execute' 'browser.execute' @{task_id=$Task.task_id;expected_sequence=$Task.sequence;operation=$Operation}
}

function Advance-Step([object]$Task) {
    return Send-Request 'task.step.advance' 'task.step.advance' @{task_id=$Task.task_id;expected_sequence=$Task.sequence}
}

if (-not (Test-Path -LiteralPath $desktopExe -PathType Leaf) -or -not (Test-Path -LiteralPath $registerAgentExe -PathType Leaf)) { throw '缺少Windows桌面或Agent注册夹具' }
if (Get-Process yonder-desktop -ErrorAction SilentlyContinue) { throw '检测到现有Yonda进程，拒绝干扰' }

New-Item -ItemType Directory -Path $applicationData -Force | Out-Null
try {
    & $registerAgentExe $applicationData $agentId
    if ($LASTEXITCODE -ne 0) { throw 'Agent登记失败' }
    $registered = $true

    $start = New-Object Diagnostics.ProcessStartInfo
    $start.FileName=$desktopExe; $start.WorkingDirectory=(Split-Path -Parent $desktopExe); $start.UseShellExecute=$false; $start.RedirectStandardError=$true
    $ownedProcess=[Diagnostics.Process]::Start($start); $result.process_id=$ownedProcess.Id
    $session=Connect-Gateway
    $hello=Send-Request 'gateway.hello' 'task.read' @{protocol_version=[ordered]@{major=1;minor=19}}
    $result.protocol=[ordered]@{major=$hello.protocol_version.major;minor=$hello.protocol_version.minor}; $result.agent_id=$agentId

    $result.stage='create-pagination-fixture'
    $created = [Collections.Generic.List[object]]::new()
    1..22 | ForEach-Object { $created.Add((Create-Task ('{0:d2}' -f $_)).task) }
    $createdTasks=@($created)
    $ordered = @($created | Sort-Object task_id)
    $browserTask = $ordered[-1]
    $uiCancelTask = $ordered[-2]
    $result.browser_task_id=$browserTask.task_id; $result.ui_cancel_task_id=$uiCancelTask.task_id
    $result.preceding_non_running_count=$ordered.Count-2
    foreach ($task in $ordered[0..($ordered.Count-3)]) {
        $cancelled=Send-Request 'task.cancel' 'task.cancel' @{task_id=$task.task_id;expected_sequence=$task.sequence}
        if ($cancelled.task.status -ne 'cancelled') { throw '分页前置任务取消失败' }
    }

    $result.stage='browser-create'
    $declared=Declare-Step $browserTask 'create-space' '创建Browser Task Space'; $opened=Execute-Browser $declared.task 'create'; $browserTask=$opened.task
    if ($browserTask.status -ne 'running') { throw 'BUA任务未进入running' }
    1..6 | ForEach-Object {
        $result.stage="browser-observe-$_"
        $advanced=Advance-Step $browserTask
        $declared=Declare-Step $advanced.task "observe-$_" "第$_轮观察Browser Task Space"
        $observed=Execute-Browser $declared.task 'observe'
        $browserTask=$observed.task
    }
    if ([long]$browserTask.sequence -le 20) { throw '真实BUA事件不足20条' }
    $result.running_sequence_before_ui=$browserTask.sequence

    $runningPage=Send-Request 'task.list' 'task.read' @{after_task_id=$null;include_finished=$false;running_only=$true;limit=20}
    $allPage=Send-Request 'task.list' 'task.read' @{after_task_id=$null;include_finished=$true;running_only=$false;limit=20}
    $result.gateway_filter=[ordered]@{running_first_page=@($runningPage.tasks).Count;running_task_id=$runningPage.tasks[0].task_id;all_first_page=@($allPage.tasks).Count;all_has_next=[bool]$allPage.next_after_task_id;running_absent_from_all_first_page=(-not (@($allPage.tasks.task_id) -contains $browserTask.task_id))}
    if (@($runningPage.tasks).Count -ne 1 -or $runningPage.tasks[0].task_id -ne $browserTask.task_id -or -not $result.gateway_filter.running_absent_from_all_first_page) { throw 'Gateway先过滤后分页断言失败' }

    Start-Sleep -Seconds 2
    $result.stage='open-native-task-space'
    $window=Open-Task-Space
    [void](Wait-Element $window '第 1 页 · 1 项')
    [void](Wait-Element $window $browserTask.name -Contains)
    $result.native_running=Capture-Window $window 'native-running.png'

    Invoke-Element (Wait-Element $window '全部')
    [void](Wait-Element $window '第 1 页 · 20 项')
    [void](Wait-Element $window '下一页')
    $result.native_all_page1=Capture-Window $window 'native-all-page1.png'
    Go-To-Second-Page $window
    $result.native_all_page2=Capture-Window $window 'native-all-page2.png'

    $located=Go-To-Task $window $uiCancelTask.name 2
    $cancelCard=$located.card
    $cancelButton=Find-Nearest-Button $window $cancelCard '取消任务'
    $result.stage='native-ui-cancel'
    Invoke-Element $cancelButton
    [void](Wait-Element $window '任务已取消，数据和历史已保留')
    $located=Go-To-Task $window $uiCancelTask.name 1
    $cancelCard=$located.card; Invoke-Element $cancelCard
    $result.stage='native-cancel-readback'
    [void](Wait-Element $window '已取消')
    [void](Wait-Element $window '#2')
    $result.native_cancel_retained=Capture-Window $window 'native-cancel-retained.png'

    $located=Go-To-Task $window $browserTask.name $located.page
    $runningCard=$located.card; Invoke-Element $runningCard
    $result.stage='native-timeline-load-more'
    $more=Wait-Element $window '加载更多时间线'
    $beforeTree=@(Snapshot-Tree $window | Where-Object { $_.name -match '^#[0-9]+$' }).Count
    Invoke-Element $more
    $until=[datetime]::UtcNow.AddSeconds(15)
    do { Start-Sleep -Milliseconds 150; $afterTree=@(Snapshot-Tree $window | Where-Object { $_.name -match '^#[0-9]+$' }).Count } while ($afterTree -le $beforeTree -and [datetime]::UtcNow -lt $until)
    if ($afterTree -le $beforeTree) { throw '原生时间线追加后事件数量未增加' }
    $result.timeline_ui=[ordered]@{before=$beforeTree;after=$afterTree;load_more_removed=(-not [bool](Find-Descendant $window '加载更多时间线'))}
    $result.native_timeline_more=Capture-Window $window 'native-timeline-more.png'
    $result.accessibility_tree=Snapshot-Tree $window

    $uiCancelled=Send-Request 'task.get' 'task.read' @{task_id=$uiCancelTask.task_id}
    $uiEvents=Send-Request 'task.events' 'task.read' @{task_id=$uiCancelTask.task_id;after_sequence='0';limit=20}
    if ($uiCancelled.task.status -ne 'cancelled' -or @($uiEvents.events).Count -ne 2) { throw 'UI取消后的任务或历史未保留' }
    $result.cancel_readback=[ordered]@{status=$uiCancelled.task.status;sequence=$uiCancelled.task.sequence;events=@($uiEvents.events).Count}

    $advanced=Advance-Step $browserTask; $declared=Declare-Step $advanced.task 'finish-space' '完成Browser Task Space'; $finished=Execute-Browser $declared.task 'finish'; $browserTask=$finished.task
    if ($browserTask.status -ne 'completed' -or -not $finished.reference.finished) { throw 'BUA任务未正常完成' }
    $browserFinished=$true
    $allEvents=Send-Request 'task.events' 'task.read' @{task_id=$browserTask.task_id;after_sequence='0';limit=100}
    $result.browser_final=[ordered]@{status=$browserTask.status;sequence=$browserTask.sequence;events=@($allEvents.events).Count;external_task_ref=$finished.reference.external_task_ref;finished=$finished.reference.finished}
    $result.passed=$true
} catch {
    $result.passed=$false; $result.error=$_.Exception.Message
    throw
} finally {
    $cleanup=[Collections.Generic.List[object]]::new()
    if ($session) {
        if ($browserTask -and -not $browserFinished) {
            try {
                $snapshot=Send-Request 'task.get' 'task.read' @{task_id=$browserTask.task_id}
                if ($snapshot.task.status -eq 'running') {
                    $advanced=Advance-Step $snapshot.task
                    $declared=Declare-Step $advanced.task 'cleanup-finish-space' '异常路径完成Browser Task Space'
                    $finished=Execute-Browser $declared.task 'finish'
                    $browserFinished=$finished.task.status -eq 'completed' -and $finished.reference.finished
                    $cleanup.Add([ordered]@{task_id=$browserTask.task_id;status=$finished.task.status;browser_finished=$finished.reference.finished})
                }
            } catch { $cleanup.Add([ordered]@{task_id=$browserTask.task_id;error=$_.Exception.Message}) }
        }
        foreach ($task in $createdTasks) {
            try {
                $snapshot=Send-Request 'task.get' 'task.read' @{task_id=$task.task_id}
                if ($snapshot.task.status -eq 'created') {
                    $cancelled=Send-Request 'task.cancel' 'task.cancel' @{task_id=$task.task_id;expected_sequence=$snapshot.task.sequence}
                    $cleanup.Add([ordered]@{task_id=$task.task_id;status=$cancelled.task.status})
                }
            } catch { $cleanup.Add([ordered]@{task_id=$task.task_id;error=$_.Exception.Message}) }
        }
    }
    $result.finally_created_cleanup=@($cleanup)
    $result.pagination_scan=@($paginationScan)
    Close-Session
    if ($ownedProcess -and -not $ownedProcess.HasExited) { $forcedCleanup=$true; $ownedProcess.Kill(); $ownedProcess.WaitForExit() }
    if ($registered -and (Test-Path -LiteralPath $registerAgentExe -PathType Leaf) -and (Test-Path -LiteralPath $applicationData)) { & $registerAgentExe $applicationData $agentId disable | Out-Null }
    if ($ownedProcess) { $result.diagnostic_log=@($ownedProcess.StandardError.ReadToEnd() -split "`r?`n" | Where-Object { $_ }) }
    $result.browser_finished=$browserFinished; $result.forced_process_cleanup=$forcedCleanup
    $result | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $resultPath -Encoding UTF8
}

$result | ConvertTo-Json -Depth 12 -Compress
