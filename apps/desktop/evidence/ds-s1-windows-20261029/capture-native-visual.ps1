$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class YondaVisualProbe {
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
    private static extern uint GetDpiForWindow(IntPtr handle);
    [StructLayout(LayoutKind.Sequential)]
    private struct Rect { public int Left, Top, Right, Bottom; }

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

    public static uint PetDpi(int expectedProcessId) {
        var handle = FindPetHandle(expectedProcessId);
        return handle == IntPtr.Zero ? 0 : GetDpiForWindow(handle);
    }
}
'@

function Convert-Color([Drawing.Color]$Color) {
    [ordered]@{ r = $Color.R; g = $Color.G; b = $Color.B; a = $Color.A }
}

$executable = 'F:\workspace\yonder\target\debug\yonder-desktop.exe'
if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) { throw 'Windows desktop executable not found' }
$existing = @(Get-CimInstance Win32_Process -Filter "Name='yonder-desktop.exe'" | Where-Object { $_.ExecutablePath -eq $executable })
if ($existing.Count -gt 0) { throw "检测到现有Yonda进程，拒绝干扰：$($existing.ProcessId -join ',')" }

$backgroundColor = [Drawing.Color]::FromArgb(255, 220, 230, 242)
$backdrop = New-Object Windows.Forms.Form
$backdrop.FormBorderStyle = [Windows.Forms.FormBorderStyle]::None
$backdrop.StartPosition = [Windows.Forms.FormStartPosition]::Manual
$backdrop.Bounds = New-Object Drawing.Rectangle(520, 280, 600, 600)
$backdrop.BackColor = $backgroundColor
$backdrop.ShowInTaskbar = $false
$backdrop.Text = 'Yonda controlled evidence background'

$testRoot = Join-Path $env:TEMP ('yonda-ds-s1-visual-' + [guid]::NewGuid().ToString('N'))
$roaming = Join-Path $testRoot 'Roaming'
$local = Join-Path $testRoot 'Local'
New-Item -ItemType Directory -Path $roaming, $local | Out-Null

$process = $null
$bitmap = $null
$graphics = $null
$result = [ordered]@{ platform = 'windows'; background = Convert-Color $backgroundColor }
try {
    $backdrop.Show()
    $backdrop.Refresh()
    [Windows.Forms.Application]::DoEvents()

    $start = New-Object Diagnostics.ProcessStartInfo
    $start.FileName = $executable
    $start.WorkingDirectory = Split-Path -Parent $executable
    $start.UseShellExecute = $false
    $start.RedirectStandardError = $true
    $start.EnvironmentVariables['APPDATA'] = $roaming
    $start.EnvironmentVariables['LOCALAPPDATA'] = $local
    $process = [Diagnostics.Process]::Start($start)
    $result.process_id = $process.Id

    $watch = [Diagnostics.Stopwatch]::StartNew()
    $rect = $null
    while ($watch.Elapsed.TotalSeconds -lt 30 -and -not $rect) {
        if ($process.HasExited) { throw "Yonda提前退出，code=$($process.ExitCode)" }
        Start-Sleep -Milliseconds 250
        $rect = [YondaVisualProbe]::FindPet($process.Id)
    }
    if (-not $rect) { throw '30秒内没有找到200×200的Yonda原生窗口' }
    Start-Sleep -Seconds 5
    if (-not [YondaVisualProbe]::MovePetWithoutActivation($process.Id, 640, 400)) { throw '无法把Yonda移到受控背景' }
    Start-Sleep -Seconds 2
    $rect = [YondaVisualProbe]::FindPet($process.Id)
    if (-not $rect -or $rect[0] -ne 640 -or $rect[1] -ne 400) { throw 'Yonda没有稳定在截图位置' }

    $dpi = [YondaVisualProbe]::PetDpi($process.Id)
    if ($dpi -lt 96) { throw '无法读取Yonda窗口DPI' }
    $scale = $dpi / 96.0
    $physicalX = [int][Math]::Round($rect[0] * $scale)
    $physicalY = [int][Math]::Round($rect[1] * $scale)
    $physicalWidth = [int][Math]::Round($rect[2] * $scale)
    $physicalHeight = [int][Math]::Round($rect[3] * $scale)
    $bitmap = New-Object Drawing.Bitmap($physicalWidth, $physicalHeight, [Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    $graphics.CopyFromScreen($physicalX, $physicalY, 0, 0, (New-Object Drawing.Size($physicalWidth, $physicalHeight)), [Drawing.CopyPixelOperation]::SourceCopy)
    $imagePath = Join-Path $PSScriptRoot 'native-visual.png'
    $bitmap.Save($imagePath, [Drawing.Imaging.ImageFormat]::Png)

    $backgroundPixels = 0
    $contentPixels = 0
    for ($y = 0; $y -lt $physicalHeight; $y++) {
        for ($x = 0; $x -lt $physicalWidth; $x++) {
            $pixel = $bitmap.GetPixel($x, $y)
            $difference = [Math]::Abs([int]$pixel.R - $backgroundColor.R) + [Math]::Abs([int]$pixel.G - $backgroundColor.G) + [Math]::Abs([int]$pixel.B - $backgroundColor.B)
            if ($difference -le 6) { $backgroundPixels++ } else { $contentPixels++ }
        }
    }
    $corners = @(
        $bitmap.GetPixel(0, 0), $bitmap.GetPixel($physicalWidth - 1, 0),
        $bitmap.GetPixel(0, $physicalHeight - 1), $bitmap.GetPixel($physicalWidth - 1, $physicalHeight - 1)
    )
    $transparentCorners = @($corners | Where-Object {
        [Math]::Abs([int]$_.R - $backgroundColor.R) + [Math]::Abs([int]$_.G - $backgroundColor.G) + [Math]::Abs([int]$_.B - $backgroundColor.B) -le 6
    }).Count

    $result.window = [ordered]@{ x = $rect[0]; y = $rect[1]; width = $rect[2]; height = $rect[3] }
    $result.dpi = $dpi
    $result.scale_factor = $scale
    $result.physical_capture = [ordered]@{ x = $physicalX; y = $physicalY; width = $physicalWidth; height = $physicalHeight }
    $result.image = 'native-visual.png'
    $result.background_pixels = $backgroundPixels
    $result.content_pixels = $contentPixels
    $result.transparent_corner_count = $transparentCorners
    $result.corners = @($corners | ForEach-Object { Convert-Color $_ })
    $result.window_size_passed = $rect[2] -eq 200 -and $rect[3] -eq 200
    $pixelCount = $physicalWidth * $physicalHeight
    $result.transparent_edges_passed = $transparentCorners -eq 4 -and $backgroundPixels -ge [int]($pixelCount * 0.1)
    $result.visible_content_passed = $contentPixels -ge [int]($pixelCount * 0.025) -and $contentPixels -lt [int]($pixelCount * 0.9)
    $result.passed = $result.window_size_passed -and $result.transparent_edges_passed -and $result.visible_content_passed
    if (-not $result.passed) { throw '原生视觉像素门禁未通过' }
} catch {
    $result.passed = $false
    $result.error = $_.Exception.Message
    throw
} finally {
    if ($graphics) { $graphics.Dispose() }
    if ($bitmap) { $bitmap.Dispose() }
    if ($process -and -not $process.HasExited) { $process.Kill(); $process.WaitForExit() }
    if ($process) { $result.diagnostic_log = @($process.StandardError.ReadToEnd() -split "`r?`n" | Where-Object { $_ }) }
    $backdrop.Close()
    $backdrop.Dispose()
    $result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'native-visual-result.json') -Encoding UTF8
    if (Test-Path -LiteralPath $testRoot) { Remove-Item -LiteralPath $testRoot -Recurse -Force }
}

$result | ConvertTo-Json -Depth 8 -Compress
