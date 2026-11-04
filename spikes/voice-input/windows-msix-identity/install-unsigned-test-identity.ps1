$ErrorActionPreference = 'Stop'

$packageName = 'Yonder.VoiceSpike.Identity'
$publisher = 'CN=AppModelSamples, OID.2.25.311729368913984317654407730594956997722=1'
$outputDirectory = Join-Path $env:LOCALAPPDATA 'Yonder\Spikes\VoiceMsixIdentity\0.1.0.0'
$packagePath = Join-Path $outputDirectory 'Yonder.VoiceSpike.Identity_0.1.0.0_neutral.msix'
$executable = Join-Path $outputDirectory 'windows-explicit-probe.exe'
$statePath = Join-Path $outputDirectory 'registration-state.json'

if (-not (Test-Path -LiteralPath $packagePath) -or -not (Test-Path -LiteralPath $executable)) {
  throw '请先运行build-unsigned.ps1。'
}
if (Test-Path -LiteralPath $statePath) { throw '注册状态文件已存在，拒绝覆盖。' }
if (@(Get-AppxPackage -Name $packageName -ErrorAction SilentlyContinue).Count -ne 0) {
  throw '同名测试包已注册，拒绝覆盖。'
}

$registeredPackage = $null
try {
  Add-AppxPackage -Path $packagePath -ExternalLocation $outputDirectory -AllowUnsigned
  $registeredPackage = Get-AppxPackage -Name $packageName -ErrorAction Stop
  if (@($registeredPackage).Count -ne 1) { throw '注册后没有得到唯一测试包。' }

  $identityResult = & $executable --identity-only | ConvertFrom-Json
  if (-not $identityResult.package_identity_present -or $identityResult.opened_microphone) {
    throw '注册后运行时身份检查失败。'
  }

  $state = [ordered]@{
    package_name = $packageName
    package_full_name = $registeredPackage.PackageFullName
    package_family_name = $registeredPackage.PackageFamilyName
    publisher = $publisher
    install_mode = 'AllowUnsigned-current-user-external-location'
    external_location = $outputDirectory
    registered_at_utc = [DateTime]::UtcNow.ToString('o')
  }
  [IO.File]::WriteAllText($statePath, ($state | ConvertTo-Json), [Text.UTF8Encoding]::new($false))

  [ordered]@{
    installed = $true
    install_mode = 'AllowUnsigned-current-user-external-location'
    package_name = $packageName
    package_full_name = $registeredPackage.PackageFullName
    package_family_name_present = -not [string]::IsNullOrWhiteSpace($registeredPackage.PackageFamilyName)
    certificate_created = $false
    trust_store_changed = $false
    developer_mode_changed = $false
    package_identity_present = $true
    opened_microphone = $false
    state_file = $statePath
  } | ConvertTo-Json -Compress
} catch {
  if ($registeredPackage) {
    Remove-AppxPackage -Package $registeredPackage.PackageFullName -ErrorAction SilentlyContinue
  } else {
    Get-AppxPackage -Name $packageName -ErrorAction SilentlyContinue |
      ForEach-Object { Remove-AppxPackage -Package $_.PackageFullName -ErrorAction SilentlyContinue }
  }
  throw
}
