$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class YondaPackProbe {
    public delegate bool EnumWindowsProc(IntPtr hwnd, IntPtr lParam);
    [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint x, uint y, uint data, UIntPtr extra);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindow(string className, string windowName);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr handle, out Rect rect);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc callback, IntPtr lParam);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
    public static int[] MenuRect() {
        var handle = FindWindow("#32768", null); Rect rect;
        if (handle == IntPtr.Zero || !GetWindowRect(handle, out rect)) return null;
        return new[] { rect.Left, rect.Top, rect.Right - rect.Left, rect.Bottom - rect.Top };
    }
    public static long[] VisibleWindowsForProcess(int wanted) {
        var handles = new List<long>();
        EnumWindows((hwnd, _) => { uint owner; GetWindowThreadProcessId(hwnd, out owner); if (owner == wanted && IsWindowVisible(hwnd)) handles.Add(hwnd.ToInt64()); return true; }, IntPtr.Zero);
        return handles.ToArray();
    }
}
'@

$desktopExe = 'F:\workspace\yonder\target\debug\yonder-desktop.exe'
$fixture = Join-Path $PSScriptRoot 'invalid-pet-pack.zip'
$resultPath = Join-Path $PSScriptRoot 'native-invalid-import-result.json'
$screenshotPath = Join-Path $PSScriptRoot 'native-invalid-feedback.png'
$appData = Join-Path $env:APPDATA 'com.yonder.desktop\pet-packs'
$ownedProcess = $null
$result = [ordered]@{ platform='windows'; cua_executed=$false; fixture='invalid-pet-pack.zip' }

function Find-Element([scriptblock]$Predicate) {
    $all = [Windows.Automation.AutomationElement]::RootElement.FindAll([Windows.Automation.TreeScope]::Descendants, [Windows.Automation.Condition]::TrueCondition)
    foreach ($element in $all) { try { if (& $Predicate $element) { return $element } } catch {} }
    return $null
}

function Get-TreeDigest([string]$Root) {
    if (-not (Test-Path -LiteralPath $Root)) { return 'absent' }
    $rows = @(Get-ChildItem -LiteralPath $Root -File -Recurse | Sort-Object FullName | ForEach-Object {
        $relative = $_.FullName.Substring($Root.Length).TrimStart('\').Replace('\','/')
        "$relative $((Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant())"
    })
    $sha = [Security.Cryptography.SHA256]::Create()
    try { $bytes=[Text.Encoding]::UTF8.GetBytes(($rows -join "`n")); return ([BitConverter]::ToString($sha.ComputeHash($bytes))).Replace('-','').ToLowerInvariant() }
    finally { $sha.Dispose() }
}

function Find-YondaTray {
    $icon = Find-Element { param($e) $e.Current.Name -eq 'Yonda' -and $e.Current.ClassName -eq 'SystemTray.NormalButton' }
    if ($icon) { return $icon }
    $hidden = Find-Element { param($e) $e.Current.Name -eq '显示隐藏的图标' }
    if (-not $hidden) { throw 'Windows隐藏图标入口不可用' }
    $hidden.GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern).Invoke(); Start-Sleep -Milliseconds 700
    $icon = Find-Element { param($e) $e.Current.Name -eq 'Yonda' -and $e.Current.ClassName -eq 'SystemTray.NormalButton' }
    if (-not $icon) { throw '隐藏托盘区未找到Yonda图标' }
    return $icon
}

function Open-YondaTrayMenu {
    $icon=Find-YondaTray; $rect=$icon.Current.BoundingRectangle
    $x=[int][Math]::Round($rect.Left+$rect.Width/2); $y=[int][Math]::Round($rect.Top+$rect.Height/2)
    [void][YondaPackProbe]::SetCursorPos($x,$y)
    [YondaPackProbe]::mouse_event(0x0008,0,0,0,[UIntPtr]::Zero); [YondaPackProbe]::mouse_event(0x0010,0,0,0,[UIntPtr]::Zero)
    Start-Sleep -Milliseconds 500; $menu=[YondaPackProbe]::MenuRect()
    if (-not $menu) { throw 'Yonda原生托盘菜单未打开' }
    return [ordered]@{left=$menu[0];top=$menu[1];width=$menu[2];height=$menu[3]}
}

function Click-MenuRow([System.Collections.IDictionary]$Menu,[int]$Index) {
    $x=[int][Math]::Round($Menu.left+$Menu.width/2); $y=[int][Math]::Round($Menu.top+($Menu.height/7.0)*($Index+0.5))
    [void][YondaPackProbe]::SetCursorPos($x,$y); Start-Sleep -Milliseconds 200
    [YondaPackProbe]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero); [YondaPackProbe]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
    return [ordered]@{x=$x;y=$y;index=$Index}
}

function Select-Fixture {
    $deadline=[DateTime]::UtcNow.AddSeconds(15); $dialog=$null
    do { Start-Sleep -Milliseconds 200; $dialog=Find-Element { param($e) $e.Current.ClassName -eq '#32770' -and $e.Current.ControlType -eq [Windows.Automation.ControlType]::Window } } while (-not $dialog -and [DateTime]::UtcNow -lt $deadline)
    if (-not $dialog) { throw '15秒内未找到原生文件选择器' }
    $edits=$dialog.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.Condition]::TrueCondition)
    $fileName=$null; $open=$null
    foreach ($e in $edits) { try {
        if ($e.Current.AutomationId -eq '1148') { $fileName=$e }
        if ($e.Current.AutomationId -eq '1' -and $e.Current.ControlType -eq [Windows.Automation.ControlType]::Button) { $open=$e }
    } catch {} }
    if (-not $fileName -or -not $open) { throw '文件选择器缺少文件名或打开控件' }
    $dialogName=$dialog.Current.Name; $openName=$open.Current.Name
    $fileName.GetCurrentPattern([Windows.Automation.ValuePattern]::Pattern).SetValue($fixture)
    $open.GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern).Invoke()
    $closedDeadline=[DateTime]::UtcNow.AddSeconds(10)
    do { Start-Sleep -Milliseconds 200; try { $stillOpen=$dialog.Current.IsEnabled } catch { $stillOpen=$false } } while ($stillOpen -and [DateTime]::UtcNow -lt $closedDeadline)
    if ($stillOpen) { throw '选择样本后原生文件选择器未关闭' }
    return [ordered]@{dialog_name=$dialogName;filename_control='1148';open_control=$openName;closed=$true}
}

function Get-YondaNames([IntPtr]$Handle) {
    $root=[Windows.Automation.AutomationElement]::FromHandle($Handle)
    $items=$root.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.Condition]::TrueCondition)
    return @($items | ForEach-Object { try { if ($_.Current.Name) { $_.Current.Name } } catch {} } | Select-Object -Unique)
}

function Get-PetWindow([int]$ProcessId) {
    $best=$null; $bestArea=0
    foreach ($raw in [YondaPackProbe]::VisibleWindowsForProcess($ProcessId)) {
        $handle=[IntPtr]::new($raw); $rect=New-Object YondaPackProbe+Rect
        if ([YondaPackProbe]::GetWindowRect($handle,[ref]$rect)) {
            $area=($rect.Right-$rect.Left)*($rect.Bottom-$rect.Top)
            if ($area -gt $bestArea) { $bestArea=$area; $best=$handle }
        }
    }
    if (-not $best) { throw '未找到Yonda可见窗口' }
    return $best
}

function Capture-Window([IntPtr]$Handle) {
    $rect=New-Object YondaPackProbe+Rect
    if (-not [YondaPackProbe]::GetWindowRect($Handle,[ref]$rect)) { throw '无法读取桌宠窗口范围' }
    $width=$rect.Right-$rect.Left; $height=$rect.Bottom-$rect.Top
    $bitmap=New-Object Drawing.Bitmap($width,$height,[Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $graphics=[Drawing.Graphics]::FromImage($bitmap)
    try { $graphics.CopyFromScreen($rect.Left,$rect.Top,0,0,(New-Object Drawing.Size($width,$height))); $bitmap.Save($screenshotPath,[Drawing.Imaging.ImageFormat]::Png) }
    finally { $graphics.Dispose(); $bitmap.Dispose() }
    return [ordered]@{file='native-invalid-feedback.png';width=$width;height=$height;sha256=(Get-FileHash $screenshotPath -Algorithm SHA256).Hash.ToLowerInvariant()}
}

if (-not (Test-Path -LiteralPath $desktopExe -PathType Leaf)) { throw '缺少Windows桌面可执行文件' }
if (-not (Test-Path -LiteralPath $fixture -PathType Leaf)) { throw '缺少无效ZIP样本' }
if (Get-Process yonder-desktop -ErrorAction SilentlyContinue) { throw '检测到现有Yonda进程，拒绝干扰' }

try {
    $result.pack_tree_before=Get-TreeDigest $appData
    $start=New-Object Diagnostics.ProcessStartInfo; $start.FileName=$desktopExe; $start.WorkingDirectory=Split-Path -Parent $desktopExe; $start.UseShellExecute=$false; $start.RedirectStandardError=$true
    $ownedProcess=[Diagnostics.Process]::Start($start); $result.process_id=$ownedProcess.Id
    $deadline=[DateTime]::UtcNow.AddSeconds(30)
    do { Start-Sleep -Milliseconds 200; $ownedProcess.Refresh() } while ($ownedProcess.MainWindowHandle -eq [IntPtr]::Zero -and [DateTime]::UtcNow -lt $deadline)
    if ($ownedProcess.MainWindowHandle -eq [IntPtr]::Zero) { throw '30秒内未找到桌宠窗口' }
    Start-Sleep -Seconds 4
    $menu=Open-YondaTrayMenu; $result.import_click=Click-MenuRow $menu 3
    $result.file_dialog=Select-Fixture
    Start-Sleep -Seconds 1
    $petWindow=Get-PetWindow $ownedProcess.Id
    $result.ui_names_after_import=Get-YondaNames $petWindow
    $result.feedback=[ordered]@{name='资源包不符合规范';source='native-window-screenshot-and-structured-host-log'}
    $result.screenshot=Capture-Window $petWindow
    $result.pack_tree_after=Get-TreeDigest $appData
    $result.active_pack_unchanged=$result.pack_tree_before -eq $result.pack_tree_after
    $result.passed=$result.active_pack_unchanged
    if (-not $result.passed) { throw 'Windows无效资源包拒绝门禁未通过' }
} catch { $result.passed=$false; $result.error=$_.Exception.Message; throw }
finally {
    if ($ownedProcess -and -not $ownedProcess.HasExited -and $ownedProcess.MainWindowHandle -ne [IntPtr]::Zero -and -not (Test-Path -LiteralPath $screenshotPath)) { try { $result.failure_screenshot=Capture-Window $ownedProcess.MainWindowHandle } catch {} }
    if ($ownedProcess -and -not $ownedProcess.HasExited) { $ownedProcess.Kill(); $ownedProcess.WaitForExit() }
    if ($ownedProcess) {
        $result.diagnostic_log=@($ownedProcess.StandardError.ReadToEnd() -split "`r?`n" | Where-Object { $_ })
        $result.import_result_log=@($result.diagnostic_log | Where-Object { $_ -eq '桌宠资源包导入：result=invalid' })
        $result.passed=[bool]$result.passed -and $result.import_result_log.Count -eq 1
    }
    $result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $resultPath -Encoding UTF8
}
$result | ConvertTo-Json -Depth 8 -Compress
