param(
    [Parameter(Mandatory = $true)]
    [int]$ProcessId,
    [int]$DurationSeconds = 300,
    [int]$IntervalSeconds = 5
)

$ErrorActionPreference = 'Stop'
$logicalProcessors = [Environment]::ProcessorCount
$samples = [Collections.Generic.List[object]]::new()
$startedAt = Get-Date
$deadline = $startedAt.AddSeconds($DurationSeconds)
$previousCpu = $null
$previousAt = $null
$maxProcesses = 0

function Get-ProcessTreeIds([int]$RootId) {
    $processes = @(Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId)
    $ids = [Collections.Generic.HashSet[int]]::new()
    [void]$ids.Add($RootId)
    do {
        $changed = $false
        foreach ($item in $processes) {
            if ($ids.Contains([int]$item.ParentProcessId) -and $ids.Add([int]$item.ProcessId)) {
                $changed = $true
            }
        }
    } while ($changed)
    return @($ids)
}

while ((Get-Date) -lt $deadline) {
    if (-not (Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)) {
        throw "Yonder主进程在采样期间退出"
    }
    $now = Get-Date
    $ids = @(Get-ProcessTreeIds $ProcessId)
    $processes = @($ids | ForEach-Object { Get-Process -Id $_ -ErrorAction SilentlyContinue })
    $workingSet = ($processes | Measure-Object WorkingSet64 -Sum).Sum
    $cpu = ($processes | ForEach-Object { $_.TotalProcessorTime.TotalSeconds } | Measure-Object -Sum).Sum
    $cpuPercent = $null
    if ($null -ne $previousCpu) {
        $elapsed = ($now - $previousAt).TotalSeconds
        # 与既有 macOS 证据相同：进程树累计 CPU 秒 / 墙钟秒，按单核 100% 口径记录。
        $cpuPercent = [Math]::Max(0, (($cpu - $previousCpu) / $elapsed) * 100)
    }
    $maxProcesses = [Math]::Max($maxProcesses, $processes.Count)
    $samples.Add([pscustomobject]@{
        at = $now.ToString('o')
        process_count = $processes.Count
        working_set_bytes = [int64]$workingSet
        cpu_percent = $cpuPercent
    })
    $previousCpu = $cpu
    $previousAt = $now
    Start-Sleep -Seconds $IntervalSeconds
}

$workingSets = @($samples | ForEach-Object { $_.working_set_bytes } | Sort-Object)
$cpuValues = @($samples | Where-Object { $null -ne $_.cpu_percent } | ForEach-Object { $_.cpu_percent })
$averageWorkingSet = ($workingSets | Measure-Object -Average).Average
$averageCpu = ($cpuValues | Measure-Object -Average).Average
$p95Index = [Math]::Min($workingSets.Count - 1, [Math]::Floor($workingSets.Count * 0.95))

[ordered]@{
    collected_at = (Get-Date).ToString('o')
    process_id = $ProcessId
    requested_duration_seconds = $DurationSeconds
    actual_duration_seconds = [Math]::Round(((Get-Date) - $startedAt).TotalSeconds, 3)
    interval_seconds = $IntervalSeconds
    sample_count = $samples.Count
    logical_processors = $logicalProcessors
    maximum_process_count = $maxProcesses
    working_set = [ordered]@{
        average_bytes = [int64]$averageWorkingSet
        maximum_bytes = [int64]($workingSets | Measure-Object -Maximum).Maximum
        p95_bytes = [int64]$workingSets[$p95Index]
    }
    cpu = [ordered]@{
        aggregate_average_percent = [Math]::Round($averageCpu, 4)
        aggregate_maximum_percent = [Math]::Round(($cpuValues | Measure-Object -Maximum).Maximum, 4)
        machine_normalized_average_percent = [Math]::Round($averageCpu / $logicalProcessors, 4)
    }
    result = 'completed'
} | ConvertTo-Json -Depth 6
