param(
    [Parameter(Mandatory=$true)][string]$AgentId,
    [Parameter(Mandatory=$true)][string]$TaskId,
    [Parameter(Mandatory=$true)][string]$ExpectedSequence
)
$ErrorActionPreference='Stop'
$root='F:\workspace\yonder\target\debug'
$appData=Join-Path $env:APPDATA 'com.yonder.desktop'
$register=Join-Path $root 'examples\register-agent.exe'
$desktop=Join-Path $root 'yonder-desktop.exe'
$cli=Join-Path $root 'yonder.exe'
$process=$null
try {
    & $register $appData $AgentId
    if($LASTEXITCODE -ne 0){throw '测试Agent重新启用失败'}
    $process=Start-Process -FilePath $desktop -WorkingDirectory $root -PassThru
    Start-Sleep -Seconds 3
    $env:YONDER_AGENT_ID=$AgentId
    $messages=@(
        '{"jsonrpc":"2.0","id":"init","method":"initialize","params":{}}',
        (ConvertTo-Json -Compress -Depth 8 ([ordered]@{jsonrpc='2.0';id='cancel';method='tools/call';params=[ordered]@{name='task_cancel';arguments=[ordered]@{task_id=$TaskId;expected_sequence=$ExpectedSequence}}}))
    )
    $responses=@($messages|& $cli mcp)
    if($LASTEXITCODE -ne 0){throw 'Yonder CLI取消调用失败'}
    $cancel=$responses[-1]|ConvertFrom-Json
    if($cancel.result.isError -or $cancel.result.content[0].text -notmatch 'cancelled'){throw '测试任务未确认取消'}
    $responses|Set-Content -LiteralPath (Join-Path $PSScriptRoot 'cleanup-created-probe-result.jsonl') -Encoding UTF8
} finally {
    if($process -and -not $process.HasExited){$process.Kill();$process.WaitForExit()}
    & $register $appData $AgentId disable|Out-Null
}
