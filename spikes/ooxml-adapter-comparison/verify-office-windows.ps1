$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient

$officeRoot = 'C:\Program Files\Microsoft Office\root\Office16'
$cases = @(
    @{ name = 'docx'; application = Join-Path $officeRoot 'WINWORD.EXE'; process = 'WINWORD'; document = Join-Path $PSScriptRoot 'output\rust\real-sample.docx' },
    @{ name = 'xlsx'; application = Join-Path $officeRoot 'EXCEL.EXE'; process = 'EXCEL'; document = Join-Path $PSScriptRoot 'output\rust\real-sample.xlsx' },
    @{ name = 'pptx'; application = Join-Path $officeRoot 'POWERPNT.EXE'; process = 'POWERPNT'; document = Join-Path $PSScriptRoot 'output\rust\real-sample.pptx' }
)

$results = [Collections.Generic.List[object]]::new()
foreach ($case in $cases) {
    if (@(Get-Process $case.process -ErrorAction SilentlyContinue).Count -ne 0) {
        throw "$($case.process)已有用户进程，停止验证"
    }
    if (-not (Test-Path -LiteralPath $case.application) -or -not (Test-Path -LiteralPath $case.document)) {
        throw "$($case.name)验证输入不可用"
    }
    $started = Start-Process -FilePath $case.application -ArgumentList @($case.document) -PassThru
    $processes = @()
    try {
        $deadline = (Get-Date).AddSeconds(30)
        do {
            Start-Sleep -Milliseconds 500
            $processes = @(Get-Process $case.process -ErrorAction SilentlyContinue)
            $window = $processes | Where-Object MainWindowHandle -ne 0 | Select-Object -First 1
        } while ($null -eq $window -and (Get-Date) -lt $deadline)
        if ($null -eq $window) { throw "$($case.name)没有创建可验证窗口" }
        Start-Sleep -Seconds 5
        $processes = @(Get-Process $case.process -ErrorAction SilentlyContinue)
        $titles = @($processes | ForEach-Object { $_.Refresh(); $_.MainWindowTitle } |
            Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
        $automationNames = @()
        $automationError = $null
        $currentWindow = $processes | Where-Object MainWindowHandle -ne 0 | Select-Object -First 1
        if ($null -ne $currentWindow) {
            try {
                $root = [Windows.Automation.AutomationElement]::FromHandle($currentWindow.MainWindowHandle)
                $automationNames = @($root.FindAll(
                    [Windows.Automation.TreeScope]::Descendants,
                    [Windows.Automation.Condition]::TrueCondition) | ForEach-Object {
                        try { $_.Current.Name } catch { $null }
                    } | Where-Object { $_ -like '*real-sample*' -or $_ -match 'repair|corrupt|damage|修复|损坏|错误' } |
                    Select-Object -Unique)
            } catch {
                $automationError = $_.Exception.Message
            }
        }
        $documentVisible = @($titles + $automationNames | Where-Object { $_ -like '*real-sample*' }).Count -gt 0
        $repairOrDamage = @($titles + $automationNames |
            Where-Object { $_ -match 'repair|corrupt|damage|修复|损坏|错误' }).Count -gt 0
        $results.Add([ordered]@{
            format = $case.name
            window_created = $true
            window_titles = $titles
            matching_automation_names = $automationNames
            automation_error = $automationError
            document_name_visible = $documentVisible
            repair_or_damage_dialog = $repairOrDamage
            passed = $documentVisible -and -not $repairOrDamage
        })
    } finally {
        foreach ($process in @(Get-Process $case.process -ErrorAction SilentlyContinue)) {
            Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
        }
        Start-Sleep -Seconds 1
    }
}

[ordered]@{
    platform = 'windows'
    office = 'Microsoft Office 16'
    results = $results
    passed = @($results | Where-Object { -not $_.passed }).Count -eq 0
} | ConvertTo-Json -Depth 5 -Compress
