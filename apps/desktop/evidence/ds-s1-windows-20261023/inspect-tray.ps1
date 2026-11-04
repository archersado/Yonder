$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName UIAutomationClient
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class TrayNative
{
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern IntPtr FindWindow(string className, string windowName);
}
'@

$matches = [Collections.Generic.List[object]]::new()
$roots = [Collections.Generic.List[object]]::new()
foreach ($className in @('Shell_TrayWnd', 'NotifyIconOverflowWindow')) {
    $handle = [TrayNative]::FindWindow($className, $null)
    if ($handle -eq [IntPtr]::Zero) { continue }
    $roots.Add([ordered]@{ class_name = $className; handle = $handle.ToInt64() })
    $root = [Windows.Automation.AutomationElement]::FromHandle($handle)
    $all = $root.FindAll(
        [Windows.Automation.TreeScope]::Descendants,
        [Windows.Automation.Condition]::TrueCondition)
    foreach ($element in $all) {
        try {
            $name = $element.Current.Name
            if ($name -like '*Yonda*' -or $name -like '*Yonder*') {
                $matches.Add([ordered]@{
                    root_class = $className
                    name = $name
                    control_type = $element.Current.ControlType.ProgrammaticName
                    automation_id = $element.Current.AutomationId
                    class_name = $element.Current.ClassName
                    offscreen = $element.Current.IsOffscreen
                    enabled = $element.Current.IsEnabled
                })
            }
        } catch {}
    }
}
[ordered]@{
    roots = $roots
    matches = $matches
} | ConvertTo-Json -Depth 5
