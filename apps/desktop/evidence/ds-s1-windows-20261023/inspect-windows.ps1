param(
    [Parameter(Mandatory = $true)]
    [int]$ProcessId
)

$ErrorActionPreference = 'Stop'

function Get-Sha256([string]$Path) {
    $stream = [IO.File]::OpenRead($Path)
    try {
        $sha = [Security.Cryptography.SHA256]::Create()
        try {
            return ([BitConverter]::ToString($sha.ComputeHash($stream))).Replace('-', '').ToLowerInvariant()
        } finally {
            $sha.Dispose()
        }
    } finally {
        $stream.Dispose()
    }
}

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class NativeWindowProbe
{
    public delegate bool EnumWindowsProc(IntPtr hwnd, IntPtr lParam);

    [StructLayout(LayoutKind.Sequential)]
    public struct Rect
    {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }

    [DllImport("user32.dll")]
    public static extern bool EnumWindows(EnumWindowsProc callback, IntPtr lParam);

    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);

    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr hwnd);

    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hwnd, out Rect rect);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern int GetWindowText(IntPtr hwnd, StringBuilder text, int count);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern int GetClassName(IntPtr hwnd, StringBuilder text, int count);

    [DllImport("user32.dll", EntryPoint = "GetWindowLongPtrW")]
    public static extern IntPtr GetWindowLongPtr(IntPtr hwnd, int index);
}
'@

$process = Get-Process -Id $ProcessId
$windows = [Collections.Generic.List[object]]::new()
$callback = [NativeWindowProbe+EnumWindowsProc]{
    param([IntPtr]$hwnd, [IntPtr]$lParam)
    $owner = [uint32]0
    [void][NativeWindowProbe]::GetWindowThreadProcessId($hwnd, [ref]$owner)
    if ($owner -eq $ProcessId) {
        $rect = [NativeWindowProbe+Rect]::new()
        [void][NativeWindowProbe]::GetWindowRect($hwnd, [ref]$rect)
        $title = [Text.StringBuilder]::new(512)
        $class = [Text.StringBuilder]::new(256)
        [void][NativeWindowProbe]::GetWindowText($hwnd, $title, $title.Capacity)
        [void][NativeWindowProbe]::GetClassName($hwnd, $class, $class.Capacity)
        $style = [uint64][NativeWindowProbe]::GetWindowLongPtr($hwnd, -16).ToInt64()
        $extended = [uint64][NativeWindowProbe]::GetWindowLongPtr($hwnd, -20).ToInt64()
        $windows.Add([ordered]@{
            handle = $hwnd.ToInt64()
            title = $title.ToString()
            class_name = $class.ToString()
            visible = [NativeWindowProbe]::IsWindowVisible($hwnd)
            left = $rect.Left
            top = $rect.Top
            right = $rect.Right
            bottom = $rect.Bottom
            width = $rect.Right - $rect.Left
            height = $rect.Bottom - $rect.Top
            style = ('0x{0:X}' -f $style)
            extended_style = ('0x{0:X}' -f $extended)
            topmost = (($extended -band 0x8) -ne 0)
            tool_window = (($extended -band 0x80) -ne 0)
            layered = (($extended -band 0x80000) -ne 0)
            no_activate = (($extended -band 0x8000000) -ne 0)
        })
    }
    return $true
}
[void][NativeWindowProbe]::EnumWindows($callback, [IntPtr]::Zero)

$tcp = @(Get-NetTCPConnection -OwningProcess $ProcessId -ErrorAction SilentlyContinue)
$children = @(Get-CimInstance Win32_Process -Filter "ParentProcessId=$ProcessId" |
    Select-Object ProcessId, Name, ExecutablePath)
$os = Get-CimInstance Win32_OperatingSystem

[ordered]@{
    collected_at = (Get-Date).ToString('o')
    os = [ordered]@{
        caption = $os.Caption
        version = $os.Version
        architecture = $os.OSArchitecture
    }
    process = [ordered]@{
        id = $process.Id
        name = $process.ProcessName
        path = $process.Path
        responding = $process.Responding
        start_time = $process.StartTime.ToString('o')
        executable_sha256 = Get-Sha256 $process.Path
    }
    windows = $windows
    owned_tcp_connections = $tcp.Count
    children = $children
} | ConvertTo-Json -Depth 8
