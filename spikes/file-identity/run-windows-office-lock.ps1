param(
    [Parameter(Mandatory = $true)]
    [string]$DocumentPath,
    [Parameter(Mandatory = $true)]
    [ValidateSet('open', 'closed')]
    [string]$Expected,
    [ValidateSet('office', 'wps')]
    [string]$HostApplication = 'wps',
    [string]$OutputPath = (Join-Path $PSScriptRoot ("evidence\windows\{0}-lock-{1}.json" -f $HostApplication, $Expected))
)

$ErrorActionPreference = 'Stop'
$source = Join-Path $PSScriptRoot 'windows-office-lock-probe.rs'
$executable = Join-Path $env:TEMP ("yonda-office-lock-{0}.exe" -f [guid]::NewGuid().ToString('N'))

try {
    & rustc --edition 2024 $source -o $executable
    if ($LASTEXITCODE -ne 0) { throw 'Windows宿主锁探针编译失败' }
    $result = & $executable $DocumentPath $Expected $HostApplication
    if ($LASTEXITCODE -ne 0) { throw "Windows宿主锁探针失败：$result" }
    $parsed = $result | ConvertFrom-Json
    if (-not $parsed.passed) { throw 'Windows宿主锁探针未通过' }
    $directory = Split-Path -Parent $OutputPath
    [IO.Directory]::CreateDirectory($directory) | Out-Null
    [IO.File]::WriteAllText($OutputPath, "$result`n", [Text.UTF8Encoding]::new($false))
    $result
} finally {
    if (Test-Path -LiteralPath $executable) { Remove-Item -LiteralPath $executable -Force }
}
