// 仅操作指定Yonda进程；右键菜单、悬停不打开与失焦隐藏的原生验证。
import AppKit
import ApplicationServices

let pidValue = Int(CommandLine.arguments[1]) ?? 0
guard CommandLine.arguments.count == 3, AXIsProcessTrusted(),
      let app = NSRunningApplication(processIdentifier: Int32(pidValue)) else { exit(2) }
let output = URL(fileURLWithPath: CommandLine.arguments[2], isDirectory: true)
try FileManager.default.createDirectory(at: output, withIntermediateDirectories: true)
let ax = AXUIElementCreateApplication(app.processIdentifier)

func attr(_ element: AXUIElement, _ name: String) -> CFTypeRef? {
    var value: CFTypeRef?
    return AXUIElementCopyAttributeValue(element, name as CFString, &value) == .success ? value : nil
}

func find(_ element: AXUIElement, _ title: String, _ depth: Int = 0) -> AXUIElement? {
    if attr(element, "AXTitle") as? String == title || attr(element, "AXDescription") as? String == title { return element }
    guard depth < 16 else { return nil }
    for child in attr(element, "AXChildren") as? [AXUIElement] ?? [] {
        if let found = find(child, title, depth + 1) { return found }
    }
    return nil
}

func window() -> AXUIElement? {
    (attr(ax, "AXWindows") as? [AXUIElement] ?? []).first { attr($0, "AXTitle") as? String == "Yonda · Task Space" }
}

func visibleWindow() -> [String: Any]? {
    let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
    return windows.first {
        ($0[kCGWindowOwnerPID as String] as? Int) == Int(app.processIdentifier)
            && ($0[kCGWindowName as String] as? String) == "Yonda · Task Space"
    }
}

func move(_ point: CGPoint) {
    CGEvent(mouseEventSource: nil, mouseType: .mouseMoved, mouseCursorPosition: point, mouseButton: .left)?.post(tap: .cghidEventTap)
}

func rightClick(_ point: CGPoint) {
    CGEvent(mouseEventSource: nil, mouseType: .rightMouseDown, mouseCursorPosition: point, mouseButton: .right)?.post(tap: .cghidEventTap)
    CGEvent(mouseEventSource: nil, mouseType: .rightMouseUp, mouseCursorPosition: point, mouseButton: .right)?.post(tap: .cghidEventTap)
}

func activateFinder() {
    let process = Process()
    process.executableURL = URL(fileURLWithPath: "/usr/bin/open")
    process.arguments = ["-a", "Finder"]
    do { try process.run() } catch { exit(9) }
    process.waitUntilExit()
}

let end = ProcessInfo.processInfo.systemUptime + 15
guard let petWindow = (attr(ax, "AXWindows") as? [AXUIElement] ?? []).first(where: { attr($0, "AXTitle") as? String == "Yonda" }) else { exit(3) }
while attr(petWindow, "AXPosition") == nil && ProcessInfo.processInfo.systemUptime < end {
    Thread.sleep(forTimeInterval: 0.2)
}
guard let position = attr(petWindow, "AXPosition"), let size = attr(petWindow, "AXSize") else { exit(4) }
var point = CGPoint.zero
var dimensions = CGSize.zero
AXValueGetValue(position as! AXValue, .cgPoint, &point)
AXValueGetValue(size as! AXValue, .cgSize, &dimensions)

if let existing = window(), let close = find(existing, "关闭任务总览") {
    _ = AXUIElementPerformAction(close, "AXPress" as CFString)
    Thread.sleep(forTimeInterval: 0.5)
}
activateFinder()
Thread.sleep(forTimeInterval: 1)
let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
guard let petInfo = windows.first(where: {
    ($0[kCGWindowOwnerPID as String] as? Int) == Int(app.processIdentifier)
        && ($0[kCGWindowName as String] as? String) == "Yonda"
}), let bounds = petInfo[kCGWindowBounds as String] as? [String: CGFloat],
    let x = bounds["X"], let y = bounds["Y"], let width = bounds["Width"], let height = bounds["Height"] else { exit(5) }
let center = CGPoint(x: x + width / 2, y: y + height / 2)
move(center)
Thread.sleep(forTimeInterval: 1)
let hoverDoesNotOpen = visibleWindow() == nil
let inactiveBeforeClick = !app.isActive
rightClick(center)
let clickEnd = ProcessInfo.processInfo.systemUptime + 5
while visibleWindow() == nil && ProcessInfo.processInfo.systemUptime < clickEnd { Thread.sleep(forTimeInterval: 0.1) }
guard let visible = visibleWindow(), let id = visible[kCGWindowNumber as String] as? Int else { exit(6) }
let focusedWindow = attr(ax, "AXFocusedWindow").map { $0 as! AXUIElement }
let focusedTitle = focusedWindow.flatMap { attr($0, "AXTitle") as? String }
let menuFocused = focusedTitle == "Yonda · Task Space"
let capture = Process()
capture.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture")
capture.arguments = ["-x", "-l", String(id), output.appendingPathComponent("native-right-click-menu.png").path]
try capture.run()
capture.waitUntilExit()
guard capture.terminationStatus == 0 else { exit(7) }
activateFinder()
let blurEnd = ProcessInfo.processInfo.systemUptime + 3
while visibleWindow() != nil && ProcessInfo.processInfo.systemUptime < blurEnd { Thread.sleep(forTimeInterval: 0.1) }
let blurHides = visibleWindow() == nil
let activeAfterBlur = app.isActive
let focusedAfterBlur = attr(ax, "AXFocusedWindow").map { $0 as! AXUIElement }.flatMap { attr($0, "AXTitle") as? String }
let report: [String: Any] = [
    "pid": Int(app.processIdentifier),
    "window_id": id,
    "hover_does_not_open": hoverDoesNotOpen,
    "inactive_before_click": inactiveBeforeClick,
    "right_click_opens_menu": true,
    "menu_focused": menuFocused,
    "focused_window_title": focusedTitle ?? "",
    "blur_hides": blurHides,
    "active_after_blur": activeAfterBlur,
    "focused_window_after_blur": focusedAfterBlur ?? "",
    "passed": hoverDoesNotOpen && inactiveBeforeClick && menuFocused && blurHides,
]
let data = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys])
try data.write(to: output.appendingPathComponent("native-result.json"))
print(String(data: data, encoding: .utf8)!)
guard hoverDoesNotOpen, inactiveBeforeClick, menuFocused, blurHides else { exit(8) }
