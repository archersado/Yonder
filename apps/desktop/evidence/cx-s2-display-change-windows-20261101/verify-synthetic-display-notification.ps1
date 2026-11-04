$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName UIAutomationClient
Add-Type @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;

public static class YonderDisplayChangeProbe {
    public delegate bool EnumWindowsProc(IntPtr hwnd, IntPtr state);
    [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc callback, IntPtr state);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetWindowText(IntPtr hwnd, StringBuilder text, int count);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindow(string className, string windowName);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindowEx(IntPtr parent, IntPtr childAfter, string className, string windowName);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd, out Rect rect);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint x, uint y, uint data, UIntPtr extra);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr hwnd, uint message, UIntPtr wparam, IntPtr lparam);

    public static long FindProcessWindow(int wantedProcessId, string titlePrefix, bool visibleOnly) {
        long result = 0;
        EnumWindows((hwnd, _) => {
            uint processId; GetWindowThreadProcessId(hwnd, out processId);
            if (processId != wantedProcessId || (visibleOnly && !IsWindowVisible(hwnd))) return true;
            var title = new StringBuilder(512); GetWindowText(hwnd, title, title.Capacity);
            if (!title.ToString().StartsWith(titlePrefix)) return true;
            result = hwnd.ToInt64(); return false;
        }, IntPtr.Zero);
        return result;
    }
    public static string WindowTitle(long raw) {
        var title = new StringBuilder(512); GetWindowText(new IntPtr(raw), title, title.Capacity); return title.ToString();
    }
    public static bool Visible(long raw) { return IsWindowVisible(new IntPtr(raw)); }
    public static int[] MenuRect() {
        var hwnd=FindWindow("#32768",null); Rect rect;
        return hwnd!=IntPtr.Zero && GetWindowRect(hwnd,out rect) ? new[]{rect.Left,rect.Top,rect.Right-rect.Left,rect.Bottom-rect.Top} : null;
    }
    public static long WatcherHandle() { return FindWindowEx(new IntPtr(-3),IntPtr.Zero,"YonderRegionDisplayWatch",null).ToInt64(); }
}
'@

$desktopExe = 'F:\workspace\yonder\target\debug\yonder-desktop.exe'
$resultPath = Join-Path $PSScriptRoot 'synthetic-display-notification-result.json'
$process = $null
$normalExit = $false
$result = [ordered]@{platform='windows';notification='synthetic-WM_DISPLAYCHANGE';physical_display_changed=$false;cua_executed=$false}

function Find-ElementInWindow([string]$ClassName,[scriptblock]$Predicate) {
    $handle=[YonderDisplayChangeProbe]::FindWindow($ClassName,$null)
    if($handle -eq [IntPtr]::Zero){return $null}
    $root=[Windows.Automation.AutomationElement]::FromHandle($handle)
    $all=$root.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.Condition]::TrueCondition)
    foreach($element in $all){try{if(& $Predicate $element){return $element}}catch{}}
    $null
}

function Find-ElementInRootChild([string]$ClassName,[scriptblock]$Predicate) {
    $windows=[Windows.Automation.AutomationElement]::RootElement.FindAll([Windows.Automation.TreeScope]::Children,[Windows.Automation.Condition]::TrueCondition)
    $root=$null
    foreach($window in $windows){try{if($window.Current.ClassName -eq $ClassName){$root=$window;break}}catch{}}
    if(-not $root){return $null}
    $all=$root.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.Condition]::TrueCondition)
    foreach($element in $all){try{if(& $Predicate $element){return $element}}catch{}}
    $null
}

function Find-YonderTray {
    $icon=Find-ElementInWindow 'Shell_TrayWnd' {param($e) $e.Current.Name -eq 'Yonda' -and $e.Current.ClassName -eq 'SystemTray.NormalButton'}
    if(-not $icon){$icon=Find-ElementInRootChild 'TopLevelWindowForOverflowXamlIsland' {param($e) $e.Current.Name -eq 'Yonda' -and $e.Current.ClassName -eq 'SystemTray.NormalButton'}}
    if($icon){return $icon}
    $hidden=Find-ElementInWindow 'Shell_TrayWnd' {param($e) $e.Current.Name -eq '显示隐藏的图标'}
    if(-not $hidden){throw 'Windows隐藏图标入口不可用'}
    $hidden.GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern).Invoke();Start-Sleep -Milliseconds 700
    $icon=Find-ElementInRootChild 'TopLevelWindowForOverflowXamlIsland' {param($e) $e.Current.Name -eq 'Yonda' -and $e.Current.ClassName -eq 'SystemTray.NormalButton'}
    if(-not $icon){throw '隐藏托盘区未找到Yonda图标'}
    $icon
}

function Open-TrayMenu {
    $icon=Find-YonderTray;$bounds=$icon.Current.BoundingRectangle
    $x=[int][Math]::Round($bounds.Left+$bounds.Width/2);$y=[int][Math]::Round($bounds.Top+$bounds.Height/2)
    [void][YonderDisplayChangeProbe]::SetCursorPos($x,$y)
    [YonderDisplayChangeProbe]::mouse_event(0x0008,0,0,0,[UIntPtr]::Zero);[YonderDisplayChangeProbe]::mouse_event(0x0010,0,0,0,[UIntPtr]::Zero)
    Start-Sleep -Milliseconds 500;$menu=[YonderDisplayChangeProbe]::MenuRect()
    if(-not $menu){throw 'Yonder托盘菜单未打开'}
    [ordered]@{left=$menu[0];top=$menu[1];width=$menu[2];height=$menu[3]}
}

function Click-MenuRow([System.Collections.IDictionary]$Menu,[int]$Index) {
    $x=[int][Math]::Round($Menu.left+$Menu.width/2);$y=[int][Math]::Round($Menu.top+($Menu.height/7.0)*($Index+0.5))
    [void][YonderDisplayChangeProbe]::SetCursorPos($x,$y)
    [YonderDisplayChangeProbe]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero);[YonderDisplayChangeProbe]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
    [ordered]@{index=$Index;x=$x;y=$y}
}

if(-not(Test-Path -LiteralPath $desktopExe -PathType Leaf)){throw '缺少Windows桌面可执行文件'}
if(Get-Process yonder-desktop -ErrorAction SilentlyContinue){throw '检测到现有Yonder进程，拒绝干扰'}

try {
    $start=[Diagnostics.ProcessStartInfo]::new();$start.FileName=$desktopExe;$start.WorkingDirectory=Split-Path -Parent $desktopExe;$start.UseShellExecute=$false;$start.RedirectStandardError=$true
    $process=[Diagnostics.Process]::Start($start);$result.process_id=$process.Id
    Write-Output 'stage=process-started'
    $deadline=[DateTime]::UtcNow.AddSeconds(30)
    do{Start-Sleep -Milliseconds 200;$process.Refresh()}while($process.MainWindowHandle -eq [IntPtr]::Zero -and [DateTime]::UtcNow -lt $deadline)
    if($process.MainWindowHandle -eq [IntPtr]::Zero){throw '30秒内未找到Yonder窗口'}
    Start-Sleep -Seconds 3
    Write-Output 'stage=desktop-ready'

    $menu=Open-TrayMenu;$result.region_click=Click-MenuRow $menu 2
    Write-Output 'stage=region-menu-clicked'
    $deadline=[DateTime]::UtcNow.AddSeconds(10);$preview=0
    do{Start-Sleep -Milliseconds 100;$preview=[YonderDisplayChangeProbe]::FindProcessWindow($process.Id,'Yonda · 圈选提问',$true)}while($preview -eq 0 -and [DateTime]::UtcNow -lt $deadline)
    if($preview -eq 0){throw '圈选预览窗口未显示'}
    $watcher=[YonderDisplayChangeProbe]::WatcherHandle()
    if($watcher -eq 0){throw 'Windows显示变化观察器未创建'}
    $result.preview_visible_before=$true;$result.watcher_created=$true
    Write-Output 'stage=watcher-ready'

    if(-not [YonderDisplayChangeProbe]::PostMessage([IntPtr]::new($watcher),0x007E,[UIntPtr]::new(32),[IntPtr]::Zero)){throw 'WM_DISPLAYCHANGE投递失败'}
    $deadline=[DateTime]::UtcNow.AddSeconds(5)
    do{Start-Sleep -Milliseconds 100;$visible=[YonderDisplayChangeProbe]::Visible($preview);$title=[YonderDisplayChangeProbe]::WindowTitle($preview)}while(($visible -or $title -notmatch 'reason=display-change') -and [DateTime]::UtcNow -lt $deadline)
    $result.preview_visible_after=$visible;$result.hidden_title=$title
    $result.watcher_removed_after_clear=([YonderDisplayChangeProbe]::WatcherHandle() -eq 0)
    $result.cleanup_reason_display_change=$title -match 'reason=display-change'
    Write-Output 'stage=notification-observed'

    $quitMenu=Open-TrayMenu;$result.quit_click=Click-MenuRow $quitMenu 6
    Write-Output 'stage=quit-clicked'
    if(-not $process.WaitForExit(15000)){throw 'Yonder未正常退出'}
    $normalExit=$true
    $result.exit_code=$process.ExitCode
    $result.passed=(-not $visible)-and $result.cleanup_reason_display_change -and $result.watcher_removed_after_clear -and $process.ExitCode -eq 0
    if(-not $result.passed){throw 'Windows显示变化合成通知门禁未通过'}
}catch{$result.passed=$false;$result.error=$_.Exception.Message;throw}
finally{
    if($process -and -not $process.HasExited){$process.Kill();$process.WaitForExit()}
    if($process){$result.diagnostic_log=@($process.StandardError.ReadToEnd() -split "`r?`n"|Where-Object{$_})}
    $result.normal_exit=$normalExit
    [IO.File]::WriteAllText($resultPath,($result|ConvertTo-Json -Depth 8),[Text.UTF8Encoding]::new($false))
}
$result|ConvertTo-Json -Depth 8 -Compress
