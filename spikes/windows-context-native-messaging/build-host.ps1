param([Parameter(Mandatory=$true)][string]$OutputPath)
$ErrorActionPreference = 'Stop'
$source = Get-Content -Raw -LiteralPath (Join-Path $PSScriptRoot 'native-host.cs')
New-Item -ItemType Directory -Force -Path (Split-Path $OutputPath) | Out-Null
Add-Type -TypeDefinition $source -OutputAssembly $OutputPath -OutputType ConsoleApplication
Write-Output $OutputPath
