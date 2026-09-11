$source = @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class ForegroundWindow {
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetWindowText(IntPtr hWnd, StringBuilder text, int count);
  [DllImport("user32.dll")] public static extern int GetWindowTextLength(IntPtr hWnd);
}
'@
Add-Type -TypeDefinition $source
$handle = [ForegroundWindow]::GetForegroundWindow()
if ($handle -eq [IntPtr]::Zero) { throw '没有前台窗口' }
$processId = 0
[void][ForegroundWindow]::GetWindowThreadProcessId($handle, [ref]$processId)
$length = [ForegroundWindow]::GetWindowTextLength($handle)
$title = [Text.StringBuilder]::new($length + 1)
[void][ForegroundWindow]::GetWindowText($handle, $title, $title.Capacity)
$process = Get-Process -Id $processId -ErrorAction Stop
[ordered]@{
  pid = $processId
  process_name = $process.ProcessName
  title_present = $title.Length -gt 0
  title_length = $title.Length
} | ConvertTo-Json -Compress
