$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;

public static class YondaTrayProbe {
    [StructLayout(LayoutKind.Sequential)]
    public struct Rect { public int Left, Top, Right, Bottom; }
    [StructLayout(LayoutKind.Sequential)]
    public struct Point { public int X, Y; }
    [DllImport("user32.dll")]
    public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")]
    public static extern bool GetCursorPos(out Point point);
    [DllImport("user32.dll")]
    public static extern void mouse_event(uint flags, uint x, uint y, uint data, UIntPtr extra);
    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr handle, int command);
    [DllImport("user32.dll")]
    public static extern bool IsIconic(IntPtr handle);
    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr handle);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern IntPtr FindWindow(string className, string windowName);
    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr handle, out Rect rect);

    public static int[] MenuRect() {
        var handle = FindWindow("#32768", null);
        Rect rect;
        if (handle == IntPtr.Zero || !GetWindowRect(handle, out rect)) return null;
        return new[] { rect.Left, rect.Top, rect.Right - rect.Left, rect.Bottom - rect.Top };
    }

    public static int[] CursorPosition() {
        Point point;
        return GetCursorPos(out point) ? new[] { point.X, point.Y } : null;
    }
}
'@

$desktopExe = 'F:\workspace\yonder\target\debug\yonder-desktop.exe'
$resultPath = Join-Path $PSScriptRoot 'native-tray-exit-result.json'
$screenshotPath = Join-Path $PSScriptRoot 'native-tray-menu.png'
$ownedProcess = $null
$forcedCleanup = $false
$result = [ordered]@{ platform = 'windows'; cua_executed = $false }

function Find-Element([string]$Name, [string]$ClassName = '') {
    $all = [Windows.Automation.AutomationElement]::RootElement.FindAll(
        [Windows.Automation.TreeScope]::Descendants,
        [Windows.Automation.Condition]::TrueCondition)
    foreach ($element in $all) {
        try {
            if ($element.Current.Name -eq $Name -and (!$ClassName -or $element.Current.ClassName -eq $ClassName)) {
                return $element
            }
        } catch {}
    }
    return $null
}

function Find-YondaTray {
    $icon = Find-Element 'Yonda' 'SystemTray.NormalButton'
    if ($icon) { return $icon }
    $hidden = Find-Element '显示隐藏的图标'
    if (-not $hidden) { throw 'Windows隐藏图标入口不可用' }
    $hidden.GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern).Invoke()
    Start-Sleep -Milliseconds 700
    $icon = Find-Element 'Yonda' 'SystemTray.NormalButton'
    if (-not $icon) { throw '隐藏托盘区未找到Yonda图标' }
    return $icon
}

function Open-YondaTrayMenu {
    $icon = Find-YondaTray
    $rect = $icon.Current.BoundingRectangle
    $x = [int][Math]::Round($rect.Left + $rect.Width / 2)
    $y = [int][Math]::Round($rect.Top + $rect.Height / 2)
    [void][YondaTrayProbe]::SetCursorPos($x, $y)
    [YondaTrayProbe]::mouse_event(0x0008, 0, 0, 0, [UIntPtr]::Zero)
    [YondaTrayProbe]::mouse_event(0x0010, 0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 500
    $menu = [YondaTrayProbe]::MenuRect()
    if (-not $menu) { throw 'Yonda原生托盘菜单未打开' }
    return [ordered]@{ x = $x; y = $y; name = $icon.Current.Name; class_name = $icon.Current.ClassName; menu = [ordered]@{ left=$menu[0]; top=$menu[1]; width=$menu[2]; height=$menu[3] } }
}

function Click-MenuRow([System.Collections.IDictionary]$Menu, [int]$Index) {
    $rowHeight = $Menu.height / 7.0
    $x = [int][Math]::Round($Menu.left + $Menu.width / 2.0)
    $y = [int][Math]::Round($Menu.top + $rowHeight * ($Index + 0.5))
    [void][YondaTrayProbe]::SetCursorPos($x, $y)
    Start-Sleep -Milliseconds 250
    $actual = [YondaTrayProbe]::CursorPosition()
    $element = [Windows.Automation.AutomationElement]::FromPoint([Windows.Point]::new($x, $y))
    $invoked = $false
    try {
        $element.GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern).Invoke()
        $invoked = $true
    } catch {
        [YondaTrayProbe]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
        [YondaTrayProbe]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
    }
    return [ordered]@{ target_x=$x; target_y=$y; actual_x=$actual[0]; actual_y=$actual[1]; element_name=$element.Current.Name; element_type=$element.Current.ControlType.ProgrammaticName; invoked=$invoked }
}

function Capture-TrayMenu([System.Collections.IDictionary]$Menu) {
    $bounds = [Windows.Forms.Screen]::PrimaryScreen.Bounds
    $width = [Math]::Min($Menu.width + 20, $bounds.Width)
    $height = [Math]::Min($Menu.height + 20, $bounds.Height)
    $left = [Math]::Max($bounds.Left, [Math]::Min($Menu.left - 10, $bounds.Right - $width))
    $top = [Math]::Max($bounds.Top, [Math]::Min($Menu.top - 10, $bounds.Bottom - $height))
    $bitmap = New-Object Drawing.Bitmap($width, $height, [Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.CopyFromScreen($left, $top, 0, 0, (New-Object Drawing.Size($width, $height)))
        $bitmap.Save($screenshotPath, [Drawing.Imaging.ImageFormat]::Png)
    } finally {
        $graphics.Dispose()
        $bitmap.Dispose()
    }
    return [ordered]@{ file = 'native-tray-menu.png'; left = $left; top = $top; width = $width; height = $height; sha256 = (Get-FileHash $screenshotPath -Algorithm SHA256).Hash.ToLowerInvariant() }
}

function Get-OwnedProcessIds([int]$RootId) {
    $all = @(Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId, Name)
    $ids = [Collections.Generic.HashSet[int]]::new()
    [void]$ids.Add($RootId)
    do {
        $changed = $false
        foreach ($process in $all) {
            if ($ids.Contains([int]$process.ParentProcessId) -and $ids.Add([int]$process.ProcessId)) { $changed = $true }
        }
    } while ($changed)
    return @($all | Where-Object { $ids.Contains([int]$_.ProcessId) } | Sort-Object ProcessId)
}

if (-not (Test-Path -LiteralPath $desktopExe -PathType Leaf)) { throw '缺少Windows桌面可执行文件' }
if (Get-Process yonder-desktop -ErrorAction SilentlyContinue) { throw '检测到现有Yonda进程，拒绝干扰' }

try {
    $start = New-Object Diagnostics.ProcessStartInfo
    $start.FileName = $desktopExe
    $start.WorkingDirectory = Split-Path -Parent $desktopExe
    $start.UseShellExecute = $false
    $start.RedirectStandardError = $true
    $ownedProcess = [Diagnostics.Process]::Start($start)
    $result.process_id = $ownedProcess.Id

    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    do {
        Start-Sleep -Milliseconds 200
        $ownedProcess.Refresh()
    } while ($ownedProcess.MainWindowHandle -eq [IntPtr]::Zero -and [DateTime]::UtcNow -lt $deadline)
    if ($ownedProcess.MainWindowHandle -eq [IntPtr]::Zero) { throw '30秒内未找到Yonda窗口' }
    Start-Sleep -Seconds 4

    $handle = $ownedProcess.MainWindowHandle
    [void][YondaTrayProbe]::ShowWindow($handle, 6)
    Start-Sleep -Milliseconds 500
    $result.minimized_before_show = [YondaTrayProbe]::IsIconic($handle)
    if (-not $result.minimized_before_show) { throw '无法建立托盘找回前置最小化状态' }

    $showMenu = Open-YondaTrayMenu
    $result.tray_icon = $showMenu
    $result.menu_capture = Capture-TrayMenu $showMenu.menu
    $result.show_click = Click-MenuRow $showMenu.menu 0
    Start-Sleep -Milliseconds 800
    $ownedProcess.Refresh()
    $currentHandle = $ownedProcess.MainWindowHandle
    $result.initial_window_handle = $handle.ToInt64()
    $result.restored_window_handle = $currentHandle.ToInt64()
    $result.restored_by_first_item = $currentHandle -ne [IntPtr]::Zero -and -not [YondaTrayProbe]::IsIconic($currentHandle) -and [YondaTrayProbe]::IsWindowVisible($currentHandle)
    if (-not $result.restored_by_first_item) { throw '托盘首项未恢复小龙窗口' }

    $tree = @(Get-OwnedProcessIds $ownedProcess.Id)
    $result.process_tree_before_exit = $tree
    $result.owned_tcp_connections = @($tree | ForEach-Object { Get-NetTCPConnection -OwningProcess $_.ProcessId -ErrorAction SilentlyContinue }).Count

    $quitMenu = Open-YondaTrayMenu
    $result.quit_click = Click-MenuRow $quitMenu.menu 6
    $exitWatch = [Diagnostics.Stopwatch]::StartNew()
    if (-not $ownedProcess.WaitForExit(20000)) { throw '托盘退出20秒内未结束宿主' }
    $result.exit_latency_ms_upper_bound = [Math]::Round($exitWatch.Elapsed.TotalMilliseconds, 3)
    Start-Sleep -Milliseconds 500
    $remaining = @($tree | Where-Object { Get-Process -Id $_.ProcessId -ErrorAction SilentlyContinue })
    $result.remaining_owned_process_ids = @($remaining | ForEach-Object { $_.ProcessId })
    $result.normal_exit_code = $ownedProcess.ExitCode
    $result.passed = $result.minimized_before_show -and $result.restored_by_first_item -and $remaining.Count -eq 0 -and $ownedProcess.ExitCode -eq 0
    if (-not $result.passed) { throw 'Windows托盘与退出门禁未通过' }
} catch {
    $result.passed = $false
    $result.error = $_.Exception.Message
    throw
} finally {
    if ($ownedProcess -and -not $ownedProcess.HasExited) {
        $forcedCleanup = $true
        $ownedProcess.Kill()
        $ownedProcess.WaitForExit()
    }
    if ($ownedProcess) { $result.diagnostic_log = @($ownedProcess.StandardError.ReadToEnd() -split "`r?`n" | Where-Object { $_ }) }
    $result.forced_cleanup = $forcedCleanup
    $result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $resultPath -Encoding UTF8
}

$result | ConvertTo-Json -Depth 8 -Compress
