param(
    [string]$EvidencePath = (Join-Path $PSScriptRoot 'evidence/windows/result.json')
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class YonderMonitorDpi {
    [StructLayout(LayoutKind.Sequential)] public struct Point { public int X; public int Y; }
    [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left; public int Top; public int Right; public int Bottom; }
    [DllImport("user32.dll")] public static extern IntPtr MonitorFromPoint(Point point, uint flags);
    [DllImport("shcore.dll")] public static extern int GetDpiForMonitor(IntPtr monitor, int type, out uint x, out uint y);
    [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr value);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr window, out Rect rect);
    [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr window, ref Point point);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr window, IntPtr insertAfter, int x, int y, int width, int height, uint flags);
    public static uint[] At(int x, int y) {
        uint dx, dy;
        var monitor = MonitorFromPoint(new Point { X = x, Y = y }, 2);
        if (monitor == IntPtr.Zero || GetDpiForMonitor(monitor, 0, out dx, out dy) != 0) throw new InvalidOperationException("dpi-unavailable");
        return new [] { dx, dy };
    }
    public static int[] ClientRect(IntPtr window) {
        Rect rect;
        var origin = new Point { X = 0, Y = 0 };
        if (!GetClientRect(window, out rect) || !ClientToScreen(window, ref origin)) throw new InvalidOperationException("client-rect-unavailable");
        return new [] { origin.X, origin.Y, rect.Right - rect.Left, rect.Bottom - rect.Top };
    }
    public static void SetPhysicalBounds(IntPtr window, int x, int y, int width, int height) {
        if (!SetWindowPos(window, IntPtr.Zero, x, y, width, height, 0x0014)) throw new InvalidOperationException("window-bounds-unavailable");
    }
}
'@
if (-not [YonderMonitorDpi]::SetProcessDpiAwarenessContext([IntPtr](-4))) { throw 'dpi-awareness-failed' }

$manifest = Join-Path $PSScriptRoot 'windows-probe/Cargo.toml'
$cargo = Join-Path $env:USERPROFILE '.cargo/bin/cargo.exe'
if (-not (Test-Path $cargo)) { throw 'cargo-unavailable' }
& $cargo build --offline --manifest-path $manifest
if ($LASTEXITCODE -ne 0) { throw 'probe-build-failed' }
$probe = Join-Path $PSScriptRoot 'windows-probe/target/debug/yonder-region-capture-windows-probe.exe'

$results = @()
foreach ($screen in [System.Windows.Forms.Screen]::AllScreens) {
    $bounds = $screen.Bounds
    $centerX = $bounds.Left + [int]($bounds.Width / 2)
    $centerY = $bounds.Top + [int]($bounds.Height / 2)
    $dpi = [YonderMonitorDpi]::At($centerX, $centerY)
    $supportedDpi = ($dpi[0] -eq $dpi[1]) -and ($dpi[0] -eq 96 -or $dpi[0] -eq 192)
    if (-not $supportedDpi) {
        $payload = & $probe $centerX $centerY 12 12 | Out-String
        $run = $payload | ConvertFrom-Json
        $results += [ordered]@{
            passed = (-not $run.passed) -and $run.error -eq 'capture-unsupported-scale'
            unsupported_scale = $true
            error = $run.error
            display_bounds = [ordered]@{ x = $bounds.Left; y = $bounds.Top; width = $bounds.Width; height = $bounds.Height }
            dpi_x = $dpi[0]
            dpi_y = $dpi[1]
            capture_returned = $null -ne $run.capture
        }
        continue
    }

    $form = New-Object System.Windows.Forms.Form
    try {
        $form.FormBorderStyle = [System.Windows.Forms.FormBorderStyle]::None
        $form.StartPosition = [System.Windows.Forms.FormStartPosition]::Manual
        $form.ClientSize = New-Object System.Drawing.Size(240, 160)
        $form.Location = New-Object System.Drawing.Point(
            ($screen.WorkingArea.Left + [int](($screen.WorkingArea.Width - 240) / 2)),
            ($screen.WorkingArea.Top + [int](($screen.WorkingArea.Height - 160) / 2))
        )
        $form.TopMost = $true
        $form.ShowInTaskbar = $false
        $form.Add_Paint({
            param($sender, $eventArgs)
            $halfWidth = [int]($sender.ClientSize.Width / 2)
            $halfHeight = [int]($sender.ClientSize.Height / 2)
            $eventArgs.Graphics.FillRectangle([System.Drawing.Brushes]::Red, 0, 0, $halfWidth, $halfHeight)
            $eventArgs.Graphics.FillRectangle([System.Drawing.Brushes]::Lime, $halfWidth, 0, $sender.ClientSize.Width - $halfWidth, $halfHeight)
            $eventArgs.Graphics.FillRectangle([System.Drawing.Brushes]::Blue, 0, $halfHeight, $halfWidth, $sender.ClientSize.Height - $halfHeight)
            $eventArgs.Graphics.FillRectangle([System.Drawing.Brushes]::Yellow, $halfWidth, $halfHeight, $sender.ClientSize.Width - $halfWidth, $sender.ClientSize.Height - $halfHeight)
        })
        $form.Show()
        [System.Windows.Forms.Application]::DoEvents()
        $physicalWidth = [int](240 * $dpi[0] / 96)
        $physicalHeight = [int](160 * $dpi[1] / 96)
        $physicalX = $screen.WorkingArea.Left + [int](($screen.WorkingArea.Width - $physicalWidth) / 2)
        $physicalY = $screen.WorkingArea.Top + [int](($screen.WorkingArea.Height - $physicalHeight) / 2)
        [YonderMonitorDpi]::SetPhysicalBounds($form.Handle, $physicalX, $physicalY, $physicalWidth, $physicalHeight)
        $form.Invalidate()
        [System.Windows.Forms.Application]::DoEvents()
        Start-Sleep -Milliseconds 400
        $client = [YonderMonitorDpi]::ClientRect($form.Handle)
        $runs = @()
        foreach ($attempt in 1..3) {
            $currentClient = [YonderMonitorDpi]::ClientRect($form.Handle)
            $currentCenter = New-Object System.Drawing.Point(
                ($currentClient[0] + [int]($currentClient[2] / 2)),
                ($currentClient[1] + [int]($currentClient[3] / 2))
            )
            $currentScreen = [System.Windows.Forms.Screen]::FromPoint($currentCenter)
            $currentDpi = [YonderMonitorDpi]::At($currentCenter.X, $currentCenter.Y)
            $sameScreen = $currentScreen.DeviceName -eq $screen.DeviceName `
                -and $currentScreen.Bounds.X -eq $bounds.X `
                -and $currentScreen.Bounds.Y -eq $bounds.Y `
                -and $currentScreen.Bounds.Width -eq $bounds.Width `
                -and $currentScreen.Bounds.Height -eq $bounds.Height
            $sameDpi = $currentDpi[0] -eq $dpi[0] -and $currentDpi[1] -eq $dpi[1]
            $sameClient = (@($currentClient) -join ',') -eq (@($client) -join ',')
            if (-not $sameScreen -or -not $sameDpi -or -not $sameClient) {
                throw 'display-topology-changed'
            }
            $payload = & $probe $currentClient[0] $currentClient[1] $currentClient[2] $currentClient[3] | Out-String
            $run = $payload | ConvertFrom-Json
            $run | Add-Member -NotePropertyName attempt -NotePropertyValue $attempt
            $runs += $run
        }
        $results += [ordered]@{
            passed = $client[2] -eq $physicalWidth -and $client[3] -eq $physicalHeight -and @($runs | Where-Object { -not $_.passed }).Count -eq 0
            display_bounds = [ordered]@{ x = $bounds.Left; y = $bounds.Top; width = $bounds.Width; height = $bounds.Height }
            dpi_x = $dpi[0]
            dpi_y = $dpi[1]
            logical_size = [ordered]@{ width = 240; height = 160 }
            expected_physical_size = [ordered]@{ width = $physicalWidth; height = $physicalHeight }
            runs = $runs
        }
    } finally {
        $form.Close()
        $form.Dispose()
        [System.Windows.Forms.Application]::DoEvents()
    }
}

$tested = @($results | Where-Object { -not $_.unsupported_scale })
$negativeTested = @($tested | Where-Object { $_.display_bounds.x -lt 0 -or $_.display_bounds.y -lt 0 })
$negativeOriginAvailable = @([System.Windows.Forms.Screen]::AllScreens | Where-Object { $_.Bounds.X -lt 0 -or $_.Bounds.Y -lt 0 }).Count -gt 0
$probeProcessRemaining = @(Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $probe }).Count -gt 0
$os = Get-CimInstance Win32_OperatingSystem
$gpus = @(Get-CimInstance Win32_VideoController | ForEach-Object { [ordered]@{ name = $_.Name; driver_version = $_.DriverVersion } })
$repository = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$head = [IO.File]::ReadAllText((Join-Path $repository '.git/HEAD')).Trim()
$sourceRevision = if ($head.StartsWith('ref: ')) {
    [IO.File]::ReadAllText((Join-Path $repository ('.git/' + $head.Substring(5)))).Trim()
} else { $head }
$verifiedDpi = @($tested | ForEach-Object { $_.dpi_x } | Sort-Object -Unique)
$sourceFiles = @(
    'apps/desktop/src/region_capture_windows.rs',
    'spikes/region-capture/windows-probe/Cargo.toml',
    'spikes/region-capture/windows-probe/src/main.rs',
    'spikes/region-capture/run-windows.ps1'
)
$sourceHashes = @($sourceFiles | ForEach-Object {
    $path = Join-Path $repository $_
    [ordered]@{ path = $_; sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash.ToLowerInvariant() }
})
$evidence = [ordered]@{
    passed = ($verifiedDpi -contains 96) -and ($verifiedDpi -contains 192) -and $negativeOriginAvailable -and $negativeTested.Count -gt 0 -and @($results | Where-Object { -not $_.passed }).Count -eq 0 -and -not $probeProcessRemaining
    generated_at_utc = [DateTime]::UtcNow.ToString('o')
    command = 'powershell -ExecutionPolicy Bypass -File .\spikes\region-capture\run-windows.ps1'
    source_revision = $sourceRevision
    source_state = 'working-tree'
    source_sha256 = $sourceHashes
    executable = 'spikes/region-capture/windows-probe/target/debug/yonder-region-capture-windows-probe.exe'
    executable_sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $probe).Hash.ToLowerInvariant()
    windows = [ordered]@{ caption = $os.Caption; version = $os.Version; build = $os.BuildNumber }
    graphics = $gpus
    route = 'Windows Graphics Capture CreateForMonitor'
    display_count = [System.Windows.Forms.Screen]::AllScreens.Count
    negative_origin_available = $negativeOriginAvailable
    negative_origin_tested = $negativeTested.Count -gt 0
    verified_dpi = $verifiedDpi
    results = $results
    screenshot_persisted = $false
    probe_process_remaining = $probeProcessRemaining
}

$folder = Split-Path -Parent $EvidencePath
New-Item -ItemType Directory -Force -Path $folder | Out-Null
$json = ($evidence | ConvertTo-Json -Depth 8) -replace "`r`n", "`n"
[System.IO.File]::WriteAllText($EvidencePath, $json + "`n", [System.Text.UTF8Encoding]::new($false))
$json
if (-not $evidence.passed) { exit 2 }
