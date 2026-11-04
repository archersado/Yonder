$ErrorActionPreference = 'Stop'

$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
if (-not (Test-Path -LiteralPath $vswhere)) { throw '未找到Visual Studio Installer。' }
$installationPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $installationPath) { throw '未找到MSVC C++工具链。' }
$kitsRoot = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin'
$sdk = Get-ChildItem -LiteralPath $kitsRoot -Directory | Where-Object { $_.Name -match '^\d+\.\d+\.\d+\.\d+$' } | Sort-Object { [version]$_.Name } -Descending | Select-Object -First 1
if (-not $sdk) { throw '未找到版本化Windows SDK工具目录。' }
$makeAppx = Join-Path $sdk.FullName 'x64\makeappx.exe'
$mt = Join-Path $sdk.FullName 'x64\mt.exe'

$source = Join-Path (Split-Path $PSScriptRoot -Parent) 'windows-explicit-probe.cpp'
$applicationManifest = Join-Path $PSScriptRoot 'probe.machine-trusted.manifest'
$packageManifest = Join-Path $PSScriptRoot 'AppxManifest.machine-trusted.xml'
$outputDirectory = Join-Path $env:LOCALAPPDATA 'Yonder\Spikes\VoiceMsixIdentity\0.1.0.0'
$layout = Join-Path $outputDirectory 'identity-layout'
$executable = Join-Path $outputDirectory 'windows-explicit-probe.exe'
$object = Join-Path $outputDirectory 'windows-explicit-probe.obj'
$embeddedManifest = Join-Path $outputDirectory 'embedded-manifest.xml'
$package = Join-Path $outputDirectory 'Yonder.VoiceSpike.Identity_0.1.0.0_neutral.msix'
New-Item -ItemType Directory -Force -Path $outputDirectory,$layout | Out-Null
Copy-Item -LiteralPath $packageManifest -Destination (Join-Path $layout 'AppxManifest.xml') -Force

$developerShell = Join-Path $installationPath 'Common7\Tools\VsDevCmd.bat'
$compile = 'cl.exe /nologo /std:c++20 /EHsc /O2 /W4 /DUNICODE /D_UNICODE "{0}" /Fo:"{1}" /Fe:"{2}" windowsapp.lib /link /MANIFEST:EMBED /MANIFESTINPUT:"{3}"' -f $source,$object,$executable,$applicationManifest
$commandLine = 'call "{0}" -no_logo -arch=x64 -host_arch=x64 && {1}' -f $developerShell,$compile
& $env:ComSpec /d /c $commandLine
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $executable)) { throw "机器信任探针编译失败，退出码：$LASTEXITCODE" }
& $mt -nologo "-inputresource:$executable;#1" "-out:$embeddedManifest"
if ($LASTEXITCODE -ne 0) { throw "内嵌清单提取失败，退出码：$LASTEXITCODE" }
$embedded = Get-Content -LiteralPath $embeddedManifest -Raw
foreach ($expected in @('Yonder.VoiceSpike.Identity','CN=Yonder Voice Spike Dev','VoiceProbe')) {
  if (-not $embedded.Contains($expected)) { throw "内嵌机器信任元数据缺少：$expected" }
}
& $makeAppx pack /o /d $layout /nv /p $package
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $package)) { throw "机器信任身份包构建失败，退出码：$LASTEXITCODE" }
[ordered]@{
  built = $true
  signed = $false
  publisher = 'CN=Yonder Voice Spike Dev'
  embedded_identity_metadata = $true
  package_registered = $false
  machine_trust_changed = $false
  executable = $executable
  package = $package
} | ConvertTo-Json -Compress
