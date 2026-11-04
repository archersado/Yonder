$ErrorActionPreference = 'Stop'

$installRoot = 'C:\Program Files\Citro Labs\ego lite\Application'
$activeDirectory = Join-Path $env:USERPROFILE '.local\share\ego\active_version_dir'
$activeAlias = Join-Path $activeDirectory 'ego-browser.exe'
$canonicalRoot = [IO.Path]::GetFullPath((Get-Item -LiteralPath $installRoot).FullName)
$activeTargetDirectory = @((Get-Item -LiteralPath $activeDirectory -Force).Target)[0]
$canonicalActive = [IO.Path]::GetFullPath((Join-Path $activeTargetDirectory 'ego-browser.exe'))
$installed = @(Get-ChildItem -LiteralPath $installRoot -Directory | ForEach-Object {
    $parsed = $null
    if ([version]::TryParse($_.Name, [ref]$parsed) -and (Test-Path -LiteralPath (Join-Path $_.FullName 'ego-browser.exe') -PathType Leaf)) {
        [ordered]@{version=$_.Name;path=(Join-Path $_.FullName 'ego-browser.exe')}
    }
} | Sort-Object { [version]$_.version })
$activeVersion = Split-Path -Leaf (Split-Path -Parent $canonicalActive)
$highestVersion = $installed[-1].version
$insideRoot = $canonicalActive.StartsWith($canonicalRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)
$result = [ordered]@{
    platform = 'windows'
    active_alias = $activeAlias
    active_canonical_path = $canonicalActive
    active_version = $activeVersion
    installed_versions = @($installed.version)
    highest_installed_version = $highestVersion
    active_target_inside_fixed_root = $insideRoot
    active_precedes_higher_unactivated_version = ([version]$highestVersion -gt [version]$activeVersion)
    cua_executed = $false
}
$result.passed = $insideRoot -and (Split-Path -Leaf $canonicalActive) -eq 'ego-browser.exe' -and $result.active_precedes_higher_unactivated_version
if (-not $result.passed) { throw '当前机器未形成active版本优先的多版本验证样本' }
$result | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'ego-runtime-selection-result.json') -Encoding UTF8
$result | ConvertTo-Json -Depth 6 -Compress
