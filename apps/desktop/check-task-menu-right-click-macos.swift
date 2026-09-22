// 仅操作com.yonder.desktop；验证清醒小龙右键唤起真实任务菜单。
import AppKit
import ApplicationServices

guard CommandLine.arguments.count == 2, AXIsProcessTrusted(),
      let app = NSRunningApplication.runningApplications(withBundleIdentifier: "com.yonder.desktop").first else { exit(2) }

let ax = AXUIElementCreateApplication(app.processIdentifier)
func attr(_ element: AXUIElement, _ name: String) -> CFTypeRef? {
    var value: CFTypeRef?
    return AXUIElementCopyAttributeValue(element, name as CFString, &value) == .success ? value : nil
}
func find(_ element: AXUIElement, _ title: String, _ depth: Int = 0) -> AXUIElement? {
    let description = attr(element, "AXDescription") as? String
    if attr(element, "AXTitle") as? String == title || description == title || description?.contains(title) == true { return element }
    guard depth < 16 else { return nil }
    for child in attr(element, "AXChildren") as? [AXUIElement] ?? [] {
        if let found = find(child, title, depth + 1) { return found }
    }
    return nil
}
func findText(_ element: AXUIElement, _ text: String, _ depth: Int = 0) -> Bool {
    guard depth < 16 else { return false }
    if ["AXValue", "AXDescription"].contains(where: { (attr(element, $0) as? String)?.contains(text) == true }) { return true }
    return (attr(element, "AXChildren") as? [AXUIElement] ?? []).contains { findText($0, text, depth + 1) }
}
func window() -> AXUIElement? {
    (attr(ax, "AXWindows") as? [AXUIElement] ?? []).first { attr($0, "AXTitle") as? String == "Yonda · Task Space" }
}
func visibleWindow() -> [String: Any]? {
    let list = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
    return list.first {
        ($0[kCGWindowOwnerPID as String] as? Int) == Int(app.processIdentifier)
            && ($0[kCGWindowName as String] as? String) == "Yonda · Task Space"
    }
}
func visiblePetWindow() -> [String: Any]? {
    let list = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
    return list.first {
        ($0[kCGWindowOwnerPID as String] as? Int) == Int(app.processIdentifier)
            && ($0[kCGWindowName as String] as? String) == "Yonda"
    }
}

let output = URL(fileURLWithPath: CommandLine.arguments[1], isDirectory: true)
guard !FileManager.default.fileExists(atPath: output.path) else { exit(2) }
try FileManager.default.createDirectory(at: output, withIntermediateDirectories: true)

guard let pet = visiblePetWindow(),
      let bounds = pet[kCGWindowBounds as String] as? [String: Any],
      let rect = CGRect(dictionaryRepresentation: bounds as CFDictionary) else { exit(3) }

if let existing = window(), let close = find(existing, "关闭任务总览") {
    _ = AXUIElementPerformAction(close, "AXPress" as CFString)
    Thread.sleep(forTimeInterval: 0.5)
}
let center = CGPoint(x: rect.midX, y: rect.midY)
CGEvent(mouseEventSource: nil, mouseType: .mouseMoved, mouseCursorPosition: center, mouseButton: .left)?.post(tap: .cghidEventTap)
CGEvent(mouseEventSource: nil, mouseType: .rightMouseDown, mouseCursorPosition: center, mouseButton: .right)?.post(tap: .cghidEventTap)
Thread.sleep(forTimeInterval: 0.1)
CGEvent(mouseEventSource: nil, mouseType: .rightMouseUp, mouseCursorPosition: center, mouseButton: .right)?.post(tap: .cghidEventTap)
Thread.sleep(forTimeInterval: 1.0)

guard let menu = window(), let visible = visibleWindow(), let id = visible[kCGWindowNumber as String] as? Int else { exit(4) }
if let all = find(menu, "全部") { _ = AXUIElementPerformAction(all, "AXPress" as CFString) }
Thread.sleep(forTimeInterval: 0.5)
guard findText(menu, "1 项") else { exit(4) }
let capture = Process()
capture.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture")
capture.arguments = ["-x", "-l", String(id), output.appendingPathComponent("native-right-click.png").path]
try capture.run()
capture.waitUntilExit()
guard capture.terminationStatus == 0 else { exit(5) }

guard let close = find(menu, "关闭任务总览"), AXUIElementPerformAction(close, "AXPress" as CFString) == .success else { exit(6) }
Thread.sleep(forTimeInterval: 0.8)
guard visibleWindow() == nil else { exit(7) }

let report: [String: Any] = [
    "pid": Int(app.processIdentifier),
    "window_id": id,
    "right_click_opens_menu": true,
    "real_task_visible": true,
    "close_hides": true,
    "passed": true
]
let data = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys])
try data.write(to: output.appendingPathComponent("result.json"))
print(String(data: data, encoding: .utf8)!)
