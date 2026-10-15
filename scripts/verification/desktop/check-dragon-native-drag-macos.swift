// 仅操作指定Yonda进程：原生拖动、窗口位移、误开菜单与原位恢复验证。
import AppKit
import ApplicationServices
import CoreGraphics

func displayContaining(_ window: CGRect, candidates: [CGRect]) -> CGRect? {
    candidates.max { left, right in
        left.intersection(window).width * left.intersection(window).height
            < right.intersection(window).width * right.intersection(window).height
    }.flatMap { $0.intersects(window) ? $0 : nil }
}

func dragTarget(from bounds: CGRect, within display: CGRect) -> CGRect {
    let safe = display.insetBy(dx: 20, dy: 20)
    let candidates = [(100.0, 100.0), (-100.0, 100.0), (100.0, -100.0), (-100.0, -100.0)]
    return candidates.compactMap { delta -> CGRect? in
        let rect = CGRect(x: bounds.origin.x + delta.0, y: bounds.origin.y + delta.1, width: bounds.width, height: bounds.height)
        return safe.contains(rect) ? rect : nil
    }.first ?? CGRect(x: safe.midX - bounds.width / 2, y: safe.midY - bounds.height / 2, width: bounds.width, height: bounds.height)
}

if CommandLine.arguments == [CommandLine.arguments[0], "--self-test"] {
    let displays = [CGRect(x: -1920, y: 0, width: 1920, height: 1080), CGRect(x: 0, y: 0, width: 2560, height: 1440)]
    let leftWindow = CGRect(x: -1800, y: 300, width: 200, height: 200)
    guard displayContaining(leftWindow, candidates: displays) == displays[0], displays[0].insetBy(dx: 20, dy: 20).contains(dragTarget(from: leftWindow, within: displays[0])) else { exit(10) }
    print("display_selection=true target_within_actual_display=true")
    exit(0)
}

let pidValue = CommandLine.arguments.count > 1 ? Int(CommandLine.arguments[1]) ?? 0 : 0
guard CommandLine.arguments.count == 3, AXIsProcessTrusted(), CGPreflightPostEventAccess(),
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

guard let eventSource = CGEventSource(stateID: .hidSystemState) else { exit(2) }
func post(_ type: CGEventType, _ point: CGPoint) {
    CGEvent(mouseEventSource: eventSource, mouseType: type, mouseCursorPosition: point, mouseButton: .left)?.post(tap: .cghidEventTap)
}

func drag(from start: CGPoint, to end: CGPoint) {
    post(.mouseMoved, start)
    Thread.sleep(forTimeInterval: 0.1)
    post(.leftMouseDown, start)
    for step in 1...16 {
        let progress = CGFloat(step) / 16
        post(.leftMouseDragged, CGPoint(x: start.x + (end.x - start.x) * progress, y: start.y + (end.y - start.y) * progress))
        Thread.sleep(forTimeInterval: 0.015)
    }
    post(.leftMouseUp, end)
}

func activeDisplayBounds() -> [CGRect] {
    var count: UInt32 = 0
    guard CGGetActiveDisplayList(0, nil, &count) == .success else { return [] }
    var displays = Array(repeating: CGDirectDisplayID(), count: Int(count))
    guard CGGetActiveDisplayList(count, &displays, &count) == .success else { return [] }
    return displays.prefix(Int(count)).map(CGDisplayBounds)
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

guard let display = displayContaining(beforeBounds, candidates: activeDisplayBounds()) else { exit(6) }
let targetBounds = dragTarget(from: beforeBounds, within: display)

let beforeCenter = CGPoint(x: beforeBounds.midX, y: beforeBounds.midY)
let targetCenter = CGPoint(x: targetBounds.midX, y: targetBounds.midY)
Thread.sleep(forTimeInterval: 0.3)
drag(from: beforeCenter, to: targetCenter)
Thread.sleep(forTimeInterval: 0.5)

guard let (afterId, afterBounds) = petInfo() else { exit(5) }
capture(afterId, "after-native-drag.png")
let moved = hypot(afterBounds.origin.x - beforeBounds.origin.x, afterBounds.origin.y - beforeBounds.origin.y) >= 50
let withinScreen = display.insetBy(dx: 10, dy: 10).contains(afterBounds)
let menuDidNotOpen = !menuVisible()
let stillVisible = afterId == windowId && !afterBounds.isEmpty
if moved { drag(from: CGPoint(x: afterBounds.midX, y: afterBounds.midY), to: beforeCenter) }
Thread.sleep(forTimeInterval: 0.5)
let restoredInfo = petInfo()
let restored = restoredInfo.map { item in item.id == windowId && hypot(item.bounds.origin.x - beforeBounds.origin.x, item.bounds.origin.y - beforeBounds.origin.y) <= 2 } ?? false
let report: [String: Any] = [
    "pid": Int(app.processIdentifier),
    "window_id": windowId,
    "before_bounds": ["x": beforeBounds.origin.x, "y": beforeBounds.origin.y, "width": beforeBounds.width, "height": beforeBounds.height],
    "after_bounds": ["x": afterBounds.origin.x, "y": afterBounds.origin.y, "width": afterBounds.width, "height": afterBounds.height],
    "target_bounds": ["x": targetBounds.origin.x, "y": targetBounds.origin.y, "width": targetBounds.width, "height": targetBounds.height],
    "display_bounds": ["x": display.origin.x, "y": display.origin.y, "width": display.width, "height": display.height],
    "movement_points": hypot(afterBounds.origin.x - beforeBounds.origin.x, afterBounds.origin.y - beforeBounds.origin.y),
    "native_drag_moved_window": moved,
    "window_within_screen": withinScreen,
    "task_menu_not_opened": menuDidNotOpen,
    "pet_window_visible": stillVisible,
    "window_restored": restored,
    "passed": moved && withinScreen && menuDidNotOpen && stillVisible && restored,
]
let data = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys])
try data.write(to: output.appendingPathComponent("native-result.json"))
print(String(data: data, encoding: .utf8)!)
guard moved, withinScreen, menuDidNotOpen, stillVisible, restored else { exit(8) }
