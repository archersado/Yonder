// 仅操作com.yonder.desktop；验证独立Jev设置窗口的读取、保存、拒绝和只读安全说明。
import AppKit
import ApplicationServices

guard CommandLine.arguments.count == 2, AXIsProcessTrusted(),
      let app = NSRunningApplication.runningApplications(withBundleIdentifier: "com.yonder.desktop").first
      ?? NSWorkspace.shared.runningApplications.first(where: { $0.localizedName == "yonder-desktop" }) else { exit(2) }

let output = URL(fileURLWithPath: CommandLine.arguments[1], isDirectory: true)
guard !FileManager.default.fileExists(atPath: output.path) else { exit(2) }
try FileManager.default.createDirectory(at: output, withIntermediateDirectories: true)

let ax = AXUIElementCreateApplication(app.processIdentifier)
func attr(_ element: AXUIElement, _ name: String) -> CFTypeRef? {
    var value: CFTypeRef?
    return AXUIElementCopyAttributeValue(element, name as CFString, &value) == .success ? value : nil
}
func find(_ element: AXUIElement, _ title: String, _ depth: Int = 0) -> AXUIElement? {
    if attr(element, "AXTitle") as? String == title || attr(element, "AXDescription") as? String == title { return element }
    guard depth < 20 else { return nil }
    for child in attr(element, "AXChildren") as? [AXUIElement] ?? [] {
        if let found = find(child, title, depth + 1) { return found }
    }
    return nil
}
func titles(_ element: AXUIElement, _ depth: Int = 0) -> [String] {
    var result = [attr(element, "AXTitle") as? String, attr(element, "AXDescription") as? String].compactMap { $0 }
    guard depth < 20 else { return result }
    for child in attr(element, "AXChildren") as? [AXUIElement] ?? [] { result.append(contentsOf: titles(child, depth + 1)) }
    return result
}
func window() -> AXUIElement? {
    (attr(ax, "AXWindows") as? [AXUIElement] ?? []).first { attr($0, "AXTitle") as? String == "Yonda · Jev 快脑设置" }
}
func value(_ element: AXUIElement) -> String { attr(element, "AXValue") as? String ?? "" }
func setValue(_ element: AXUIElement, _ next: String) -> Bool {
    return AXUIElementSetAttributeValue(element, "AXValue" as CFString, next as CFTypeRef) == .success
}
func press(_ element: AXUIElement) -> Bool { AXUIElementPerformAction(element, "AXPress" as CFString) == .success }
func until(_ deadline: TimeInterval, _ body: () -> Bool) -> Bool {
    let end = ProcessInfo.processInfo.systemUptime + deadline
    while ProcessInfo.processInfo.systemUptime < end {
        if body() { return true }
        Thread.sleep(forTimeInterval: 0.2)
    }
    return body()
}
func containsValue(_ element: AXUIElement, _ text: String, _ depth: Int = 0) -> Bool {
    if value(element).contains(text) { return true }
    guard depth < 20 else { return false }
    return (attr(element, "AXChildren") as? [AXUIElement] ?? []).contains { containsValue($0, text, depth + 1) }
}
func visibleWindowId() -> Int? {
    let list = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
    return list.first {
        ($0[kCGWindowOwnerPID as String] as? Int) == Int(app.processIdentifier) &&
        ($0[kCGWindowName as String] as? String) == "Yonda · Jev 快脑设置"
    }?[kCGWindowNumber as String] as? Int
}
func capture(_ name: String) -> Bool {
    guard let id = visibleWindowId() else { return false }
    let process = Process()
    process.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture")
    process.arguments = ["-x", "-l", String(id), output.appendingPathComponent(name).path]
    do { try process.run(); process.waitUntilExit(); return process.terminationStatus == 0 } catch { return false }
}
func sql(_ statement: String) -> String? {
    let process = Process()
    process.executableURL = URL(fileURLWithPath: "/usr/bin/sqlite3")
    process.arguments = [NSHomeDirectory() + "/Library/Application Support/com.yonder.desktop/tasks.db", statement]
    let pipe = Pipe(); process.standardOutput = pipe
    do { try process.run(); process.waitUntilExit() } catch { return nil }
    guard process.terminationStatus == 0 else { return nil }
    return String(data: pipe.fileHandleForReading.readDataToEndOfFile(), encoding: .utf8)
}

guard let extras = attr(ax, "AXExtrasMenuBar"), CFGetTypeID(extras) == AXUIElementGetTypeID() else { exit(3) }
guard let show = find(extras as! AXUIElement, "Jev 快脑设置") else { print("tray_menu_missing=true titles=\(titles(extras as! AXUIElement))"); exit(3) }
guard press(show) else { print("tray_menu_press_failed=true"); exit(3) }
guard until(10, { window() != nil }), let settings = window() else { exit(4) }
guard until(5, { find(settings, "端点") != nil }) else { exit(5) }
Thread.sleep(forTimeInterval: 0.3)

let countsStatement = "SELECT (SELECT count(*) FROM tasks)||','||(SELECT count(*) FROM events)||','||(SELECT count(*) FROM outbox)"
guard let beforeCounts = sql(countsStatement) else { exit(8) }
guard let endpoint = find(settings, "端点"), let enabled = find(settings, "启用 Jev"),
      let save = find(settings, "保存 Jev 配置") else { exit(9) }
let initialEndpoint = value(endpoint)
let initiallyEnabled = attr(enabled, "AXValue") as? Int == 1
guard capture("jev-config-read.png") else { exit(11) }
if !initiallyEnabled && !press(enabled) { exit(10) }
guard setValue(endpoint, "http://127.0.0.1:1234/jev"), press(save) else { exit(11) }
Thread.sleep(forTimeInterval: 1)
guard capture("jev-config-saved.png") else { exit(12) }

guard setValue(endpoint, "http://127.0.0.1:1234/jev?secret=1"), press(save) else { exit(16) }
Thread.sleep(forTimeInterval: 1)
guard capture("jev-config-invalid.png") else { exit(16) }
guard containsValue(settings, "置信阈值 0.75、动作后强制 Observe、敏感操作须确认；这些安全闸不可降低。") else { exit(17) }

guard let afterCounts = sql(countsStatement), beforeCounts == afterCounts,
      let configRows = sql("SELECT count(*) FROM jev_config"), configRows.trimmingCharacters(in: .whitespacesAndNewlines) == "1",
      let config = sql("SELECT config_json FROM jev_config WHERE id=1"),
      config.contains("\"enabled\":true"), config.contains("\"endpoint\":\"http://127.0.0.1:1234/jev\"") else { exit(18) }

let report: [String: Any] = [
    "pid": Int(app.processIdentifier),
    "read_config": !initialEndpoint.isEmpty,
    "valid_save": true,
    "invalid_rejected": true,
    "readonly_safety_visible": true,
    "task_event_outbox_counts_unchanged": true,
    "before_counts": beforeCounts,
    "after_counts": afterCounts,
    "jev_config_rows": 1,
    "passed": true
]
let data = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys])
try data.write(to: output.appendingPathComponent("result.json"))
print(String(data: data, encoding: .utf8)!)
