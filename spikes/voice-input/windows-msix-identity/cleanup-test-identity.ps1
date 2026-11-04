$ErrorActionPreference = 'Stop'

$expectedPackageName = 'Yonder.VoiceSpike.Identity'
$expectedPublisher = 'CN=AppModelSamples, OID.2.25.311729368913984317654407730594956997722=1'
$outputDirectory = Join-Path $env:LOCALAPPDATA 'Yonder\Spikes\VoiceMsixIdentity\0.1.0.0'
$statePath = Join-Path $outputDirectory 'registration-state.json'
if (-not (Test-Path -LiteralPath $statePath)) { throw '缺少精确清理所需的注册状态文件。' }

$state = Get-Content -LiteralPath $statePath -Raw | ConvertFrom-Json
if ($state.package_name -ne $expectedPackageName -or $state.publisher -ne $expectedPublisher) {
  throw '注册状态与预期测试身份不匹配，拒绝清理。'
}
$package = Get-AppxPackage -Name $expectedPackageName -ErrorAction SilentlyContinue |
  Where-Object { $_.PackageFullName -eq $state.package_full_name }
if ($package) { Remove-AppxPackage -Package $package.PackageFullName }

$remainingPackage = @(Get-AppxPackage -Name $expectedPackageName -ErrorAction SilentlyContinue).Count
if ($remainingPackage -ne 0) {
  throw '测试身份清理后仍有残留。'
}
Remove-Item -LiteralPath $statePath

[ordered]@{
  cleaned = $true
  package_removed = $true
  certificate_removal_required = $false
  remaining_package_count = 0
  trust_store_changed = $false
} | ConvertTo-Json -Compress
