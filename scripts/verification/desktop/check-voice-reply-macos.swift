// 只操作当前运行的 com.yonder.desktop 候选包；验证本地语音回复设置入口与持久化。
import AppKit
import ApplicationServices

guard CommandLine.arguments.count == 2, AXIsProcessTrusted(),
      let app = NSWorkspace.shared.runningApplications.first(where: {
          $0.bundleIdentifier == "com.yonder.desktop" &&
          ($0.executableURL?.path.contains("Yonder-vi-s3-product") ?? false)
      }) else { exit(2) }

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
func containsValue(_ element: AXUIElement, _ text: String, _ depth: Int = 0) -> Bool {
    if (attr(element, "AXValue") as? String)?.contains(text) == true { return true }
    guard depth < 20 else { return false }
    return (attr(element, "AXChildren") as? [AXUIElement] ?? []).contains { containsValue($0, text, depth + 1) }
}
func press(_ element: AXUIElement) -> Bool { AXUIElementPerformAction(element, "AXPress" as CFString) == .success }
func until(_ seconds: TimeInterval, _ body: () -> Bool) -> Bool {
    let end = ProcessInfo.processInfo.systemUptime + seconds
    while ProcessInfo.processInfo.systemUptime < end {
        if body() { return true }
        Thread.sleep(forTimeInterval: 0.2)
    }
    return body()
}
func window() -> AXUIElement? {
    (attr(ax, "AXWindows") as? [AXUIElement] ?? []).first { attr($0, "AXTitle") as? String == "Yonda · Agent 管理" }
}
func visibleWindowId() -> Int? {
    let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
    return windows.first {
        ($0[kCGWindowOwnerPID as String] as? Int) == Int(app.processIdentifier) &&
        ($0[kCGWindowName as String] as? String) == "Yonda · Agent 管理"
    }?[kCGWindowNumber as String] as? Int
}
func capture(_ name: String) -> Bool {
    guard let id = visibleWindowId() else { return false }
    let process = Process()
    process.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture")
    process.arguments = ["-x", "-l", String(id), output.appendingPathComponent(name).path]
    do { try process.run(); process.waitUntilExit(); return process.terminationStatus == 0 } catch { return false }
}
func storedEnabled() -> Bool {
    let process = Process()
    process.executableURL = URL(fileURLWithPath: "/usr/bin/defaults")
    process.arguments = ["read", "com.yonder.desktop", "yonda.voiceReply.enabled"]
    let pipe = Pipe(); process.standardOutput = pipe
    do { try process.run(); process.waitUntilExit() } catch { return false }
    let value = String(data: pipe.fileHandleForReading.readDataToEndOfFile(), encoding: .utf8) ?? ""
    return process.terminationStatus == 0 && value.trimmingCharacters(in: .whitespacesAndNewlines) == "1"
}

guard let extras = attr(ax, "AXExtrasMenuBar"), CFGetTypeID(extras) == AXUIElementGetTypeID(),
      let show = find(extras as! AXUIElement, "Agent 管理"), press(show),
      until(10, { window() != nil }), let settings = window(),
      until(5, { find(settings, "使用本地中文声音朗读 Agent 回复") != nil }),
      let toggle = find(settings, "使用本地中文声音朗读 Agent 回复") else { exit(3) }

let initial = attr(toggle, "AXValue") as? Int == 1
let initialCapture = capture("voice-reply-setting-initial.png")
if !initial { guard press(toggle) else { exit(5) } }
guard until(5, { (attr(toggle, "AXValue") as? Int == 1) && storedEnabled() }),
      containsValue(settings, "只使用本机已安装的中文声音") else { exit(6) }
let enabledCapture = capture("voice-reply-setting-enabled.png")

let report: [String: Any] = [
    "candidate_bundle": true,
    "explicit_toggle_visible": true,
    "local_boundary_visible": true,
    "enabled_persisted": true,
    "initially_enabled": initial,
    "screenshots_captured": initialCapture && enabledCapture,
    "passed": true
]
let data = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys])
try data.write(to: output.appendingPathComponent("result.json"))
print(String(data: data, encoding: .utf8)!)
