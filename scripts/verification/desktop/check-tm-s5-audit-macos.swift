// 仅操作调用方指定的隔离Yonda进程；验证Agent登记及TM-S5终态确认原生UI。
import AppKit
import ApplicationServices
import Foundation

guard CommandLine.arguments.count >= 4,
      AXIsProcessTrusted(),
      let pid = Int32(CommandLine.arguments[2]),
      let application = NSRunningApplication(processIdentifier: pid) else { exit(2) }

let mode = CommandLine.arguments[1]
let app = AXUIElementCreateApplication(application.processIdentifier)

func attribute(_ element: AXUIElement, _ name: String) -> CFTypeRef? {
    var value: CFTypeRef?
    return AXUIElementCopyAttributeValue(element, name as CFString, &value) == .success ? value : nil
}

func strings(_ element: AXUIElement) -> [String] {
    ["AXTitle", "AXDescription", "AXValue"].compactMap { attribute(element, $0) as? String }
}

func find(_ element: AXUIElement, containing text: String, role: String? = nil, depth: Int = 0) -> AXUIElement? {
    let roleMatches = role == nil || attribute(element, "AXRole") as? String == role
    if roleMatches && strings(element).contains(where: { $0.contains(text) }) { return element }
    guard depth < 20 else { return nil }
    for child in attribute(element, "AXChildren") as? [AXUIElement] ?? [] {
        if let found = find(child, containing: text, role: role, depth: depth + 1) { return found }
    }
    return nil
}

func wait(_ seconds: TimeInterval, until condition: () -> Bool) -> Bool {
    let deadline = ProcessInfo.processInfo.systemUptime + seconds
    while ProcessInfo.processInfo.systemUptime < deadline {
        if condition() { return true }
        Thread.sleep(forTimeInterval: 0.2)
    }
    return condition()
}

func press(_ element: AXUIElement) -> Bool {
    AXUIElementPerformAction(element, "AXPress" as CFString) == .success
}

func window(_ title: String) -> AXUIElement? {
    (attribute(app, "AXWindows") as? [AXUIElement] ?? []).first {
        attribute($0, "AXTitle") as? String == title
    }
}

func openTrayItem(_ title: String) -> Bool {
    guard let extras = attribute(app, "AXExtrasMenuBar"),
          CFGetTypeID(extras) == AXUIElementGetTypeID(),
          let item = find(extras as! AXUIElement, containing: title) else { return false }
    return press(item)
}

func describe(_ element: AXUIElement, depth: Int = 0) -> [String] {
    let line = "\(String(repeating: "  ", count: depth))\(attribute(element, "AXRole") as? String ?? "?") \(strings(element))"
    guard depth < 12 else { return [line] }
    return [line] + (attribute(element, "AXChildren") as? [AXUIElement] ?? []).flatMap { describe($0, depth: depth + 1) }
}

func scrollPageDown(_ title: String, steps: Int) -> Bool {
    let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
    guard let target = windows.first(where: {
        ($0[kCGWindowOwnerPID as String] as? Int) == Int(application.processIdentifier)
            && ($0[kCGWindowName as String] as? String) == title
    }), let bounds = target[kCGWindowBounds as String] as? [String: CGFloat],
          let x = bounds["X"], let y = bounds["Y"], let width = bounds["Width"], let height = bounds["Height"] else { return false }
    let point = CGPoint(x: x + width * 0.75, y: y + height * 0.75)
    CGEvent(mouseEventSource: nil, mouseType: .mouseMoved, mouseCursorPosition: point, mouseButton: .left)?.post(tap: .cghidEventTap)
    for _ in 0..<steps {
        CGEvent(scrollWheelEvent2Source: nil, units: .pixel, wheelCount: 1, wheel1: -240, wheel2: 0, wheel3: 0)?.post(tap: .cghidEventTap)
        Thread.sleep(forTimeInterval: 0.08)
    }
    Thread.sleep(forTimeInterval: 0.5)
    return true
}

func visibleWindowId(_ title: String) -> Int? {
    let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
    return windows.first {
        ($0[kCGWindowOwnerPID as String] as? Int) == Int(application.processIdentifier)
            && ($0[kCGWindowName as String] as? String) == title
    }?[kCGWindowNumber as String] as? Int
}

func rightClickPet() -> Bool {
    let finder = Process()
    finder.executableURL = URL(fileURLWithPath: "/usr/bin/open")
    finder.arguments = ["-a", "Finder"]
    do { try finder.run(); finder.waitUntilExit() } catch { return false }
    Thread.sleep(forTimeInterval: 1)
    func click() -> Bool {
        let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
        guard let pet = windows.first(where: {
            ($0[kCGWindowOwnerPID as String] as? Int) == Int(application.processIdentifier)
                && ($0[kCGWindowName as String] as? String) == "Yonda"
        }), let bounds = pet[kCGWindowBounds as String] as? [String: CGFloat],
              let x = bounds["X"], let y = bounds["Y"], let width = bounds["Width"], let height = bounds["Height"] else { return false }
        let point = CGPoint(x: x + width / 2, y: y + height / 2)
        CGEvent(mouseEventSource: nil, mouseType: .mouseMoved, mouseCursorPosition: point, mouseButton: .left)?.post(tap: .cghidEventTap)
        Thread.sleep(forTimeInterval: 0.3)
        CGEvent(mouseEventSource: nil, mouseType: .rightMouseDown, mouseCursorPosition: point, mouseButton: .right)?.post(tap: .cghidEventTap)
        CGEvent(mouseEventSource: nil, mouseType: .rightMouseUp, mouseCursorPosition: point, mouseButton: .right)?.post(tap: .cghidEventTap)
        return true
    }
    guard click() else { return false }
    Thread.sleep(forTimeInterval: 1)
    guard visibleWindowId("Yonda · Task Space") != nil || click() else { return false }
    return true
}

func capture(_ title: String, to path: String) -> Bool {
    guard let id = visibleWindowId(title) else { return false }
    let process = Process()
    process.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture")
    process.arguments = ["-x", "-l", String(id), path]
    do { try process.run(); process.waitUntilExit() } catch { return false }
    return process.terminationStatus == 0
}

if mode == "register" {
    let agentId = CommandLine.arguments[3]
    guard openTrayItem("Agent 管理"), wait(10, until: { window("Yonda · Agent 管理") != nil }),
          let panel = window("Yonda · Agent 管理") else { print("agent_window_unavailable"); exit(3) }
    guard let input = find(panel, containing: "Agent ID", role: "AXTextField") else {
        print(describe(panel).joined(separator: "\n")); exit(3)
    }
    guard AXUIElementSetAttributeValue(input, "AXValue" as CFString, agentId as CFTypeRef) == .success,
          let register = find(panel, containing: "登记 Agent", role: "AXButton"), press(register),
          wait(10, until: { find(panel, containing: agentId) != nil }) else { print(describe(panel).joined(separator: "\n")); exit(3) }
    if let close = find(panel, containing: "关闭 Agent 管理", role: "AXButton") { _ = press(close) }
    print("{\"agent_registered\":true}")
    exit(0)
}

guard mode == "verify" else { exit(2) }
let output = URL(fileURLWithPath: CommandLine.arguments[3], isDirectory: true)
guard !FileManager.default.fileExists(atPath: output.path) else { exit(2) }
try FileManager.default.createDirectory(at: output, withIntermediateDirectories: true)

guard rightClickPet(), wait(10, until: { visibleWindowId("Yonda · Task Space") != nil }),
      let panel = window("Yonda · Task Space") else { print("task_space_not_visible"); exit(4) }
guard wait(10, until: { find(panel, containing: "全部") != nil }),
      let all = find(panel, containing: "全部"), press(all) else {
    print(describe(panel).joined(separator: "\n")); exit(4)
}
guard wait(10, until: { find(panel, containing: "TM-S5 原生审计验证") != nil }),
      let task = find(panel, containing: "TM-S5 原生审计验证"), press(task),
      wait(10, until: { find(panel, containing: "结果待确认") != nil }) else {
    print(describe(panel).joined(separator: "\n")); exit(4)
}

guard find(panel, containing: "已完成") != nil,
      find(panel, containing: "确认后生成首个清单") != nil,
      find(panel, containing: "时间线") != nil,
      scrollPageDown("Yonda · Task Space", steps: 4),
      capture("Yonda · Task Space", to: output.appendingPathComponent("audit-pending.png").path),
      let comment = find(panel, containing: "结果确认意见", role: "AXTextArea"),
      AXUIElementSetAttributeValue(comment, "AXValue" as CFString, "原生验证：结果可用" as CFTypeRef) == .success,
      let confirm = find(panel, containing: "确认结果", role: "AXButton"), press(confirm) else {
    print(describe(panel).joined(separator: "\n")); exit(5)
}

Thread.sleep(forTimeInterval: 1)
if visibleWindowId("Yonda · Task Space") == nil {
    guard rightClickPet(), wait(10, until: { visibleWindowId("Yonda · Task Space") != nil }) else { exit(5) }
}
guard let confirmedPanel = window("Yonda · Task Space") else { exit(5) }
if find(confirmedPanel, containing: "已确认 · 结果序号") == nil {
    if find(confirmedPanel, containing: "TM-S5 原生审计验证") == nil,
       let allAgain = find(confirmedPanel, containing: "全部") { _ = press(allAgain) }
    guard wait(10, until: { find(confirmedPanel, containing: "TM-S5 原生审计验证") != nil }),
          let taskAgain = find(confirmedPanel, containing: "TM-S5 原生审计验证"), press(taskAgain) else {
        print(describe(confirmedPanel).joined(separator: "\n")); exit(5)
    }
}
guard wait(10, until: { find(confirmedPanel, containing: "已确认 · 结果序号") != nil }),
      find(confirmedPanel, containing: "清单版本 1") != nil,
      find(confirmedPanel, containing: "已完成") != nil,
      scrollPageDown("Yonda · Task Space", steps: 3),
      capture("Yonda · Task Space", to: output.appendingPathComponent("audit-confirmed.png").path) else {
    print(describe(confirmedPanel).joined(separator: "\n")); exit(5)
}

let report: [String: Any] = [
    "platform": "macos",
    "isolated_profile": true,
    "terminal_status_before": "completed",
    "pending_confirmation_visible": true,
    "empty_manifest_explained": true,
    "timeline_visible": true,
    "confirmation_submitted": true,
    "confirmed_projection_visible": true,
    "manifest_version": 1,
    "terminal_status_after": "completed",
    "contains_user_content": false,
    "passed": true,
]
let data = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys])
try data.write(to: output.appendingPathComponent("native-result.json"))
print(String(data: data, encoding: .utf8)!)
