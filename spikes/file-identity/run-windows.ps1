param(
    [string]$OutputPath = (Join-Path $PSScriptRoot 'evidence\windows\result.json')
)

$ErrorActionPreference = 'Stop'
$source = Join-Path $PSScriptRoot 'windows-probe.rs'
$executable = Join-Path $env:TEMP ("yonda-file-identity-{0}.exe" -f [guid]::NewGuid().ToString('N'))

try {
    & rustc --edition 2024 $source -o $executable
    if ($LASTEXITCODE -ne 0) { throw 'Windows文件身份探针编译失败' }
    $result = & $executable
    if ($LASTEXITCODE -ne 0) { throw "Windows文件身份探针失败：$result" }
    $parsed = $result | ConvertFrom-Json
    if (-not $parsed.passed) { throw 'Windows文件身份探针未通过' }
    $directory = Split-Path -Parent $OutputPath
    [IO.Directory]::CreateDirectory($directory) | Out-Null
    [IO.File]::WriteAllText($OutputPath, "$result`n", [Text.UTF8Encoding]::new($false))
    $result
} finally {
    if (Test-Path -LiteralPath $executable) { Remove-Item -LiteralPath $executable -Force }
}
