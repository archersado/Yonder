$ErrorActionPreference = 'Stop'

$principal = [Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
  throw '该清理脚本必须在管理员PowerShell中运行。'
}
$expectedPackageName = 'Yonder.VoiceSpike.Identity'
$expectedPublisher = 'CN=Yonder Voice Spike Dev'
$outputDirectory = Join-Path $env:LOCALAPPDATA 'Yonder\Spikes\VoiceMsixIdentity\0.1.0.0'
$statePath = Join-Path $outputDirectory 'machine-registration-state.json'
$publicCertificate = Join-Path $outputDirectory 'Yonder.VoiceSpike.Identity.machine-trusted.cer'
if (-not (Test-Path -LiteralPath $statePath)) { throw '缺少机器信任精确清理所需的状态文件。' }
$state = Get-Content -LiteralPath $statePath -Raw | ConvertFrom-Json
if ($state.package_name -ne $expectedPackageName -or $state.publisher -ne $expectedPublisher) {
  throw '注册状态与预期测试身份不匹配，拒绝清理。'
}
$thumbprint = [string]$state.certificate_thumbprint
if ($thumbprint -notmatch '^[0-9A-Fa-f]{40}$') { throw '证书指纹格式无效，拒绝清理。' }

$package = Get-AppxPackage -Name $expectedPackageName -ErrorAction SilentlyContinue |
  Where-Object { $_.PackageFullName -eq $state.package_full_name }
if ($package) { Remove-AppxPackage -Package $package.PackageFullName }
Remove-Item -LiteralPath "Cert:\LocalMachine\TrustedPeople\$thumbprint" -ErrorAction SilentlyContinue
Remove-Item -LiteralPath "Cert:\CurrentUser\My\$thumbprint" -ErrorAction SilentlyContinue
if (Test-Path -LiteralPath $publicCertificate) { Remove-Item -LiteralPath $publicCertificate }

$remainingPackage = @(Get-AppxPackage -Name $expectedPackageName -ErrorAction SilentlyContinue).Count
$remainingMachineTrusted = @(Get-ChildItem Cert:\LocalMachine\TrustedPeople -ErrorAction SilentlyContinue | Where-Object { $_.Thumbprint -eq $thumbprint }).Count
$remainingPersonal = @(Get-ChildItem Cert:\CurrentUser\My -ErrorAction SilentlyContinue | Where-Object { $_.Thumbprint -eq $thumbprint }).Count
if ($remainingPackage -ne 0 -or $remainingMachineTrusted -ne 0 -or $remainingPersonal -ne 0) {
  throw '机器信任测试身份清理后仍有残留。'
}
Remove-Item -LiteralPath $statePath
[ordered]@{
  cleaned = $true
  package_removed = $true
  machine_trusted_certificate_removed = $true
  private_certificate_removed = $true
  machine_root_changed = $false
  remaining_count = 0
} | ConvertTo-Json -Compress
