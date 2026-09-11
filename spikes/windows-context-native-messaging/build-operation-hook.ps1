param([Parameter(Mandatory=$true)][string]$OutputPath)
$ErrorActionPreference = 'Stop'
$source = Get-Content -Raw -LiteralPath (Join-Path $PSScriptRoot 'operation-hook.cs')
New-Item -ItemType Directory -Force -Path (Split-Path $OutputPath) | Out-Null
Add-Type -TypeDefinition $source -OutputAssembly $OutputPath -OutputType ConsoleApplication
Write-Output $OutputPath
