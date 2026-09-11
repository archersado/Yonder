param(
  [Parameter(Mandatory=$true)][string]$HostPath,
  [ValidateSet('Chrome','Edge','Both')][string]$Browser = 'Both'
)
$extensionId = 'mofdddjaniddgalgegfdjegiegpneokc'
$directory = Join-Path $env:LOCALAPPDATA 'Yonder\context-spike'
$manifestPath = Join-Path $directory 'com.yonder.context_spike.json'
$extensionPath = Join-Path $directory 'extension'
New-Item -ItemType Directory -Force -Path $directory | Out-Null
Copy-Item -Recurse -Force -LiteralPath (Join-Path $PSScriptRoot 'extension') -Destination $directory
$json = [ordered]@{
  name = 'com.yonder.context_spike'
  description = 'Yonder Native Messaging 验证 Host'
  path = (Resolve-Path $HostPath).Path
  type = 'stdio'
  allowed_origins = @("chrome-extension://$extensionId/")
} | ConvertTo-Json
[IO.File]::WriteAllText($manifestPath, $json, [Text.UTF8Encoding]::new($false))
if ($Browser -in @('Chrome','Both')) {
  New-Item -Force 'HKCU:\Software\Google\Chrome\NativeMessagingHosts\com.yonder.context_spike' | Out-Null
  Set-Item 'HKCU:\Software\Google\Chrome\NativeMessagingHosts\com.yonder.context_spike' -Value $manifestPath
}
if ($Browser -in @('Edge','Both')) {
  New-Item -Force 'HKCU:\Software\Microsoft\Edge\NativeMessagingHosts\com.yonder.context_spike' | Out-Null
  Set-Item 'HKCU:\Software\Microsoft\Edge\NativeMessagingHosts\com.yonder.context_spike' -Value $manifestPath
}
[ordered]@{ manifest = $manifestPath; extension = $extensionPath; extension_id = $extensionId; browser = $Browser } | ConvertTo-Json -Compress
