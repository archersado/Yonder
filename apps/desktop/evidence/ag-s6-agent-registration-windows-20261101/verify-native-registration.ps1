$ErrorActionPreference='Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class YondaAgentProbe {
  [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left,Top,Right,Bottom; }
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint flags,uint x,uint y,uint data,UIntPtr extra);
  [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string className,string windowName);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr handle,out Rect rect);
  public static int[] MenuRect(){var h=FindWindow("#32768",null);Rect r;if(h==IntPtr.Zero||!GetWindowRect(h,out r))return null;return new[]{r.Left,r.Top,r.Right-r.Left,r.Bottom-r.Top};}
}
'@

$desktopExe='F:\workspace\yonder\target\debug\yonder-desktop.exe'
$pipeName='com.yonder.desktop.agent'
$agentId='ag-s6-win-'+[guid]::NewGuid().ToString('N').Substring(0,12)
$unknownId='ag-s6-unknown-'+[guid]::NewGuid().ToString('N').Substring(0,8)
$resultPath=Join-Path $PSScriptRoot 'native-agent-registration-result.json'
$screenshotPath=Join-Path $PSScriptRoot 'native-agent-revoked.png'
$owned=$null;$session=$null;$requestNumber=0;$task=$null
$result=[ordered]@{platform='windows';cua_executed=$false;agent_id=$agentId}

function Deadline {[long](([datetime]::UtcNow-[datetime]'1970-01-01').TotalMilliseconds+60000)}
function Connect-Pipe {
  $until=[datetime]::UtcNow.AddSeconds(20)
  do { $pipe=[IO.Pipes.NamedPipeClientStream]::new('.',$pipeName,[IO.Pipes.PipeDirection]::InOut,[IO.Pipes.PipeOptions]::Asynchronous)
    try{$pipe.Connect(500);$enc=[Text.UTF8Encoding]::new($false);$reader=[IO.StreamReader]::new($pipe,$enc,$false,65536,$true);$writer=[IO.StreamWriter]::new($pipe,$enc,65536,$true);$writer.AutoFlush=$true;return @{Pipe=$pipe;Reader=$reader;Writer=$writer}}catch{$pipe.Dispose();Start-Sleep -Milliseconds 200}
  }while([datetime]::UtcNow -lt $until);throw 'Named Pipe未就绪'
}
function Close-Pipe([object]$Value){if($Value){try{$Value.Writer.Dispose()}catch{};try{$Value.Reader.Dispose()}catch{};try{$Value.Pipe.Dispose()}catch{}}}
function Request([object]$Session,[string]$Agent,[string]$Method,[string]$Capability,[hashtable]$Extra){
  $script:requestNumber++;$params=[ordered]@{agent_id=$Agent;capability=$Capability;deadline=Deadline};foreach($e in $Extra.GetEnumerator()){$params[$e.Key]=$e.Value}
  $req=[ordered]@{jsonrpc='2.0';id="ag-s6-$script:requestNumber";method=$Method;params=$params};$Session.Writer.WriteLine(($req|ConvertTo-Json -Compress -Depth 10));$line=$Session.Reader.ReadLine();if([string]::IsNullOrWhiteSpace($line)){throw "Gateway关闭：$Method"};return $line|ConvertFrom-Json
}
function Find-Element([scriptblock]$Predicate,[Windows.Automation.AutomationElement]$Root=[Windows.Automation.AutomationElement]::RootElement){
  try{$all=$Root.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.Condition]::TrueCondition)}catch{return $null};foreach($e in $all){try{if(& $Predicate $e){return $e}}catch{}};return $null
}
function Wait-Element([scriptblock]$Predicate,[Windows.Automation.AutomationElement]$Root=[Windows.Automation.AutomationElement]::RootElement,[int]$Seconds=15){$until=[datetime]::UtcNow.AddSeconds($Seconds);do{$e=Find-Element $Predicate $Root;if($e){return $e};Start-Sleep -Milliseconds 150}while([datetime]::UtcNow -lt $until);throw '等待原生UI元素超时'}
function Click([Windows.Automation.AutomationElement]$Element){$r=$Element.Current.BoundingRectangle;if(!$Element.Current.IsOffscreen -and $r.Width -gt 0 -and $r.Height -gt 0){[void][YondaAgentProbe]::SetCursorPos([int]($r.Left+$r.Width/2),[int]($r.Top+$r.Height/2));[YondaAgentProbe]::mouse_event(2,0,0,0,[UIntPtr]::Zero);[YondaAgentProbe]::mouse_event(4,0,0,0,[UIntPtr]::Zero)}else{$Element.GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern).Invoke()};Start-Sleep -Milliseconds 700}
function Find-Tray { $e=Find-Element {param($x)$x.Current.Name -eq 'Yonda' -and $x.Current.ClassName -eq 'SystemTray.NormalButton' -and !$x.Current.IsOffscreen};if($e){return $e};$hidden=Wait-Element {param($x)$x.Current.Name -eq '显示隐藏的图标'};Click $hidden;return Wait-Element {param($x)$x.Current.Name -eq 'Yonda' -and $x.Current.ClassName -eq 'SystemTray.NormalButton' -and !$x.Current.IsOffscreen} }
function Open-AgentWindow {
  [void](Find-Tray)
  $all=[Windows.Automation.AutomationElement]::RootElement.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.Condition]::TrueCondition)
  $icons=@($all|Where-Object{try{$_.Current.Name -eq 'Yonda' -and $_.Current.ClassName -eq 'SystemTray.NormalButton' -and !$_.Current.IsOffscreen}catch{$false}})
  foreach($icon in $icons){
    $r=$icon.Current.BoundingRectangle;if($r.Width -le 0 -or $r.Height -le 0){continue};[void][YondaAgentProbe]::SetCursorPos([int]($r.Left+$r.Width/2),[int]($r.Top+$r.Height/2));[YondaAgentProbe]::mouse_event(8,0,0,0,[UIntPtr]::Zero);[YondaAgentProbe]::mouse_event(16,0,0,0,[UIntPtr]::Zero);Start-Sleep -Milliseconds 500
    $m=[YondaAgentProbe]::MenuRect();if(!$m){continue};$x=[int]($m[0]+$m[2]/2);$y=[int]($m[1]+($m[3]/7.0)*5.5);[void][YondaAgentProbe]::SetCursorPos($x,$y);[YondaAgentProbe]::mouse_event(2,0,0,0,[UIntPtr]::Zero);[YondaAgentProbe]::mouse_event(4,0,0,0,[UIntPtr]::Zero)
    $until=[datetime]::UtcNow.AddSeconds(4);do{$window=Find-Element {param($e)$e.Current.Name -eq 'Yonda · Agent 管理' -and $e.Current.ControlType -eq [Windows.Automation.ControlType]::Window};if($window){return $window};Start-Sleep -Milliseconds 150}while([datetime]::UtcNow -lt $until)
  }
  throw '没有可用的Yonda托盘图标打开Agent管理'
}
function Nearby-Button([Windows.Automation.AutomationElement]$Window,[Windows.Automation.AutomationElement]$Reference,[string]$Name){$rr=$Reference.Current.BoundingRectangle;$best=$null;$distance=1e9;$all=$Window.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.Condition]::TrueCondition);foreach($e in $all){try{if($e.Current.Name -eq $Name -and $e.Current.ControlType -eq [Windows.Automation.ControlType]::Button){$r=$e.Current.BoundingRectangle;$d=[Math]::Abs(($r.Top+$r.Height/2)-($rr.Top+$rr.Height/2));if($d -lt $distance){$best=$e;$distance=$d}}}catch{}};if(!$best -or $distance -gt 90){throw "找不到同行按钮：$Name"};return $best}
function Capture([Windows.Automation.AutomationElement]$Window){$r=$Window.Current.BoundingRectangle;$w=[int]$r.Width;$h=[int]$r.Height;$b=New-Object Drawing.Bitmap($w,$h,[Drawing.Imaging.PixelFormat]::Format32bppArgb);$g=[Drawing.Graphics]::FromImage($b);try{$g.CopyFromScreen([int]$r.Left,[int]$r.Top,0,0,(New-Object Drawing.Size($w,$h)));$b.Save($screenshotPath,[Drawing.Imaging.ImageFormat]::Png)}finally{$g.Dispose();$b.Dispose()};return [ordered]@{file='native-agent-revoked.png';width=$w;height=$h;sha256=(Get-FileHash $screenshotPath -Algorithm SHA256).Hash.ToLowerInvariant()}}
function Start-Yonda {$s=New-Object Diagnostics.ProcessStartInfo;$s.FileName=$desktopExe;$s.WorkingDirectory=Split-Path -Parent $desktopExe;$s.UseShellExecute=$false;$s.RedirectStandardError=$true;$script:owned=[Diagnostics.Process]::Start($s);Start-Sleep -Seconds 4}
function Stop-Yonda {if($script:owned -and !$script:owned.HasExited){$script:owned.Kill();$script:owned.WaitForExit()};if($script:owned){$script:result.diagnostic_log=@($script:owned.StandardError.ReadToEnd()-split "`r?`n"|Where-Object{$_})}}

if(!(Test-Path -LiteralPath $desktopExe)){throw '缺少Windows桌面可执行文件'}
if(Get-Process yonder-desktop -ErrorAction SilentlyContinue){throw '检测到现有Yonda进程，拒绝干扰'}
try{
  $result.stage='start-first-host'
  Start-Yonda
  $result.stage='reject-unregistered'
  $unknown=Connect-Pipe;try{$denied=Request $unknown $unknownId 'gateway.hello' 'task.read' @{protocol_version=[ordered]@{major=1;minor=19}};$result.unregistered=[ordered]@{code=$denied.error.code;message=$denied.error.message};if($denied.error.code -ne -32003){throw '未登记Agent未被拒绝'}}finally{Close-Pipe $unknown}

  $result.stage='open-agent-window';$window=Open-AgentWindow;$result.stage='find-agent-input';$input=Wait-Element {param($e)$e.Current.ControlType -eq [Windows.Automation.ControlType]::Edit -and $e.Current.Name -eq 'Agent ID'} $window;$input.GetCurrentPattern([Windows.Automation.ValuePattern]::Pattern).SetValue($agentId);$result.stage='register-ui';Click (Wait-Element {param($e)$e.Current.Name -eq '登记 Agent' -and $e.Current.ControlType -eq [Windows.Automation.ControlType]::Button} $window);Start-Sleep -Seconds 2;$result.ui_names_after_register=@($window.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.Condition]::TrueCondition)|ForEach-Object{try{$_.Current.Name}catch{}}|Where-Object{$_}|Select-Object -Unique);$row=Wait-Element {param($e)$e.Current.Name.Contains($agentId)} $window

  $result.stage='enabled-gateway-create';$session=Connect-Pipe;$hello=Request $session $agentId 'gateway.hello' 'task.read' @{protocol_version=[ordered]@{major=1;minor=19}};if($hello.error){throw '登记后握手失败'};$created=Request $session $agentId 'task.create' 'task.create' @{idempotency_key="ag-s6-$([guid]::NewGuid().ToString('N'))";name='AG-S6 Windows撤权验证';description='验证Agent禁用、撤权与任务保留'};if($created.error){throw '登记后创建任务失败'};$task=$created.result.task;$result.task_id=$task.task_id

  $result.stage='disable-ui';$row=Wait-Element {param($e)$e.Current.Name.Contains($agentId)} $window;Click (Nearby-Button $window $row '禁用');[void](Wait-Element {param($e)$e.Current.Name.Contains($agentId) -and $e.Current.Name.Contains('已禁用')} $window)
  $disabledActive=$null;try{$disabledActive=Request $session $agentId 'task.step.declare' 'task.step.declare' @{task_id=$task.task_id;expected_sequence=$task.sequence;step_id='blocked-after-disable';label='禁用后不得写入'}}catch{};$result.disabled_active_rejection=$disabledActive.error.code;if($disabledActive.error.code -ne -32003){throw '禁用后活动会话写请求未被拒绝'};Close-Pipe $session;$session=$null
  $deniedSession=Connect-Pipe;try{$disabled=Request $deniedSession $agentId 'gateway.hello' 'task.read' @{protocol_version=[ordered]@{major=1;minor=19}};$result.disabled_rejection=$disabled.error.code;if($disabled.error.code -ne -32003){throw '禁用Agent重新连接未拒绝'}}finally{Close-Pipe $deniedSession}

  $row=Wait-Element {param($e)$e.Current.Name.Contains($agentId)} $window;Click (Nearby-Button $window $row '启用');[void](Wait-Element {param($e)$e.Current.Name.Contains($agentId) -and $e.Current.Name.Contains('已启用')} $window)
  $session=Connect-Pipe;$hello=Request $session $agentId 'gateway.hello' 'task.read' @{protocol_version=[ordered]@{major=1;minor=19}};$cancel=Request $session $agentId 'task.cancel' 'task.cancel' @{task_id=$task.task_id;expected_sequence=$task.sequence};if($cancel.result.task.status -ne 'cancelled'){throw '验证任务未正式取消'};$result.task_final=$cancel.result.task.status;Close-Pipe $session;$session=Connect-Pipe;$hello=Request $session $agentId 'gateway.hello' 'task.read' @{protocol_version=[ordered]@{major=1;minor=19};session_id="ag-s6-session-$([guid]::NewGuid().ToString('N').Substring(0,8))";offered_capabilities=@('user_input')}

  $row=Wait-Element {param($e)$e.Current.Name.Contains($agentId)} $window;$revoke=Nearby-Button $window $row '撤权';Click $revoke;$row=Wait-Element {param($e)$e.Current.Name.Contains($agentId)} $window;Click (Nearby-Button $window $row '确认撤权');[void](Wait-Element {param($e)$e.Current.Name.Contains($agentId) -and $e.Current.Name.Contains('已撤权')} $window)
  $closed=$false;try{$read=$session.Reader.ReadLineAsync();$closed=$read.Wait(5000) -and [string]::IsNullOrWhiteSpace($read.Result)}catch{$closed=$true};$result.active_session_disconnected=$closed;if(!$closed){throw '撤权后活动会话未断开'};Close-Pipe $session;$session=$null
  $revokedSession=Connect-Pipe;try{$revoked=Request $revokedSession $agentId 'gateway.hello' 'task.read' @{protocol_version=[ordered]@{major=1;minor=19}};$result.revoked_rejection=$revoked.error.code;if($revoked.error.code -ne -32003){throw '撤权Agent重新连接未拒绝'}}finally{Close-Pipe $revokedSession}
  Stop-Yonda;$owned=$null;Start-Yonda;$window=Open-AgentWindow;$persisted=Wait-Element {param($e)$e.Current.Name.Contains($agentId) -and $e.Current.Name.Contains('已撤权')} $window;try{$persisted.GetCurrentPattern([Windows.Automation.ScrollItemPattern]::Pattern).ScrollIntoView();Start-Sleep -Milliseconds 500}catch{};$result.restart_persisted=$true;$result.screenshot=Capture $window;$result.passed=$result.active_session_disconnected -and $result.task_final -eq 'cancelled' -and $result.restart_persisted
}catch{$result.passed=$false;$result.error=$_.Exception.Message;throw}
finally{Close-Pipe $session;Stop-Yonda;$result|ConvertTo-Json -Depth 10|Set-Content -LiteralPath $resultPath -Encoding UTF8}
$result|ConvertTo-Json -Depth 10 -Compress
