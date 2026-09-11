$ErrorActionPreference = 'Stop'
$vcvars = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat'
$environment = & cmd.exe /d /s /c "`"$vcvars`" >nul && set"
foreach ($line in $environment) {
  if ($line -match '^([^=]+)=(.*)$') { Set-Item -Path "env:$($matches[1])" -Value $matches[2] }
}
$env:Path = 'C:\Strawberry\perl\bin;' + $env:Path
$directory = Join-Path $env:LOCALAPPDATA 'Yonder\encrypted-storage-spike'
New-Item -ItemType Directory -Force (Join-Path $directory 'src') | Out-Null
Copy-Item -Force (Join-Path $PSScriptRoot 'Cargo.toml') $directory
Copy-Item -Force (Join-Path $PSScriptRoot 'src\main.rs') (Join-Path $directory 'src')
Set-Location $directory
& 'C:\Program Files\Rust stable MSVC 1.98\bin\cargo.exe' run --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
