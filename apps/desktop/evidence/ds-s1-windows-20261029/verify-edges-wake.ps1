param(
    [int]$InteractionDelaySeconds = 20,
    [int]$PreDockCheckSeconds = 170,
    [int]$DockTimeoutSeconds = 190
)

$ErrorActionPreference = 'Stop'

Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;

public static class YondaNativeProbe {
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
    private static extern bool SetForegroundWindow(IntPtr handle);
    [DllImport("user32.dll")]
    private static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")]
    private static extern IntPtr WindowFromPoint(Point point);
    [DllImport("user32.dll")]
    private static extern IntPtr GetAncestor(IntPtr handle, uint flags);
    [DllImport("user32.dll")]
    private static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extraInfo);
    [DllImport("user32.dll")]
    private static extern void keybd_event(byte virtualKey, byte scanCode, uint flags, UIntPtr extraInfo);
    [DllImport("user32.dll")]
    private static extern bool SystemParametersInfo(uint action, uint parameter, out Rect value, uint flags);

    [StructLayout(LayoutKind.Sequential)]
    private struct Rect { public int Left, Top, Right, Bottom; }
    [StructLayout(LayoutKind.Sequential)]
    private struct Point { public int X, Y; }

    private static IntPtr FindPetHandle(int expectedProcessId) {
        IntPtr result = IntPtr.Zero;
        EnumWindows(delegate(IntPtr handle, IntPtr state) {
            uint processId;
            GetWindowThreadProcessId(handle, out processId);
            if (processId != expectedProcessId || !IsWindowVisible(handle)) return true;
            var title = new StringBuilder(128);
            GetWindowText(handle, title, title.Capacity);
            if (title.ToString() != "Yonda") return true;
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

    public static int[] PrimaryWorkArea() {
        Rect rect;
        if (!SystemParametersInfo(0x0030, 0, out rect, 0)) return null;
        return new[] { rect.Left, rect.Top, rect.Right, rect.Bottom };
    }

    public static bool MovePetWithoutActivation(int expectedProcessId, int x, int y) {
        var handle = FindPetHandle(expectedProcessId);
        return handle != IntPtr.Zero && SetWindowPos(handle, IntPtr.Zero, x, y, 0, 0, 0x0001 | 0x0004 | 0x0010);
    }

    public static bool ClickPet(int expectedProcessId, double xRatio, double yRatio) {
        var handle = FindPetHandle(expectedProcessId);
        Rect rect;
        if (handle == IntPtr.Zero || !GetWindowRect(handle, out rect)) return false;
        SetForegroundWindow(handle);
        // user32 对 DPI-unaware 的 PowerShell 同时虚拟化矩形和光标坐标；两者
        // 必须保持同一坐标系。WindowFromPoint 再确认命中本测试窗口而非桌面。
        int x = (int)Math.Round(rect.Left + (rect.Right - rect.Left) * xRatio);
        int y = (int)Math.Round(rect.Top + (rect.Bottom - rect.Top) * yRatio);
        if (!SetCursorPos(x, y)) return false;
        Thread.Sleep(80);
        var hit = WindowFromPoint(new Point { X = x, Y = y });
        if (hit == IntPtr.Zero || GetAncestor(hit, 2) != handle) return false;
        mouse_event(0x0002, 0, 0, 0, UIntPtr.Zero);
        Thread.Sleep(80);
        mouse_event(0x0004, 0, 0, 0, UIntPtr.Zero);
        return true;
    }

    public static bool PressEnter(int expectedProcessId) {
        var handle = FindPetHandle(expectedProcessId);
        if (handle == IntPtr.Zero || !SetForegroundWindow(handle)) return false;
        Thread.Sleep(150);
        keybd_event(0x0D, 0, 0, UIntPtr.Zero);
        Thread.Sleep(80);
        keybd_event(0x0D, 0, 0x0002, UIntPtr.Zero);
        return true;
    }
}
'@

function Convert-Rect([int[]]$Rect) {
    if (-not $Rect) { return $null }
    [ordered]@{ x = $Rect[0]; y = $Rect[1]; width = $Rect[2]; height = $Rect[3] }
}

function Test-Expanded([int[]]$Rect) {
    $Rect -and $Rect[2] -eq 200 -and $Rect[3] -eq 200
}

function Test-Docked([int[]]$Rect, [string]$Edge, [int[]]$WorkArea) {
    if (-not $Rect) { return $false }
    switch ($Edge) {
        'top' { return $Rect[2] -eq 112 -and $Rect[3] -eq 56 -and [Math]::Abs($Rect[1] - $WorkArea[1]) -le 2 }
        'right' { return $Rect[2] -eq 56 -and $Rect[3] -eq 112 -and [Math]::Abs(($Rect[0] + $Rect[2]) - $WorkArea[2]) -le 2 }
        default { throw "不支持的边缘：$Edge" }
    }
}

function Wait-PetRect([Diagnostics.Process]$Process, [scriptblock]$Predicate, [double]$TimeoutSeconds) {
    $watch = [Diagnostics.Stopwatch]::StartNew()
    do {
        if ($Process.HasExited) { throw "Yonda提前退出，code=$($Process.ExitCode)" }
        $rect = [YondaNativeProbe]::FindPet($Process.Id)
        if (& $Predicate $rect) {
            return [ordered]@{ rect = $rect; elapsed_seconds = [Math]::Round($watch.Elapsed.TotalSeconds, 3) }
        }
        Start-Sleep -Milliseconds 250
    } while ($watch.Elapsed.TotalSeconds -lt $TimeoutSeconds)
    return $null
}

$executable = 'F:\workspace\yonder\target\debug\yonder-desktop.exe'
if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) { throw 'Windows desktop executable not found' }
$existing = @(Get-CimInstance Win32_Process -Filter "Name='yonder-desktop.exe'" | Where-Object { $_.ExecutablePath -eq $executable })
if ($existing.Count -gt 0) { throw "检测到现有Yonda进程，拒绝干扰：$($existing.ProcessId -join ',')" }

$testRoot = Join-Path $env:TEMP ('yonda-ds-s1-edges-' + [guid]::NewGuid().ToString('N'))
$roaming = Join-Path $testRoot 'Roaming'
$local = Join-Path $testRoot 'Local'
New-Item -ItemType Directory -Path $roaming, $local | Out-Null

$process = $null
$result = [ordered]@{
    platform = 'windows'
    executable = $executable
    interaction_delay_seconds = $InteractionDelaySeconds
    pre_dock_check_seconds = $PreDockCheckSeconds
    dock_timeout_seconds = $DockTimeoutSeconds
}
try {
    $start = New-Object Diagnostics.ProcessStartInfo
    $start.FileName = $executable
    $start.WorkingDirectory = Split-Path -Parent $executable
    $start.UseShellExecute = $false
    $start.RedirectStandardError = $true
    $start.EnvironmentVariables['APPDATA'] = $roaming
    $start.EnvironmentVariables['LOCALAPPDATA'] = $local
    $process = [Diagnostics.Process]::Start($start)
    $result.process_id = $process.Id

    $opened = Wait-PetRect $process { param($rect) Test-Expanded $rect } 30
    if (-not $opened) { throw '30秒内没有找到200×200的Yonda原生窗口' }
    Start-Sleep -Seconds 5
    $workArea = [YondaNativeProbe]::PrimaryWorkArea()
    if (-not $workArea) { throw '无法读取Windows主屏工作区' }
    $result.work_area = [ordered]@{ left = $workArea[0]; top = $workArea[1]; right = $workArea[2]; bottom = $workArea[3] }

    $topX = $workArea[0] + [Math]::Max(0, [int](($workArea[2] - $workArea[0] - 200) / 2))
    if (-not [YondaNativeProbe]::MovePetWithoutActivation($process.Id, $topX, $workArea[1])) { throw '无法把测试窗口移到顶部' }
    Start-Sleep -Seconds $InteractionDelaySeconds
    if (-not [YondaNativeProbe]::ClickPet($process.Id, 0.5, 0.5)) { throw '清醒态点击坐标未命中Yonda窗口' }
    Write-Output 'PROGRESS interaction-hit'
    $interactionAt = Get-Date
    Start-Sleep -Seconds $PreDockCheckSeconds
    $preDockRect = [YondaNativeProbe]::FindPet($process.Id)
    $result.interaction_reset = [ordered]@{
        interaction = 'native primary click'
        checked_after_seconds = [Math]::Round(((Get-Date) - $interactionAt).TotalSeconds, 3)
        rect = Convert-Rect $preDockRect
        remained_expanded = Test-Expanded $preDockRect
    }
    if (-not $result.interaction_reset.remained_expanded) { throw '交互后未满180秒窗口已提前停靠' }

    $topRemaining = [Math]::Max(1, $DockTimeoutSeconds - $PreDockCheckSeconds)
    $top = Wait-PetRect $process { param($rect) Test-Docked $rect 'top' $workArea } $topRemaining
    if (-not $top) { throw "交互后$DockTimeoutSeconds秒内未进入顶部停靠" }
    Write-Output 'PROGRESS top-docked'
    $result.top_dock = [ordered]@{
        rect = Convert-Rect $top.rect
        elapsed_since_interaction_seconds = [Math]::Round(((Get-Date) - $interactionAt).TotalSeconds, 3)
        passed = $true
    }

    # 顶边图像旋转180度，眼睛位于窗口上部；点击可见眼睛而非透明中心。
    if (-not [YondaNativeProbe]::ClickPet($process.Id, 0.5, 0.18)) { throw '顶部探头点击坐标未命中Yonda窗口' }
    $clickWake = Wait-PetRect $process { param($rect) Test-Expanded $rect } 5
    if (-not $clickWake) { throw '顶部停靠后鼠标点击未恢复200×200窗口' }
    Write-Output 'PROGRESS click-woke'
    $result.click_wake = [ordered]@{
        rect = Convert-Rect $clickWake.rect
        elapsed_seconds = $clickWake.elapsed_seconds
        passed = $true
    }

    Start-Sleep -Seconds 1
    $rightY = $workArea[1] + [Math]::Max(0, [int](($workArea[3] - $workArea[1] - 200) / 2))
    if (-not [YondaNativeProbe]::MovePetWithoutActivation($process.Id, $workArea[2] - 200, $rightY)) { throw '无法把测试窗口移到右侧' }
    $rightStarted = Get-Date
    $right = Wait-PetRect $process { param($rect) Test-Docked $rect 'right' $workArea } $DockTimeoutSeconds
    if (-not $right) { throw "右侧样本在$DockTimeoutSeconds秒内未停靠" }
    Write-Output 'PROGRESS right-docked'
    $result.right_dock = [ordered]@{
        rect = Convert-Rect $right.rect
        elapsed_seconds = [Math]::Round(((Get-Date) - $rightStarted).TotalSeconds, 3)
        passed = $true
    }

    if (-not [YondaNativeProbe]::PressEnter($process.Id)) { throw '无法向右侧停靠窗口发送Enter' }
    $keyboardWake = Wait-PetRect $process { param($rect) Test-Expanded $rect } 5
    if (-not $keyboardWake) { throw '右侧停靠后Enter未恢复200×200窗口' }
    Write-Output 'PROGRESS keyboard-woke'
    $result.keyboard_wake = [ordered]@{
        key = 'Enter'
        rect = Convert-Rect $keyboardWake.rect
        elapsed_seconds = $keyboardWake.elapsed_seconds
        passed = $true
    }

    $result.passed = $true
} catch {
    $result.passed = $false
    $result.error = $_.Exception.Message
    throw
} finally {
    if ($process -and -not $process.HasExited) {
        $process.Kill()
        $process.WaitForExit()
    }
    if ($process) {
        $result.diagnostic_log = @($process.StandardError.ReadToEnd() -split "`r?`n" | Where-Object { $_ })
    }
    $result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'edges-wake-result.json') -Encoding UTF8
    if (Test-Path -LiteralPath $testRoot) { Remove-Item -LiteralPath $testRoot -Recurse -Force }
}

$result | ConvertTo-Json -Depth 8 -Compress
