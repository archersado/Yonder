param(
    [int]$ObservationSeconds = 190,
    [switch]$RequireDock
)

Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class YondaWindowProbe {
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

    [StructLayout(LayoutKind.Sequential)]
    private struct Rect { public int Left, Top, Right, Bottom; }

    public static int[] FindPet(int expectedProcessId) {
        int[] result = null;
        EnumWindows(delegate(IntPtr handle, IntPtr state) {
            uint processId;
            GetWindowThreadProcessId(handle, out processId);
            if (processId != expectedProcessId || !IsWindowVisible(handle)) return true;
            var title = new StringBuilder(128);
            GetWindowText(handle, title, title.Capacity);
            if (title.ToString() != "Yonda") return true;
            Rect rect;
            if (!GetWindowRect(handle, out rect)) return true;
            result = new[] { rect.Left, rect.Top, rect.Right - rect.Left, rect.Bottom - rect.Top };
            return false;
        }, IntPtr.Zero);
        return result;
    }

    public static bool MovePetWithoutActivation(int expectedProcessId, int x, int y) {
        bool moved = false;
        EnumWindows(delegate(IntPtr handle, IntPtr state) {
            uint processId;
            GetWindowThreadProcessId(handle, out processId);
            if (processId != expectedProcessId || !IsWindowVisible(handle)) return true;
            var title = new StringBuilder(128);
            GetWindowText(handle, title, title.Capacity);
            if (title.ToString() != "Yonda") return true;
            moved = SetWindowPos(handle, IntPtr.Zero, x, y, 0, 0, 0x0001 | 0x0004 | 0x0010);
            return false;
        }, IntPtr.Zero);
        return moved;
    }
}
'@

$executable = 'F:\workspace\yonder\target\debug\yonder-desktop.exe'
if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) {
    throw 'Windows desktop executable not found'
}

$testRoot = Join-Path $env:TEMP ('yonda-ds-s1-' + [guid]::NewGuid().ToString('N'))
$roaming = Join-Path $testRoot 'Roaming'
$local = Join-Path $testRoot 'Local'
New-Item -ItemType Directory -Path $roaming, $local | Out-Null

$process = $null
$watch = [Diagnostics.Stopwatch]::StartNew()
$initial = $null
$docked = $null
$dockElapsed = $null
try {
    $start = New-Object Diagnostics.ProcessStartInfo
    $start.FileName = $executable
    $start.WorkingDirectory = Split-Path -Parent $executable
    $start.UseShellExecute = $false
    $start.RedirectStandardError = $true
    $start.EnvironmentVariables['APPDATA'] = $roaming
    $start.EnvironmentVariables['LOCALAPPDATA'] = $local
    $process = [Diagnostics.Process]::Start($start)

    while ($watch.Elapsed.TotalSeconds -lt 30 -and -not $initial) {
        Start-Sleep -Milliseconds 250
        $initial = [YondaWindowProbe]::FindPet($process.Id)
    }
    if (-not $initial) { throw '30秒内没有找到Yonda原生窗口' }
    # 静态窗口会在 setup 中再次 center/show；等待其完成后再移动隔离测试窗口，避免覆盖。
    Start-Sleep -Seconds 5
    if (-not [YondaWindowProbe]::MovePetWithoutActivation($process.Id, 0, 0)) {
        throw '无法把隔离测试窗口移到不遮挡工作区的位置'
    }
    Start-Sleep -Milliseconds 250
    $initial = [YondaWindowProbe]::FindPet($process.Id)

    while ($watch.Elapsed.TotalSeconds -lt $ObservationSeconds) {
        Start-Sleep -Milliseconds 250
        $rect = [YondaWindowProbe]::FindPet($process.Id)
        if ($rect -and (($rect[2] -eq 112 -and $rect[3] -eq 56) -or ($rect[2] -eq 56 -and $rect[3] -eq 112))) {
            $docked = $rect
            $dockElapsed = [Math]::Round($watch.Elapsed.TotalSeconds, 3)
            break
        }
        if ($process.HasExited) { throw "Yonda提前退出，code=$($process.ExitCode)" }
    }
    $current = [YondaWindowProbe]::FindPet($process.Id)
    if (-not $process.HasExited) {
        $process.Kill()
        $process.WaitForExit()
    }
    $diagnostics = @($process.StandardError.ReadToEnd() -split "`r?`n" | Where-Object { $_ })

    [ordered]@{
        platform = 'windows'
        isolated_data = $true
        process_id = $process.Id
        initial = [ordered]@{ x = $initial[0]; y = $initial[1]; width = $initial[2]; height = $initial[3] }
        current = if ($current) { [ordered]@{ x = $current[0]; y = $current[1]; width = $current[2]; height = $current[3] } } else { $null }
        docked = if ($docked) { [ordered]@{ x = $docked[0]; y = $docked[1]; width = $docked[2]; height = $docked[3] } } else { $null }
        dock_elapsed_seconds = $dockElapsed
        automatic_idle_dock_passed = [bool]$docked
        diagnostic_log = $diagnostics
    } | ConvertTo-Json -Compress
    if ($RequireDock -and -not $docked) { throw "$ObservationSeconds 秒内没有进入自动停靠尺寸" }
} finally {
    if ($process -and -not $process.HasExited) {
        $process.Kill()
        $process.WaitForExit()
    }
    if (Test-Path -LiteralPath $testRoot) {
        Remove-Item -LiteralPath $testRoot -Recurse -Force
    }
}
