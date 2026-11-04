$ErrorActionPreference='Stop'
$desktopExe='F:\workspace\yonder\target\debug\yonder-desktop.exe'
$registerExe='F:\workspace\yonder\target\debug\examples\register-agent.exe'
$appData=Join-Path $env:APPDATA 'com.yonder.desktop'
$pipeName='com.yonder.desktop.agent'
$agentId='tm-s6-win-'+[guid]::NewGuid().ToString('N').Substring(0,10)
$resultPath=Join-Path $PSScriptRoot 'native-running-dock-result.json'
$owned=$null;$session=$null;$task=$null;$finished=$false;$request=0
$result=[ordered]@{platform='windows';cua_executed=$false;ego_lite_executed=$true;running_observation_seconds=190;post_finish_timeout_seconds=205}
Add-Type @'
using System;using System.Runtime.InteropServices;using System.Text;
public static class YondaRunningDockProbe{
 private delegate bool EnumWindowsProc(IntPtr h,IntPtr s);
 [DllImport("user32.dll")]private static extern bool EnumWindows(EnumWindowsProc c,IntPtr s);
 [DllImport("user32.dll")]private static extern uint GetWindowThreadProcessId(IntPtr h,out uint p);
 [DllImport("user32.dll")]private static extern bool IsWindowVisible(IntPtr h);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)]private static extern int GetWindowText(IntPtr h,StringBuilder t,int n);
 [DllImport("user32.dll")]private static extern bool GetWindowRect(IntPtr h,out Rect r);
 [StructLayout(LayoutKind.Sequential)]private struct Rect{public int Left,Top,Right,Bottom;}
 public static int[] FindPet(int wanted){int[] found=null;EnumWindows(delegate(IntPtr h,IntPtr s){uint p;GetWindowThreadProcessId(h,out p);if(p!=wanted||!IsWindowVisible(h))return true;var t=new StringBuilder(128);GetWindowText(h,t,t.Capacity);if(t.ToString()!="Yonda")return true;Rect r;if(GetWindowRect(h,out r))found=new[]{r.Left,r.Top,r.Right-r.Left,r.Bottom-r.Top};return found==null;},IntPtr.Zero);return found;}
}
'@
function Deadline{[long](([datetime]::UtcNow-[datetime]'1970-01-01').TotalMilliseconds+60000)}
function Connect-Gateway{$until=[datetime]::UtcNow.AddSeconds(30);do{$p=[IO.Pipes.NamedPipeClientStream]::new('.',$pipeName,[IO.Pipes.PipeDirection]::InOut,[IO.Pipes.PipeOptions]::Asynchronous);try{$p.Connect(500);$e=[Text.UTF8Encoding]::new($false);$r=[IO.StreamReader]::new($p,$e,$false,65536,$true);$w=[IO.StreamWriter]::new($p,$e,65536,$true);$w.AutoFlush=$true;return @{Pipe=$p;Reader=$r;Writer=$w}}catch{$p.Dispose();Start-Sleep -Milliseconds 200}}while([datetime]::UtcNow -lt $until);throw 'Named Pipe未就绪'}
function Send([string]$method,[string]$cap,[hashtable]$extra){$script:request++;$params=[ordered]@{agent_id=$agentId;capability=$cap;deadline=Deadline};foreach($e in $extra.GetEnumerator()){$params[$e.Key]=$e.Value};$q=[ordered]@{jsonrpc='2.0';id="tm-s6-$script:request";method=$method;params=$params};$session.Writer.WriteLine(($q|ConvertTo-Json -Compress -Depth 10));$line=$session.Reader.ReadLine();if([string]::IsNullOrWhiteSpace($line)){throw "Gateway未响应：$method"};$a=$line|ConvertFrom-Json;if($a.error){throw "Gateway失败 $method：$($a.error|ConvertTo-Json -Compress)"};return $a.result}
function Close-Session{if($session){$session.Writer.Dispose();$session.Reader.Dispose();$session.Pipe.Dispose();$script:session=$null}}
function Snapshot([int]$ProcessId){$r=[YondaRunningDockProbe]::FindPet($ProcessId);if(!$r){return $null};return [ordered]@{x=$r[0];y=$r[1];width=$r[2];height=$r[3]}}
function Is-Docked($Rect){return $Rect -and (($Rect.width -eq 168 -and $Rect.height -eq 84)-or($Rect.width -eq 84 -and $Rect.height -eq 168)-or($Rect.width -eq 112 -and $Rect.height -eq 56)-or($Rect.width -eq 56 -and $Rect.height -eq 112))}

if(!(Test-Path -LiteralPath $desktopExe)-or !(Test-Path -LiteralPath $registerExe)){throw '缺少Windows桌面或注册夹具'}
if(Get-Process yonder-desktop -ErrorAction SilentlyContinue){throw '检测到现有Yonda进程，拒绝干扰'}
try{
 & $registerExe $appData $agentId|Out-Null;if($LASTEXITCODE -ne 0){throw '测试Agent登记失败'}
 $s=New-Object Diagnostics.ProcessStartInfo;$s.FileName=$desktopExe;$s.WorkingDirectory=Split-Path -Parent $desktopExe;$s.UseShellExecute=$false;$s.RedirectStandardError=$true;$owned=[Diagnostics.Process]::Start($s);$result.process_id=$owned.Id
 $session=Connect-Gateway;$hello=Send 'gateway.hello' 'task.read' @{protocol_version=[ordered]@{major=1;minor=19}};$result.agent_id=$agentId
 $created=Send 'task.create' 'task.create' @{idempotency_key="tm-s6-$([guid]::NewGuid().ToString('N'))";name='TM-S6 Windows运行态收起门禁';description='验证真实BUA运行期间阻止空闲收起，完成后允许收起'};$task=$created.task;$result.task_id=$task.task_id
 $declared=Send 'task.step.declare' 'task.step.declare' @{task_id=$task.task_id;expected_sequence=$task.sequence;step_id='open-space';label='创建Browser Task Space'}
 $opened=Send 'browser.execute' 'browser.execute' @{task_id=$task.task_id;expected_sequence=$declared.task.sequence;operation='create'};$task=$opened.task;if($task.status -ne 'running'){throw 'BUA任务未进入running'};$result.external_task_ref=$opened.reference.external_task_ref
 Start-Sleep -Seconds 2;$result.expanded_at_running_start=Snapshot $owned.Id;if(!$result.expanded_at_running_start -or (Is-Docked $result.expanded_at_running_start)){throw '运行观察开始时桌宠不是展开态'}
 $watch=[Diagnostics.Stopwatch]::StartNew();$runningDocked=$false
 while($watch.Elapsed.TotalSeconds -lt 190){Start-Sleep -Seconds 1;$rect=Snapshot $owned.Id;if(Is-Docked $rect){$runningDocked=$true;break};if($owned.HasExited){throw '运行观察期间桌面进程退出'};if([Math]::Floor($watch.Elapsed.TotalSeconds)%30 -eq 0){Write-Output ("RUNNING {0}s expanded" -f [Math]::Floor($watch.Elapsed.TotalSeconds));Start-Sleep -Milliseconds 1100}}
 $result.running_elapsed_seconds=[Math]::Round($watch.Elapsed.TotalSeconds,3);$result.running_docked=$runningDocked;$result.expanded_after_running_threshold=Snapshot $owned.Id;if($runningDocked){throw '真实运行任务超过空闲阈值时错误收起'}
 $advanced=Send 'task.step.advance' 'task.step.advance' @{task_id=$task.task_id;expected_sequence=$task.sequence};$declared=Send 'task.step.declare' 'task.step.declare' @{task_id=$task.task_id;expected_sequence=$advanced.task.sequence;step_id='finish-space';label='完成Browser Task Space'};$done=Send 'browser.execute' 'browser.execute' @{task_id=$task.task_id;expected_sequence=$declared.task.sequence;operation='finish'};$task=$done.task;if($task.status -ne 'completed'-or !$done.reference.finished){throw 'BUA任务未完成'};$finished=$true;$result.task_final=$task.status
 $watch.Restart();$docked=$null
 while($watch.Elapsed.TotalSeconds -lt 205 -and !$docked){Start-Sleep -Seconds 1;$rect=Snapshot $owned.Id;if(Is-Docked $rect){$docked=$rect;break};if($owned.HasExited){throw '完成后观察期间桌面进程退出'};if([Math]::Floor($watch.Elapsed.TotalSeconds)%30 -eq 0){Write-Output ("POST-FINISH {0}s waiting dock" -f [Math]::Floor($watch.Elapsed.TotalSeconds));Start-Sleep -Milliseconds 1100}}
 $result.post_finish_dock_elapsed_seconds=[Math]::Round($watch.Elapsed.TotalSeconds,3);$result.docked_after_finish=$docked;$result.passed=!$runningDocked -and $finished -and [bool]$docked;if(!$result.passed){throw '完成后未在期限内自然收起'}
}catch{$result.passed=$false;$result.error=$_.Exception.Message;throw}
finally{
 if($session -and $task -and !$finished){try{$snap=Send 'task.get' 'task.read' @{task_id=$task.task_id};if($snap.task.status -eq 'running'){$a=Send 'task.step.advance' 'task.step.advance' @{task_id=$task.task_id;expected_sequence=$snap.task.sequence};$d=Send 'task.step.declare' 'task.step.declare' @{task_id=$task.task_id;expected_sequence=$a.task.sequence;step_id='cleanup-finish';label='异常路径完成Browser Task Space'};$f=Send 'browser.execute' 'browser.execute' @{task_id=$task.task_id;expected_sequence=$d.task.sequence;operation='finish'};$result.cleanup=[ordered]@{status=$f.task.status;finished=$f.reference.finished}}elseif($snap.task.status -eq 'created'){$c=Send 'task.cancel' 'task.cancel' @{task_id=$task.task_id;expected_sequence=$snap.task.sequence};$result.cleanup=[ordered]@{status=$c.task.status}}}catch{$result.cleanup_error=$_.Exception.Message}}
 Close-Session;if($owned -and !$owned.HasExited){$owned.Kill();$owned.WaitForExit()};if($owned){$result.diagnostic_log=@($owned.StandardError.ReadToEnd()-split "`r?`n"|Where-Object{$_})};& $registerExe $appData $agentId disable|Out-Null;$result|ConvertTo-Json -Depth 10|Set-Content -LiteralPath $resultPath -Encoding UTF8
}
$result|ConvertTo-Json -Depth 10 -Compress
