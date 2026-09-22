// 仅操作指定Yonda进程：原生拖动、窗口位移与误开菜单验证。
import AppKit
import ApplicationServices
import CoreGraphics

let pidValue = Int(CommandLine.arguments[1]) ?? 0
guard CommandLine.arguments.count == 3, AXIsProcessTrusted(),
      let app = NSRunningApplication(processIdentifier: Int32(pidValue)) else { exit(2) }
_ = app.activate(options: [])
let output = URL(fileURLWithPath: CommandLine.arguments[2], isDirectory: true)
try FileManager.default.createDirectory(at: output, withIntermediateDirectories: true)

let session = CGSessionCopyCurrentDictionary() as? [String: Any]
if session?["CGSSessionScreenIsLocked"] as? Int == 1 {
    let blocked: [String: Any] = ["blocked_by": "screen_locked", "passed": false]
    let data = try JSONSerialization.data(withJSONObject: blocked, options: [.prettyPrinted, .sortedKeys])
    try data.write(to: output.appendingPathComponent("native-result.json"))
    print(String(data: data, encoding: .utf8)!)
    exit(9)
}

func petInfo() -> (id: Int, bounds: CGRect)? {
    let windows = CGWindowListCopyWindowInfo([.optionAll, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
    guard let info = windows.first(where: {
        ($0[kCGWindowOwnerPID as String] as? Int) == Int(app.processIdentifier)
            && ($0[kCGWindowName as String] as? String) == "Yonda"
    }), let id = info[kCGWindowNumber as String] as? Int,
       let values = info[kCGWindowBounds as String] as? NSDictionary,
       let bounds = CGRect(dictionaryRepresentation: values) else { return nil }
    return (id, bounds)
}

func menuVisible() -> Bool {
    let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
    return windows.contains {
        ($0[kCGWindowOwnerPID as String] as? Int) == Int(app.processIdentifier)
            && ($0[kCGWindowName as String] as? String) == "Yonda · Task Space"
    }
}

func post(_ type: CGEventType, _ point: CGPoint) {
    CGEvent(mouseEventSource: nil, mouseType: type, mouseCursorPosition: point, mouseButton: .left)?.post(tap: .cghidEventTap)
}

func capture(_ id: Int, _ name: String) {
    let process = Process()
    process.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture")
    process.arguments = ["-x", "-l", String(id), output.appendingPathComponent(name).path]
    do { try process.run() } catch { exit(7) }
    process.waitUntilExit()
    guard process.terminationStatus == 0 else { exit(7) }
}

let end = ProcessInfo.processInfo.systemUptime + 15
var before = petInfo()
while before == nil && ProcessInfo.processInfo.systemUptime < end {
    Thread.sleep(forTimeInterval: 0.25)
    before = petInfo()
}
guard let (windowId, beforeBounds) = before else { exit(4) }
capture(windowId, "before-native-drag.png")

let screen = NSScreen.main?.visibleFrame ?? .zero
let candidates = [(100.0, 100.0), (-100.0, 100.0), (100.0, -100.0), (-100.0, -100.0)]
let targetBounds = candidates.compactMap { delta -> CGRect? in
    let origin = CGPoint(x: beforeBounds.origin.x + delta.0, y: beforeBounds.origin.y + delta.1)
    let rect = CGRect(origin: origin, size: beforeBounds.size)
    return screen.insetBy(dx: 20, dy: 20).contains(rect) ? rect : nil
}.first ?? CGRect(x: screen.midX - beforeBounds.width / 2, y: screen.midY - beforeBounds.height / 2, width: beforeBounds.width, height: beforeBounds.height)

let beforeCenter = CGPoint(x: beforeBounds.midX, y: beforeBounds.midY)
let targetCenter = CGPoint(x: targetBounds.midX, y: targetBounds.midY)
Thread.sleep(forTimeInterval: 0.3)
post(.mouseMoved, beforeCenter)
Thread.sleep(forTimeInterval: 0.1)
post(.leftMouseDown, beforeCenter)
for step in 1...16 {
    let progress = CGFloat(step) / 16
    let point = CGPoint(
        x: beforeCenter.x + (targetCenter.x - beforeCenter.x) * progress,
        y: beforeCenter.y + (targetCenter.y - beforeCenter.y) * progress
    )
    post(.mouseMoved, point)
    Thread.sleep(forTimeInterval: 0.015)
}
post(.leftMouseUp, targetCenter)
Thread.sleep(forTimeInterval: 0.5)

guard let (afterId, afterBounds) = petInfo() else { exit(5) }
capture(afterId, "after-native-drag.png")
let moved = hypot(afterBounds.origin.x - beforeBounds.origin.x, afterBounds.origin.y - beforeBounds.origin.y) >= 50
let withinScreen = screen.insetBy(dx: 10, dy: 10).contains(afterBounds)
let menuDidNotOpen = !menuVisible()
let stillVisible = afterId == windowId && !afterBounds.isEmpty
let report: [String: Any] = [
    "pid": Int(app.processIdentifier),
    "window_id": windowId,
    "before_bounds": ["x": beforeBounds.origin.x, "y": beforeBounds.origin.y, "width": beforeBounds.width, "height": beforeBounds.height],
    "after_bounds": ["x": afterBounds.origin.x, "y": afterBounds.origin.y, "width": afterBounds.width, "height": afterBounds.height],
    "target_bounds": ["x": targetBounds.origin.x, "y": targetBounds.origin.y, "width": targetBounds.width, "height": targetBounds.height],
    "movement_points": hypot(afterBounds.origin.x - beforeBounds.origin.x, afterBounds.origin.y - beforeBounds.origin.y),
    "native_drag_moved_window": moved,
    "window_within_screen": withinScreen,
    "task_menu_not_opened": menuDidNotOpen,
    "pet_window_visible": stillVisible,
    "passed": moved && withinScreen && menuDidNotOpen && stillVisible,
]
let data = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys])
try data.write(to: output.appendingPathComponent("native-result.json"))
print(String(data: data, encoding: .utf8)!)
guard moved, withinScreen, menuDidNotOpen, stillVisible else { exit(8) }
