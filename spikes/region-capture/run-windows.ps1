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
    [DllImport("user32.dll")] public static extern IntPtr MonitorFromPoint(Point point, uint flags);
    [DllImport("shcore.dll")] public static extern int GetDpiForMonitor(IntPtr monitor, int type, out uint x, out uint y);
    [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr value);
    public static uint[] At(int x, int y) {
        uint dx, dy;
        var monitor = MonitorFromPoint(new Point { X = x, Y = y }, 2);
        if (monitor == IntPtr.Zero || GetDpiForMonitor(monitor, 0, out dx, out dy) != 0) throw new InvalidOperationException("dpi-unavailable");
        return new [] { dx, dy };
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
    if ($dpi[0] -ne 96 -or $dpi[1] -ne 96) {
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
            $eventArgs.Graphics.FillRectangle([System.Drawing.Brushes]::Red, 0, 0, 120, 80)
            $eventArgs.Graphics.FillRectangle([System.Drawing.Brushes]::Lime, 120, 0, 120, 80)
            $eventArgs.Graphics.FillRectangle([System.Drawing.Brushes]::Blue, 0, 80, 120, 80)
            $eventArgs.Graphics.FillRectangle([System.Drawing.Brushes]::Yellow, 120, 80, 120, 80)
        })
        $form.Show()
        [System.Windows.Forms.Application]::DoEvents()
        Start-Sleep -Milliseconds 400
        $runs = @()
        foreach ($attempt in 1..3) {
            $payload = & $probe $form.Left $form.Top $form.ClientSize.Width $form.ClientSize.Height | Out-String
            $run = $payload | ConvertFrom-Json
            $run | Add-Member -NotePropertyName attempt -NotePropertyValue $attempt
            $runs += $run
        }
        $results += [ordered]@{
            passed = @($runs | Where-Object { -not $_.passed }).Count -eq 0
            display_bounds = [ordered]@{ x = $bounds.Left; y = $bounds.Top; width = $bounds.Width; height = $bounds.Height }
            dpi_x = $dpi[0]
            dpi_y = $dpi[1]
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
$evidence = [ordered]@{
    passed = $tested.Count -gt 0 -and $negativeOriginAvailable -and $negativeTested.Count -gt 0 -and @($results | Where-Object { -not $_.passed }).Count -eq 0 -and -not $probeProcessRemaining
    generated_at_utc = [DateTime]::UtcNow.ToString('o')
    command = 'powershell -File spikes/region-capture/run-windows.ps1'
    source_revision = $sourceRevision
    source_state = 'working-tree'
    windows = [ordered]@{ caption = $os.Caption; version = $os.Version; build = $os.BuildNumber }
    graphics = $gpus
    route = 'Windows Graphics Capture CreateForMonitor'
    display_count = [System.Windows.Forms.Screen]::AllScreens.Count
    negative_origin_available = $negativeOriginAvailable
    negative_origin_tested = $negativeTested.Count -gt 0
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
