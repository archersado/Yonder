param(
    [Parameter(Mandatory = $true)]
    [int]$ProcessId
)

$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName UIAutomationClient
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;

public static class VoiceWindowNative
{
    public delegate bool EnumWindowsProc(IntPtr handle, IntPtr parameter);

    [DllImport("user32.dll")]
    public static extern bool EnumWindows(EnumWindowsProc callback, IntPtr parameter);

    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr handle);

    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr handle, out uint processId);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern int GetWindowText(IntPtr handle, StringBuilder text, int count);

    public static IntPtr FindVisibleWindow(uint expectedProcessId, string expectedTitle)
    {
        IntPtr result = IntPtr.Zero;
        EnumWindows((handle, parameter) => {
            uint actualProcessId;
            GetWindowThreadProcessId(handle, out actualProcessId);
            if (actualProcessId != expectedProcessId || !IsWindowVisible(handle)) return true;
            var title = new StringBuilder(512);
            GetWindowText(handle, title, title.Capacity);
            if (title.ToString() == expectedTitle) {
                result = handle;
                return false;
            }
            return true;
        }, IntPtr.Zero);
        return result;
    }
}
'@

$process = Get-Process -Id $ProcessId -ErrorAction Stop
$deadline = [DateTime]::UtcNow.AddSeconds(10)
do {
    $process.Refresh()
    $petHandle = $process.MainWindowHandle
    if ($petHandle -ne [IntPtr]::Zero) { break }
    Start-Sleep -Milliseconds 200
} while ([DateTime]::UtcNow -lt $deadline)

if ($petHandle -eq [IntPtr]::Zero) { throw '未找到 Yonda 主窗口' }

$pet = [Windows.Automation.AutomationElement]::FromHandle($petHandle)
$voiceCondition = [Windows.Automation.PropertyCondition]::new(
    [Windows.Automation.AutomationElement]::AutomationIdProperty,
    'voice-trigger')
$voiceTrigger = $null
$deadline = [DateTime]::UtcNow.AddSeconds(10)
do {
    $voiceTrigger = $pet.FindFirst([Windows.Automation.TreeScope]::Descendants, $voiceCondition)
    if ($null -ne $voiceTrigger) { break }
    Start-Sleep -Milliseconds 200
} while ([DateTime]::UtcNow -lt $deadline)
if ($null -eq $voiceTrigger) { throw '未找到语音输入按钮' }
$invoke = $voiceTrigger.GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern)
$invoke.Invoke()

$voiceHandle = [IntPtr]::Zero
$deadline = [DateTime]::UtcNow.AddSeconds(5)
do {
    $voiceHandle = [VoiceWindowNative]::FindVisibleWindow([uint32]$ProcessId, 'Yonda · 语音输入')
    if ($voiceHandle -ne [IntPtr]::Zero) { break }
    Start-Sleep -Milliseconds 200
} while ([DateTime]::UtcNow -lt $deadline)

$controls = [Collections.Generic.List[object]]::new()
if ($voiceHandle -ne [IntPtr]::Zero) {
    Start-Sleep -Seconds 2
    $voiceWindow = [Windows.Automation.AutomationElement]::FromHandle($voiceHandle)
    $all = $voiceWindow.FindAll(
        [Windows.Automation.TreeScope]::Descendants,
        [Windows.Automation.Condition]::TrueCondition)
    foreach ($element in $all) {
        try {
            $name = $element.Current.Name
            if ($name -and $name -match '准备|权限|暂不可用|失败|重试|重新开始|停止|取消') {
                $controls.Add([ordered]@{
                    name = $name
                    control_type = $element.Current.ControlType.ProgrammaticName
                    automation_id = $element.Current.AutomationId
                    enabled = $element.Current.IsEnabled
                    offscreen = $element.Current.IsOffscreen
                })
            }
        } catch {}
    }
}

$statusNames = @($controls | ForEach-Object { $_.name })
$reportsExplicitFailure = @($statusNames | Where-Object { $_ -match '暂不可用|失败' }).Count -gt 0
[ordered]@{
    process_id = $ProcessId
    voice_window_visible = $voiceHandle -ne [IntPtr]::Zero
    controls = $controls
    reports_explicit_failure = $reportsExplicitFailure
    passed = ($voiceHandle -ne [IntPtr]::Zero) -and $reportsExplicitFailure
} | ConvertTo-Json -Depth 5
