$ErrorActionPreference = 'Stop'

$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
if (-not (Test-Path -LiteralPath $vswhere)) { throw 'visual-studio-installer-unavailable' }
$installationPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $installationPath) { throw 'msvc-toolchain-unavailable' }

$source = Join-Path $PSScriptRoot 'windows-probe.cpp'
$outputDirectory = Join-Path $env:TEMP 'yonder-region-capture-spike'
$output = Join-Path $outputDirectory 'windows-probe.exe'
$object = Join-Path $outputDirectory 'windows-probe.obj'
New-Item -ItemType Directory -Force -Path $outputDirectory | Out-Null

$developerShell = Join-Path $installationPath 'Common7\Tools\VsDevCmd.bat'
$compile = 'cl.exe /nologo /std:c++20 /EHsc /O2 /W4 /DUNICODE /D_UNICODE "{0}" /Fo:"{1}" /Fe:"{2}" windowsapp.lib d3d11.lib dxgi.lib user32.lib gdi32.lib dwmapi.lib' -f $source,$object,$output
$commandLine = 'call "{0}" -no_logo -arch=x64 -host_arch=x64 && {1}' -f $developerShell,$compile
& $env:ComSpec /d /c $commandLine
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $output)) { throw "region-capture-probe-build-failed:$LASTEXITCODE" }

[ordered]@{built=$true;executable=$output;capture_started_during_build=$false}|ConvertTo-Json -Compress
