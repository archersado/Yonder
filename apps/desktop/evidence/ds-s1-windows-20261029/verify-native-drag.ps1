param(
    [int]$DeltaX = 180,
    [int]$DeltaY = 120
)

$ErrorActionPreference = 'Stop'

Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;

public static class YondaDragProbe {
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

    public static bool DragPet(int expectedProcessId, int deltaX, int deltaY) {
        var handle = FindPetHandle(expectedProcessId);
        Rect rect;
        if (handle == IntPtr.Zero || !GetWindowRect(handle, out rect)) return false;
        SetForegroundWindow(handle);
        int startX = (rect.Left + rect.Right) / 2;
        int startY = (rect.Top + rect.Bottom) / 2;
        if (!SetCursorPos(startX, startY)) return false;
        Thread.Sleep(100);
        var hit = WindowFromPoint(new Point { X = startX, Y = startY });
        if (hit == IntPtr.Zero || GetAncestor(hit, 2) != handle) return false;
        mouse_event(0x0002, 0, 0, 0, UIntPtr.Zero);
        Thread.Sleep(120);
        bool nativeMoveStarted = false;
        for (int step = 1; step <= 15; step++) {
            int x = startX + (int)Math.Round(deltaX * step / 15.0);
            int y = startY + (int)Math.Round(deltaY * step / 15.0);
            SetCursorPos(x, y);
            Thread.Sleep(45);
            Rect moved;
            if (GetWindowRect(handle, out moved) && (moved.Left != rect.Left || moved.Top != rect.Top)) nativeMoveStarted = true;
        }
        // start_dragging 通过WebView异步进入Windows移动循环；确认原生窗口已经
        // 实际移动后才释放，避免验证器先松键、命令后启动造成丢失释放事件。
        if (!nativeMoveStarted) {
            mouse_event(0x0004, 0, 0, 0, UIntPtr.Zero);
            return false;
        }
        mouse_event(0x0004, 0, 0, 0, UIntPtr.Zero);
        return true;
    }

    public static bool MoveCursor(int x, int y) { return SetCursorPos(x, y); }
}
'@

function Convert-Rect([int[]]$Rect) {
    if (-not $Rect) { return $null }
    [ordered]@{ x = $Rect[0]; y = $Rect[1]; width = $Rect[2]; height = $Rect[3] }
}

function Wait-Pet([Diagnostics.Process]$Process, [double]$TimeoutSeconds) {
    $watch = [Diagnostics.Stopwatch]::StartNew()
    do {
        if ($Process.HasExited) { throw "Yonda提前退出，code=$($Process.ExitCode)" }
        $rect = [YondaDragProbe]::FindPet($Process.Id)
        if ($rect) { return $rect }
        Start-Sleep -Milliseconds 250
    } while ($watch.Elapsed.TotalSeconds -lt $TimeoutSeconds)
    return $null
}

$executable = 'F:\workspace\yonder\target\debug\yonder-desktop.exe'
if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) { throw 'Windows desktop executable not found' }
$existing = @(Get-CimInstance Win32_Process -Filter "Name='yonder-desktop.exe'" | Where-Object { $_.ExecutablePath -eq $executable })
if ($existing.Count -gt 0) { throw "检测到现有Yonda进程，拒绝干扰：$($existing.ProcessId -join ',')" }

$testRoot = Join-Path $env:TEMP ('yonda-ds-s1-drag-' + [guid]::NewGuid().ToString('N'))
$roaming = Join-Path $testRoot 'Roaming'
$local = Join-Path $testRoot 'Local'
New-Item -ItemType Directory -Path $roaming, $local | Out-Null

$process = $null
$result = [ordered]@{
    platform = 'windows'
    input = 'Win32 primary-button drag'
    delta = [ordered]@{ x = $DeltaX; y = $DeltaY }
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

    $opened = Wait-Pet $process 30
    if (-not $opened) { throw '30秒内没有找到200×200的Yonda原生窗口' }
    Start-Sleep -Seconds 5
    if (-not [YondaDragProbe]::MovePetWithoutActivation($process.Id, 640, 400)) { throw '无法设置拖动样本初始位置' }
    Start-Sleep -Milliseconds 500
    $before = [YondaDragProbe]::FindPet($process.Id)
    if (-not $before) { throw '设置初始位置后窗口不可用' }

    $watch = [Diagnostics.Stopwatch]::StartNew()
    if (-not [YondaDragProbe]::DragPet($process.Id, $DeltaX, $DeltaY)) { throw '拖动起点没有命中Yonda窗口' }
    Start-Sleep -Seconds 1
    $afterRelease = [YondaDragProbe]::FindPet($process.Id)
    if (-not $afterRelease) { throw '拖动后窗口不可用' }
    $movedX = $afterRelease[0] - $before[0]
    $movedY = $afterRelease[1] - $before[1]
    $moved = [Math]::Abs($movedX) -ge 80 -and [Math]::Abs($movedY) -ge 50
    $sizePreserved = $afterRelease[2] -eq 200 -and $afterRelease[3] -eq 200
    $result.before = Convert-Rect $before
    $result.after_release = Convert-Rect $afterRelease
    $result.observed_delta = [ordered]@{ x = $movedX; y = $movedY }
    $result.position_changed = $moved
    $result.size_preserved = $sizePreserved
    if (-not $moved) { throw "原生拖动没有产生足够位移：dx=$movedX, dy=$movedY" }
    if (-not $sizePreserved) { throw '拖动改变了窗口尺寸' }

    [void][YondaDragProbe]::MoveCursor($afterRelease[0] + 360, $afterRelease[1] + 300)
    Start-Sleep -Seconds 1
    $afterFreeMove = [YondaDragProbe]::FindPet($process.Id)
    $stoppedAfterRelease = $afterFreeMove -and $afterFreeMove[0] -eq $afterRelease[0] -and $afterFreeMove[1] -eq $afterRelease[1]
    $result.after_free_cursor_move = Convert-Rect $afterFreeMove
    $result.stopped_after_release = $stoppedAfterRelease
    if (-not $stoppedAfterRelease) { throw '松开主键后窗口仍跟随鼠标移动' }

    $result.drag_duration_seconds = [Math]::Round($watch.Elapsed.TotalSeconds, 3)
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
    if ($process) { $result.diagnostic_log = @($process.StandardError.ReadToEnd() -split "`r?`n" | Where-Object { $_ }) }
    $result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'native-drag-result.json') -Encoding UTF8
    if (Test-Path -LiteralPath $testRoot) { Remove-Item -LiteralPath $testRoot -Recurse -Force }
}

$result | ConvertTo-Json -Depth 8 -Compress
