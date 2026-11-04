$ErrorActionPreference = 'Stop'

$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
if (-not (Test-Path -LiteralPath $vswhere)) {
  throw '未找到 Visual Studio Installer (vswhere.exe)。'
}

$installationPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $installationPath) {
  throw '未找到已安装的 MSVC C++ 工具链。'
}

$source = Join-Path $PSScriptRoot 'windows-explicit-probe.cpp'
$outputDirectory = Join-Path $env:TEMP 'yonder-voice-spike'
$output = Join-Path $outputDirectory 'windows-explicit-probe.exe'
$object = Join-Path $outputDirectory 'windows-explicit-probe.obj'
New-Item -ItemType Directory -Force -Path $outputDirectory | Out-Null

$developerShell = Join-Path $installationPath 'Common7\Tools\VsDevCmd.bat'
$compileCommand = 'cl.exe /nologo /std:c++20 /EHsc /O2 /W4 /DUNICODE /D_UNICODE "{0}" /Fo:"{1}" /Fe:"{2}" windowsapp.lib' -f $source, $object, $output
$commandLine = 'call "{0}" -no_logo -arch=x64 -host_arch=x64 && {1}' -f $developerShell, $compileCommand
& $env:ComSpec /d /c $commandLine
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $output)) {
  throw "Windows显式语音探针编译失败，退出码：$LASTEXITCODE"
}

[ordered]@{
  built = $true
  executable = $output
  opens_microphone_during_build = $false
} | ConvertTo-Json -Compress
