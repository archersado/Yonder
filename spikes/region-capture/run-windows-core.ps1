param(
    [string]$EvidencePath = (Join-Path $PSScriptRoot 'evidence/windows-20261104/result.json')
)

$ErrorActionPreference = 'Stop'
$repository = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$buildScript = Join-Path $PSScriptRoot 'build-windows.ps1'
$probe = Join-Path $env:TEMP 'yonder-region-capture-spike/windows-probe.exe'

& $buildScript | Out-Host
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $probe)) {
    throw 'probe-build-failed'
}

$runs = @()
foreach ($attempt in 1..3) {
    $payload = & $probe | Out-String
    if ($LASTEXITCODE -ne 0) { throw "probe-run-failed:$attempt" }
    $run = $payload | ConvertFrom-Json
    $run | Add-Member -NotePropertyName attempt -NotePropertyValue $attempt
    $runs += $run
}

$sourceFiles = @(
    'spikes/region-capture/windows-probe.cpp',
    'spikes/region-capture/build-windows.ps1',
    'spikes/region-capture/run-windows-core.ps1'
)
$sourceHashes = @($sourceFiles | ForEach-Object {
    $path = Join-Path $repository $_
    [ordered]@{ path = $_; sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash.ToLowerInvariant() }
})
$headFile = Join-Path $repository '.git/HEAD'
$headValue = [IO.File]::ReadAllText($headFile).Trim()
$head = if ($headValue.StartsWith('ref: ')) {
    [IO.File]::ReadAllText((Join-Path $repository ('.git/' + $headValue.Substring(5)))).Trim()
} else { $headValue }
$probeProcessRemaining = @(Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $probe }).Count -gt 0
$first = $runs[0]
$passed = @($runs | Where-Object { -not $_.passed }).Count -eq 0 `
    -and @($runs | Where-Object { $_.scale -ne 2 -or $_.pixels_width -ne 480 -or $_.pixels_height -ne 320 }).Count -eq 0 `
    -and -not $probeProcessRemaining
$evidence = [ordered]@{
    platform = $first.platform
    generated_at_utc = [DateTime]::UtcNow.ToString('o')
    command = 'powershell -ExecutionPolicy Bypass -File .\spikes\region-capture\run-windows-core.ps1'
    source_revision = $head
    source_state = 'working-tree'
    source_sha256 = $sourceHashes
    executable = "$env:TEMP\yonder-region-capture-spike\windows-probe.exe"
    executable_sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $probe).Hash.ToLowerInvariant()
    capture_api = $first.capture_api
    display_config_available = $first.display_config_available
    display_count = $first.display_count
    negative_origin_available = $first.negative_origin_available
    dpi = 192
    scale = $first.scale
    logical_width = $first.logical_width
    logical_height = $first.logical_height
    pixels_width = $first.pixels_width
    pixels_height = $first.pixels_height
    runs = $runs
    screenshot_persisted = $false
    input_listeners_installed = $false
    probe_process_remaining = $probeProcessRemaining
    passed = $passed
}

$folder = Split-Path -Parent $EvidencePath
New-Item -ItemType Directory -Force -Path $folder | Out-Null
$json = ($evidence | ConvertTo-Json -Depth 8) -replace "`r`n", "`n"
[System.IO.File]::WriteAllText($EvidencePath, $json + "`n", [System.Text.UTF8Encoding]::new($false))
$json
if (-not $passed) { exit 2 }
