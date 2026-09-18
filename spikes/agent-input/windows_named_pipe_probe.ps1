$ErrorActionPreference = 'Stop'
$pipeName = "yonder-agent-input-$([Guid]::NewGuid().ToString('N'))"
$utf8 = [Text.UTF8Encoding]::new($false)
$server = [IO.Pipes.NamedPipeServerStream]::new(
    $pipeName,
    [IO.Pipes.PipeDirection]::InOut,
    1,
    [IO.Pipes.PipeTransmissionMode]::Byte,
    [IO.Pipes.PipeOptions]::None
)

$agent = Start-Job -ArgumentList $pipeName -ScriptBlock {
    param($name)
    $ErrorActionPreference = 'Stop'
    $utf8 = [Text.UTF8Encoding]::new($false)
    $pipe = [IO.Pipes.NamedPipeClientStream]::new('.', $name, [IO.Pipes.PipeDirection]::InOut)
    try {
        $pipe.Connect(5000)
        $reader = [IO.StreamReader]::new($pipe, $utf8, $false, 1024, $true)
        $writer = [IO.StreamWriter]::new($pipe, $utf8, 1024, $true)
        $writer.AutoFlush = $true
        $writer.WriteLine((@{ method = 'gateway.hello'; id = 'hello-1'; params = @{
            agent_id = 'windows-fixture'; session_id = 'windows-session-1';
            offered_capabilities = @('user_input')
        }} | ConvertTo-Json -Compress -Depth 4))
        $hello = $reader.ReadLine() | ConvertFrom-Json
        if (-not $hello.result.accepted) { throw '握手未确认' }
        $input = $reader.ReadLine() | ConvertFrom-Json
        if ($input.method -ne 'agent.input' -or $input.params.session_id -ne 'windows-session-1') {
            throw '输入没有绑定原会话'
        }
        $writer.WriteLine((@{ id = $input.id; result = @{ accepted = $true } } |
            ConvertTo-Json -Compress -Depth 3))
    } finally {
        $pipe.Dispose()
    }
}

try {
    $server.WaitForConnection()
    $reader = [IO.StreamReader]::new($server, $utf8, $false, 1024, $true)
    $writer = [IO.StreamWriter]::new($server, $utf8, 1024, $true)
    $writer.AutoFlush = $true
    $hello = $reader.ReadLine() | ConvertFrom-Json
    if ($hello.params.offered_capabilities -notcontains 'user_input') { throw '缺少user_input能力' }
    $writer.WriteLine('{"id":"hello-1","result":{"accepted":true}}')
    $now = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    $writer.WriteLine((@{ method = 'agent.input'; id = 'input-1'; params = @{
        input_id = 'input-1'; session_id = $hello.params.session_id; source = 'voice';
        content = '测试Windows输入'; created_at = $now; deadline = $now + 10000
    }} | ConvertTo-Json -Compress -Depth 4))
    $ack = $reader.ReadLine() | ConvertFrom-Json
    if ($ack.id -ne 'input-1' -or -not $ack.result.accepted) { throw '输入未确认' }
    Wait-Job $agent -Timeout 5 | Out-Null
    if ($agent.State -ne 'Completed') { throw "Agent作业失败：$($agent.State)" }
    Receive-Job $agent -ErrorAction Stop | Out-Null
    @{ transport = 'named-pipe'; session = 'bound'; result = 'accepted' } |
        ConvertTo-Json -Compress
} finally {
    $server.Dispose()
    Remove-Job $agent -Force -ErrorAction SilentlyContinue
}
