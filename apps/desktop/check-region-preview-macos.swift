// 仅操作 com.yonder.desktop；验收圈选、笔画、确认卡与取消路径，不保存屏幕内容。
import AppKit
import ApplicationServices
import CoreGraphics
import Foundation

guard AXIsProcessTrusted(), CGPreflightPostEventAccess(),
      let app = NSRunningApplication.runningApplications(withBundleIdentifier: "com.yonder.desktop").first else { exit(2) }
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
let priorPointer = CGEvent(source: nil)?.location
defer { if let priorPointer { CGWarpMouseCursorPosition(priorPointer) } }

guard press("圈选提问"), wait(3, { window("Yonda · 圈选提问") != nil }), let overlay = window("Yonda · 圈选提问"), let overlayRect = rect(overlay) else { exit(3) }
guard overlayRect.width > 800, overlayRect.height > 500 else { exit(4) }
drag([CGPoint(x: overlayRect.minX + 180, y: overlayRect.minY + 180), CGPoint(x: overlayRect.minX + 360, y: overlayRect.minY + 280)])
guard wait(5, { window("Yonda · 圈选提问").flatMap(rect).map { $0.width == 440 && $0.height == 460 } ?? false }),
      let review = window("Yonda · 圈选提问"), let reviewRect = rect(review), find(ax, "已截取选区") != nil, find(ax, "重新圈选") != nil, find(ax, "取消") != nil else { exit(5) }
guard press("重新圈选"), wait(3, { window("Yonda · 圈选提问").flatMap(rect).map { $0.width > 800 } ?? false }), press("笔画") else { exit(6) }
guard let strokeOverlay = window("Yonda · 圈选提问"), let strokeRect = rect(strokeOverlay) else { exit(7) }
drag([CGPoint(x: strokeRect.minX + 180, y: strokeRect.minY + 180), CGPoint(x: strokeRect.minX + 260, y: strokeRect.minY + 220), CGPoint(x: strokeRect.minX + 340, y: strokeRect.minY + 180), CGPoint(x: strokeRect.minX + 420, y: strokeRect.minY + 250)])
guard wait(5, { window("Yonda · 圈选提问").flatMap(rect).map { $0.width == 440 && $0.height == 460 } ?? false }), press("取消"), wait(3, { window("Yonda · 圈选提问") == nil }) else { exit(8) }
let result: [String: Any] = ["passed": true, "overlay": [Int(overlayRect.width), Int(overlayRect.height)], "review": [Int(reviewRect.width), Int(reviewRect.height)], "rectangle": true, "stroke": true, "cancel_clears": true, "screenshots_saved": false]
let data = try JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted, .sortedKeys])
print(String(data: data, encoding: .utf8)!)
