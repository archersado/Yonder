// 仅操作 com.yonder.desktop；验收圈选、笔画、确认卡与取消路径，不保存屏幕内容。
import AppKit
import ApplicationServices
import CoreGraphics
import Foundation

func fail(_ code: Int32, _ reason: String) -> Never {
    let data = try! JSONSerialization.data(withJSONObject: ["passed": false, "reason": reason], options: [.sortedKeys])
    print(String(data: data, encoding: .utf8)!)
    exit(code)
}
guard AXIsProcessTrusted() else { fail(2, "accessibility-denied") }
guard CGPreflightPostEventAccess() else { fail(3, "input-event-denied") }
let bundleID = ProcessInfo.processInfo.environment["YONDA_BUNDLE_ID"] ?? "com.yonder.desktop"
guard let app = NSRunningApplication.runningApplications(withBundleIdentifier: bundleID).first else { fail(4, "preview-not-running") }
let ax = AXUIElementCreateApplication(app.processIdentifier)
func attr(_ element: AXUIElement, _ name: String) -> CFTypeRef? {
    var value: CFTypeRef?; return AXUIElementCopyAttributeValue(element, name as CFString, &value) == .success ? value : nil
}
func find(_ element: AXUIElement, _ text: String, _ depth: Int = 0) -> AXUIElement? {
    if attr(element, "AXTitle") as? String == text || attr(element, "AXDescription") as? String == text { return element }
    guard depth < 16 else { return nil }
    return (attr(element, "AXChildren") as? [AXUIElement] ?? []).lazy.compactMap { find($0, text, depth + 1) }.first
}
func wait(_ seconds: TimeInterval, _ condition: () -> Bool) -> Bool {
    let end = ProcessInfo.processInfo.systemUptime + seconds
    while ProcessInfo.processInfo.systemUptime < end { if condition() { return true }; Thread.sleep(forTimeInterval: 0.1) }
    return condition()
}
func window(_ title: String) -> [String: Any]? {
    let all = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
    return all.first { ($0[kCGWindowOwnerPID as String] as? Int) == Int(app.processIdentifier) && ($0[kCGWindowName as String] as? String) == title }
}
func rect(_ info: [String: Any]) -> CGRect? {
    (info[kCGWindowBounds as String] as? NSDictionary).flatMap(CGRect.init(dictionaryRepresentation:))
}
func press(_ text: String) -> Bool { find(ax, text).map { AXUIElementPerformAction($0, "AXPress" as CFString) == .success } ?? false }
func drag(_ points: [CGPoint]) {
    guard let first = points.first, let last = points.last else { return }
    let source = CGEventSource(stateID: .hidSystemState)
    CGEvent(mouseEventSource: source, mouseType: .leftMouseDown, mouseCursorPosition: first, mouseButton: .left)?.post(tap: .cghidEventTap)
    for point in points.dropFirst() { CGEvent(mouseEventSource: source, mouseType: .leftMouseDragged, mouseCursorPosition: point, mouseButton: .left)?.post(tap: .cghidEventTap); Thread.sleep(forTimeInterval: 0.04) }
    CGEvent(mouseEventSource: source, mouseType: .leftMouseUp, mouseCursorPosition: last, mouseButton: .left)?.post(tap: .cghidEventTap)
}
func escape() {
    let source = CGEventSource(stateID: .hidSystemState)
    CGEvent(keyboardEventSource: source, virtualKey: 53, keyDown: true)?.post(tap: .cghidEventTap)
    CGEvent(keyboardEventSource: source, virtualKey: 53, keyDown: false)?.post(tap: .cghidEventTap)
}
let priorPointer = CGEvent(source: nil)?.location
defer { if let priorPointer { CGWarpMouseCursorPosition(priorPointer) } }

guard wait(3, { find(ax, "圈选提问") != nil }), press("圈选提问"), wait(3, { window("Yonda · 圈选提问") != nil }), let overlay = window("Yonda · 圈选提问"), let overlayRect = rect(overlay) else { exit(3) }
guard overlayRect.width > 800, overlayRect.height > 500 else { exit(4) }
Thread.sleep(forTimeInterval: 0.45) // 启动点击不能穿透为一次选择。
drag([CGPoint(x: overlayRect.minX + 180, y: overlayRect.minY + 180), CGPoint(x: overlayRect.minX + 360, y: overlayRect.minY + 280)])
guard wait(5, { window("Yonda · 圈选提问").flatMap(rect).map { (430...450).contains(Int($0.width)) && (550...570).contains(Int($0.height)) } ?? false }),
      let review = window("Yonda · 圈选提问"), let reviewRect = rect(review) else { exit(5) }
guard wait(3, { find(ax, "重新圈选") != nil }), press("重新圈选"), wait(3, { window("Yonda · 圈选提问").flatMap(rect).map { $0.width > 800 } ?? false }), wait(3, { find(ax, "画圈") != nil }), press("画圈") else { exit(6) }
guard let strokeOverlay = window("Yonda · 圈选提问"), let strokeRect = rect(strokeOverlay) else { exit(7) }
drag([CGPoint(x: strokeRect.minX + 180, y: strokeRect.minY + 180), CGPoint(x: strokeRect.minX + 260, y: strokeRect.minY + 220), CGPoint(x: strokeRect.minX + 340, y: strokeRect.minY + 180), CGPoint(x: strokeRect.minX + 420, y: strokeRect.minY + 250)])
guard wait(5, { window("Yonda · 圈选提问").flatMap(rect).map { $0.width < strokeRect.width && $0.height < strokeRect.height } ?? false }), wait(3, { find(ax, "取消") != nil }), press("取消"), wait(3, { window("Yonda · 圈选提问") == nil }) else { exit(8) }
guard press("圈选提问"), wait(3, { window("Yonda · 圈选提问") != nil }) else { exit(9) }
escape()
guard wait(3, { window("Yonda · 圈选提问") == nil }) else { exit(10) }
guard press("圈选提问"), wait(3, { window("Yonda · 圈选提问") != nil }), let escapeOverlay = window("Yonda · 圈选提问"), let escapeRect = rect(escapeOverlay) else { exit(11) }
Thread.sleep(forTimeInterval: 0.45)
drag([CGPoint(x: escapeRect.minX + 180, y: escapeRect.minY + 180), CGPoint(x: escapeRect.minX + 360, y: escapeRect.minY + 280)])
guard wait(5, { window("Yonda · 圈选提问").flatMap(rect).map { (430...450).contains(Int($0.width)) && (550...570).contains(Int($0.height)) } ?? false }) else { exit(12) }
escape()
guard wait(3, { window("Yonda · 圈选提问") == nil }) else { exit(13) }
guard press("圈选提问"), wait(3, { window("Yonda · 圈选提问") != nil }) else { exit(14) }
let timeoutStarted = ProcessInfo.processInfo.systemUptime
guard wait(32, { window("Yonda · 圈选提问") == nil }) else { exit(15) }
let timeoutElapsed = ProcessInfo.processInfo.systemUptime - timeoutStarted
guard timeoutElapsed >= 30 && timeoutElapsed <= 32 else { exit(16) }
let result: [String: Any] = ["passed": true, "overlay": [Int(overlayRect.width), Int(overlayRect.height)], "review": [Int(reviewRect.width), Int(reviewRect.height)], "rectangle": true, "stroke": true, "cancel_clears": true, "escape_selecting_clears": true, "escape_reviewing_clears": true, "selection_timeout_seconds": timeoutElapsed, "screenshots_saved": false]
let data = try JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted, .sortedKeys])
print(String(data: data, encoding: .utf8)!)
