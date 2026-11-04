$ErrorActionPreference='Stop'
$desktopExe='F:\workspace\yonder\target\debug\yonder-desktop.exe'
$registerExe='F:\workspace\yonder\target\debug\examples\register-agent.exe'
$appData=Join-Path $env:APPDATA 'com.yonder.desktop'
$pipeName='com.yonder.desktop.agent'
$pending=[ordered]@{
  'ag-s6-win-f77e18aba651'='task_4c1b804159c3ee7e823c5dc16556df7c'
  'ag-s6-win-f01e86ed7066'='task_67c98f1f657a37da01c572cb9e1cac28'
  'ag-s6-win-76c8e9c4dd2b'='task_703f38be203196808f029bbbdc1ca9f1'
}
$disable=@('ag-s6-win-573605781fa3','ag-s6-win-7843d751fa6b','ag-s6-win-b04b23d3a8a3','ag-s6-win-e5f1ebe7cfb0')+$pending.Keys
$process=$null;$results=[Collections.Generic.List[object]]::new();$request=0
function Connect { $p=[IO.Pipes.NamedPipeClientStream]::new('.',$pipeName,[IO.Pipes.PipeDirection]::InOut);$p.Connect(5000);$e=[Text.UTF8Encoding]::new($false);$r=[IO.StreamReader]::new($p,$e,$false,65536,$true);$w=[IO.StreamWriter]::new($p,$e,65536,$true);$w.AutoFlush=$true;return @{Pipe=$p;Reader=$r;Writer=$w} }
function Send([object]$s,[string]$agent,[string]$method,[string]$cap,[hashtable]$extra){$script:request++;$params=[ordered]@{agent_id=$agent;capability=$cap;deadline=[long](([datetime]::UtcNow-[datetime]'1970-01-01').TotalMilliseconds+60000)};foreach($x in $extra.GetEnumerator()){$params[$x.Key]=$x.Value};$q=[ordered]@{jsonrpc='2.0';id="cleanup-$script:request";method=$method;params=$params};$s.Writer.WriteLine(($q|ConvertTo-Json -Compress -Depth 8));return $s.Reader.ReadLine()|ConvertFrom-Json}
function Close([object]$s){if($s){$s.Writer.Dispose();$s.Reader.Dispose();$s.Pipe.Dispose()}}
try{
  foreach($agent in $pending.Keys){& $registerExe $appData $agent | Out-Null;if($LASTEXITCODE -ne 0){throw "无法启用测试Agent：$agent"}}
  $start=New-Object Diagnostics.ProcessStartInfo;$start.FileName=$desktopExe;$start.WorkingDirectory=Split-Path -Parent $desktopExe;$start.UseShellExecute=$false;$process=[Diagnostics.Process]::Start($start);Start-Sleep -Seconds 4
  foreach($agent in $pending.Keys){$s=$null;try{$s=Connect;$hello=Send $s $agent 'gateway.hello' 'task.read' @{protocol_version=[ordered]@{major=1;minor=19}};$cancel=Send $s $agent 'task.cancel' 'task.cancel' @{task_id=$pending[$agent];expected_sequence='1'};if($cancel.result.task.status -ne 'cancelled'){throw "测试任务未取消：$($pending[$agent])"};$results.Add([ordered]@{agent_id=$agent;task_id=$pending[$agent];status='cancelled'})}finally{Close $s}}
}finally{
  if($process -and !$process.HasExited){$process.Kill();$process.WaitForExit()}
  foreach($agent in $disable){& $registerExe $appData $agent disable | Out-Null}
}
$results|ConvertTo-Json -Depth 5
