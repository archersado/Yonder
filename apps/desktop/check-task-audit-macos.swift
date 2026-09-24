// 仅操作com.yonder.desktop；验证终态任务的本机结果确认UI。
import AppKit
import ApplicationServices

guard CommandLine.arguments.count == 3, AXIsProcessTrusted(),
      let pid = Int32(CommandLine.arguments[2]),
      let app = NSRunningApplication(processIdentifier: pid)
else { exit(2) }

let ax = AXUIElementCreateApplication(app.processIdentifier)
func attr(_ element: AXUIElement, _ name: String) -> CFTypeRef? {
    var value: CFTypeRef?
    return AXUIElementCopyAttributeValue(element, name as CFString, &value) == .success ? value : nil
}
func find(_ element: AXUIElement, _ title: String, _ depth: Int = 0) -> AXUIElement? {
    if ["AXTitle", "AXDescription", "AXValue"].contains(where: { (attr(element, $0) as? String)?.contains(title) == true }) {
        return element
    }
    guard depth < 20 else { return nil }
    for child in attr(element, "AXChildren") as? [AXUIElement] ?? [] {
        if let found = find(child, title, depth + 1) { return found }
    }
    return nil
}
func findButton(_ element: AXUIElement, _ title: String, _ depth: Int = 0) -> AXUIElement? {
    guard depth < 20 else { return nil }
    if ["AXButton", "AXCheckBox", "AXToggleButton"].contains(attr(element, "AXRole") as? String ?? ""),
       ["AXTitle", "AXDescription", "AXValue"].contains(where: { (attr(element, $0) as? String)?.contains(title) == true }) {
        return element
    }
    for child in attr(element, "AXChildren") as? [AXUIElement] ?? [] {
        if let found = findButton(child, title, depth + 1) { return found }
    }
    return nil
}
func taskWindow() -> AXUIElement? {
    (attr(ax, "AXWindows") as? [AXUIElement] ?? []).first {
        attr($0, "AXTitle") as? String == "Yonda · Task Space"
    }
}
func visibleWindowId() -> Int? {
    let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
    return windows.first {
        ($0[kCGWindowOwnerPID as String] as? Int) == Int(app.processIdentifier)
            && ($0[kCGWindowName as String] as? String) == "Yonda · Task Space"
    }?[kCGWindowNumber as String] as? Int
}

let output = URL(fileURLWithPath: CommandLine.arguments[1], isDirectory: true)
try FileManager.default.createDirectory(at: output, withIntermediateDirectories: true)
let deadline = ProcessInfo.processInfo.systemUptime + 20
guard let extras = attr(ax, "AXExtrasMenuBar"),
      CFGetTypeID(extras) == AXUIElementGetTypeID(),
      let show = find(extras as! AXUIElement, "任务总览"),
      AXUIElementPerformAction(show, "AXPress" as CFString) == .success
else { exit(3) }
while taskWindow() == nil && ProcessInfo.processInfo.systemUptime < deadline { Thread.sleep(forTimeInterval: 0.1) }
guard let menu = taskWindow() else { exit(4) }
guard let all = findButton(menu, "全部"), AXUIElementPerformAction(all, "AXPress" as CFString) == .success else { exit(5) }
Thread.sleep(forTimeInterval: 0.5)
guard let card = findButton(taskWindow() ?? menu, "TM-S5 审计确认"),
      AXUIElementPerformAction(card, "AXPress" as CFString) == .success
else { exit(6) }
while find(taskWindow() ?? menu, "结果待确认") == nil && ProcessInfo.processInfo.systemUptime < deadline {
    Thread.sleep(forTimeInterval: 0.1)
}
guard find(taskWindow() ?? menu, "结果待确认") != nil else { exit(7) }
if let comment = find(taskWindow() ?? menu, "结果确认意见") {
    AXUIElementSetAttributeValue(comment, "AXValue" as CFString, "原生验证：结果可用" as CFTypeRef)
}
guard let confirm = findButton(taskWindow() ?? menu, "确认结果"),
      AXUIElementPerformAction(confirm, "AXPress" as CFString) == .success
else { exit(8) }
Thread.sleep(forTimeInterval: 1)
guard let refreshed = findButton(taskWindow() ?? menu, "TM-S5 审计确认"),
      AXUIElementPerformAction(refreshed, "AXPress" as CFString) == .success
else { exit(9) }
while find(taskWindow() ?? menu, "已确认 · 结果序号 3") == nil && ProcessInfo.processInfo.systemUptime < deadline {
    Thread.sleep(forTimeInterval: 0.1)
}
guard let confirmedWindow = taskWindow(),
      find(confirmedWindow, "已确认 · 结果序号 3") != nil,
      find(confirmedWindow, "清单版本 1") != nil,
      let windowId = visibleWindowId()
else { exit(10) }
let capture = Process()
capture.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture")
capture.arguments = ["-x", "-l", String(windowId), output.appendingPathComponent("native-confirmed.png").path]
try capture.run()
capture.waitUntilExit()
guard capture.terminationStatus == 0 else { exit(11) }

let report: [String: Any] = [
    "pid": Int(app.processIdentifier),
    "window_id": windowId,
    "terminal_task_visible": true,
    "pending_confirmation_visible": true,
    "confirmation_submitted": true,
    "confirmed_projection_visible": true,
    "manifest_version_visible": true,
    "passed": true
]
let data = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys])
try data.write(to: output.appendingPathComponent("native-result.json"))
print(String(data: data, encoding: .utf8)!)
