$ErrorActionPreference = 'Stop'

$principal = [Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
  throw '该脚本必须在用户明确授权的管理员PowerShell中运行。'
}

$packageName = 'Yonder.VoiceSpike.Identity'
$publisher = 'CN=Yonder Voice Spike Dev'
$outputDirectory = Join-Path $env:LOCALAPPDATA 'Yonder\Spikes\VoiceMsixIdentity\0.1.0.0'
$packagePath = Join-Path $outputDirectory 'Yonder.VoiceSpike.Identity_0.1.0.0_neutral.msix'
$executable = Join-Path $outputDirectory 'windows-explicit-probe.exe'
$publicCertificate = Join-Path $outputDirectory 'Yonder.VoiceSpike.Identity.machine-trusted.cer'
$statePath = Join-Path $outputDirectory 'machine-registration-state.json'

if (-not (Test-Path -LiteralPath $packagePath) -or -not (Test-Path -LiteralPath $executable)) {
  throw '请先使用签名Publisher清单重新运行build-unsigned.ps1。'
}
if (Test-Path -LiteralPath $statePath) { throw '机器信任注册状态文件已存在，拒绝覆盖。' }
if (@(Get-AppxPackage -Name $packageName -ErrorAction SilentlyContinue).Count -ne 0) {
  throw '同名测试包已注册，拒绝覆盖。'
}
$existingPersonal = @(Get-ChildItem Cert:\CurrentUser\My -ErrorAction SilentlyContinue | Where-Object { $_.Subject -eq $publisher })
$existingMachineTrusted = @(Get-ChildItem Cert:\LocalMachine\TrustedPeople -ErrorAction SilentlyContinue | Where-Object { $_.Subject -eq $publisher })
if ($existingPersonal.Count -ne 0 -or $existingMachineTrusted.Count -ne 0) {
  throw '同Subject证书已存在，拒绝覆盖。'
}

$kitsRoot = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin'
$sdk = Get-ChildItem -LiteralPath $kitsRoot -Directory |
  Where-Object { $_.Name -match '^\d+\.\d+\.\d+\.\d+$' } |
  Sort-Object { [version]$_.Name } -Descending |
  Select-Object -First 1
if (-not $sdk) { throw '未找到版本化Windows SDK工具目录。' }
$signTool = Join-Path $sdk.FullName 'x64\signtool.exe'
if (-not (Test-Path -LiteralPath $signTool)) { throw '未找到SignTool.exe。' }

$certificate = $null
$registeredPackage = $null
try {
  $certificate = New-SelfSignedCertificate `
    -Type CodeSigningCert `
    -Subject $publisher `
    -FriendlyName 'Yonder Voice Spike temporary machine-trusted certificate' `
    -CertStoreLocation Cert:\CurrentUser\My `
    -KeyAlgorithm RSA `
    -KeyLength 2048 `
    -HashAlgorithm SHA256 `
    -KeyExportPolicy NonExportable `
    -NotAfter (Get-Date).AddDays(2)

  Export-Certificate -Cert $certificate -FilePath $publicCertificate -Force | Out-Null
  Import-Certificate -FilePath $publicCertificate -CertStoreLocation Cert:\LocalMachine\TrustedPeople | Out-Null
  & $signTool sign /fd SHA256 /sha1 $certificate.Thumbprint /s My $packagePath
  if ($LASTEXITCODE -ne 0) { throw "身份包签名失败，退出码：$LASTEXITCODE" }
  $signature = Get-AuthenticodeSignature -FilePath $packagePath
  if (-not $signature.SignerCertificate -or $signature.SignerCertificate.Thumbprint -ne $certificate.Thumbprint) {
    throw '身份包签名证书与本次测试证书不匹配。'
  }

  Add-AppxPackage -Path $packagePath -ExternalLocation $outputDirectory
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
    certificate_thumbprint = $certificate.Thumbprint
    certificate_private_store = 'CurrentUser\My'
    certificate_trust_store = 'LocalMachine\TrustedPeople'
    external_location = $outputDirectory
    registered_at_utc = [DateTime]::UtcNow.ToString('o')
  }
  [IO.File]::WriteAllText($statePath, ($state | ConvertTo-Json), [Text.UTF8Encoding]::new($false))
  [ordered]@{
    installed = $true
    package_full_name = $registeredPackage.PackageFullName
    certificate_thumbprint = $certificate.Thumbprint
    private_key_exportable = $false
    machine_trusted_people = $true
    machine_root_changed = $false
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
  if ($certificate) {
    $thumbprint = $certificate.Thumbprint
    Remove-Item -LiteralPath "Cert:\LocalMachine\TrustedPeople\$thumbprint" -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath "Cert:\CurrentUser\My\$thumbprint" -ErrorAction SilentlyContinue
  }
  throw
}
