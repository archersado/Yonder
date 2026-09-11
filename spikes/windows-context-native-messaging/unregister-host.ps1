$keys = @(
  'HKCU:\Software\Google\Chrome\NativeMessagingHosts\com.yonder.context_spike',
  'HKCU:\Software\Microsoft\Edge\NativeMessagingHosts\com.yonder.context_spike'
)
foreach ($key in $keys) { Remove-Item -LiteralPath $key -ErrorAction SilentlyContinue }
Remove-Item -LiteralPath (Join-Path $env:LOCALAPPDATA 'Yonder\context-spike') -Recurse -Force -ErrorAction SilentlyContinue
