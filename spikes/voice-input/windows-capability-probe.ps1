$ErrorActionPreference = 'Stop'

$speech = [type]::GetType('Windows.Media.SpeechRecognition.SpeechRecognizer, Windows.Media.SpeechRecognition, ContentType=WindowsRuntime')
$capture = [type]::GetType('Windows.Media.Capture.MediaCapture, Windows.Media.Capture, ContentType=WindowsRuntime')

[ordered]@{
  platform = 'Windows'
  capture_framework = 'Windows.Media.Capture'
  speech_framework = 'Windows.Media.SpeechRecognition'
  capture_api = $null -ne $capture
  speech_api = $null -ne $speech
  requested_permission = $false
  opened_microphone = $false
} | ConvertTo-Json -Compress
