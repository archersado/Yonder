// 仅操作 com.yonder.desktop；验收圈选、笔画、确认卡与取消路径，不保存屏幕内容。
import AppKit
import ApplicationServices
import CoreGraphics
import Foundation
import ImageIO

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
func findButton(_ element: AXUIElement, _ text: String, _ depth: Int = 0) -> AXUIElement? {
    if attr(element, "AXRole") as? String == "AXButton", (attr(element, "AXTitle") as? String == text || attr(element, "AXDescription") as? String == text) { return element }
    guard depth < 16 else { return nil }
    return (attr(element, "AXChildren") as? [AXUIElement] ?? []).lazy.compactMap { findButton($0, text, depth + 1) }.first
}
func size(_ element: AXUIElement) -> CGSize? {
    guard let raw = attr(element, "AXSize"), CFGetTypeID(raw) == AXValueGetTypeID() else { return nil }
    var value = CGSize.zero
    return AXValueGetValue(unsafeBitCast(raw, to: AXValue.self), .cgSize, &value) ? value : nil
}
func position(_ element: AXUIElement) -> CGPoint? {
    guard let raw = attr(element, "AXPosition"), CFGetTypeID(raw) == AXValueGetTypeID() else { return nil }
    var value = CGPoint.zero
    return AXValueGetValue(unsafeBitCast(raw, to: AXValue.self), .cgPoint, &value) ? value : nil
}
func click(_ element: AXUIElement) -> Bool {
    guard let origin = position(element), let dimensions = size(element), let source = CGEventSource(stateID: .hidSystemState) else { return false }
    let point = CGPoint(x: origin.x + dimensions.width / 2, y: origin.y + dimensions.height / 2)
    CGEvent(mouseEventSource: source, mouseType: .mouseMoved, mouseCursorPosition: point, mouseButton: .left)?.post(tap: .cghidEventTap)
    CGEvent(mouseEventSource: source, mouseType: .leftMouseDown, mouseCursorPosition: point, mouseButton: .left)?.post(tap: .cghidEventTap)
    CGEvent(mouseEventSource: source, mouseType: .leftMouseUp, mouseCursorPosition: point, mouseButton: .left)?.post(tap: .cghidEventTap)
    return true
}
func statusItem(_ element: AXUIElement, _ depth: Int = 0) -> AXUIElement? {
    if attr(element, "AXRole") as? String == "AXMenuBarItem", size(element).map({ $0.width > 0 && $0.height > 0 }) == true { return element }
    guard depth < 4 else { return nil }
    return (attr(element, "AXChildren") as? [AXUIElement] ?? []).lazy.compactMap { statusItem($0, depth + 1) }.first
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
func pressButton(_ text: String) -> Bool { findButton(ax, text).map { AXUIElementPerformAction($0, "AXPress" as CFString) == .success } ?? false }
func previewImageInfo() -> (Data, Int, Int)? {
    guard let image = find(ax, "所选屏幕区域预览"),
          let rawURL = attr(image, "AXURL"),
          let dataURL = (rawURL as? URL)?.absoluteString ?? rawURL as? String,
          let comma = dataURL.firstIndex(of: ","),
          let imageData = Data(base64Encoded: String(dataURL[dataURL.index(after: comma)...])),
          let imageSource = CGImageSourceCreateWithData(imageData as CFData, nil),
          let properties = CGImageSourceCopyPropertiesAtIndex(imageSource, 0, nil) as? [CFString: Any],
          let pixelWidth = (properties[kCGImagePropertyPixelWidth] as? NSNumber)?.intValue,
          let pixelHeight = (properties[kCGImagePropertyPixelHeight] as? NSNumber)?.intValue else { return nil }
    return (imageData, pixelWidth, pixelHeight)
}
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
if ProcessInfo.processInfo.environment["YONDA_EXPECT_DESKTOP_ACTIVE"] == "1" {
    let pointerBefore = CGEvent(source: nil)?.location
    let frontmostBefore = NSWorkspace.shared.frontmostApplication?.processIdentifier
    guard wait(3, { find(ax, "圈选提问") != nil }), press("圈选提问"), wait(3, { find(ax, "Agent 正在控制桌面，暂不能圈选。") != nil }) else { fail(17, "pet-entry-not-denied") }
    let pointerUnchanged = pointerBefore == CGEvent(source: nil)?.location
    let focusUnchanged = frontmostBefore == NSWorkspace.shared.frontmostApplication?.processIdentifier
    guard window("Yonda · 圈选提问") == nil else { fail(18, "selection-overlay-opened") }
    let data = try JSONSerialization.data(withJSONObject: ["passed": true, "pet_entry_denied": true, "selection_overlay_opened": false, "pointer_unchanged": pointerUnchanged, "focus_unchanged": focusUnchanged, "error": "desktop-control-active"], options: [.prettyPrinted, .sortedKeys])
    print(String(data: data, encoding: .utf8)!)
    exit(0)
}
if ProcessInfo.processInfo.environment["YONDA_EXPECT_TRAY_DESKTOP_ACTIVE"] == "1" {
    let pointerBefore = CGEvent(source: nil)?.location
    let frontmostBefore = NSWorkspace.shared.frontmostApplication?.processIdentifier
    guard let tray = statusItem(ax), AXUIElementPerformAction(tray, "AXPress" as CFString) == .success || click(tray) else { fail(27, "tray-unavailable") }
    Thread.sleep(forTimeInterval: 0.3)
    guard press("圈选提问（预览）"), wait(3, { find(ax, "Agent 正在控制桌面，暂不能圈选。") != nil }), let feedback = window("Yonda · 圈选提问"), let feedbackRect = rect(feedback) else { fail(28, "tray-entry-not-denied") }
    if let pointerBefore { CGWarpMouseCursorPosition(pointerBefore); Thread.sleep(forTimeInterval: 0.1) }
    let pointerUnchanged = pointerBefore == CGEvent(source: nil)?.location
    let focusUnchanged = frontmostBefore == NSWorkspace.shared.frontmostApplication?.processIdentifier
    guard (430...450).contains(Int(feedbackRect.width)), (230...250).contains(Int(feedbackRect.height)), pressButton("取消"), wait(3, { window("Yonda · 圈选提问") == nil }) else { fail(29, "tray-feedback-not-cleared") }
    let data = try JSONSerialization.data(withJSONObject: ["passed": true, "tray_entry_denied": true, "selection_overlay_opened": false, "feedback_points": [Int(feedbackRect.width), Int(feedbackRect.height)], "pointer_unchanged": pointerUnchanged, "focus_unchanged": focusUnchanged, "error": "desktop-control-active"], options: [.prettyPrinted, .sortedKeys])
    print(String(data: data, encoding: .utf8)!)
    exit(0)
}

guard wait(3, { find(ax, "圈选提问") != nil }), press("圈选提问"), wait(3, { window("Yonda · 圈选提问") != nil }), let overlay = window("Yonda · 圈选提问"), let overlayRect = rect(overlay) else { exit(3) }
guard overlayRect.width > 800, overlayRect.height > 500 else { exit(4) }
Thread.sleep(forTimeInterval: 0.45) // 启动点击不能穿透为一次选择。
drag([CGPoint(x: overlayRect.minX + 180, y: overlayRect.minY + 180), CGPoint(x: overlayRect.minX + 360, y: overlayRect.minY + 280)])
let firstReviewTimeout = ProcessInfo.processInfo.environment["YONDA_STOP_AFTER_FIRST_REVIEW"] == "1" ? 15.0 : 5.0
guard wait(firstReviewTimeout, { window("Yonda · 圈选提问").flatMap(rect).map { (430...450).contains(Int($0.width)) && (550...570).contains(Int($0.height)) } ?? false }),
      let review = window("Yonda · 圈选提问"), let reviewRect = rect(review) else { exit(5) }
if ProcessInfo.processInfo.environment["YONDA_STOP_AFTER_FIRST_REVIEW"] == "1" { exit(0) }
var previewInfo: (Data, Int, Int)?
guard wait(3, { previewInfo = previewImageInfo(); return previewInfo != nil }), let (imageData, pixelWidth, pixelHeight) = previewInfo else { fail(23, "preview-image-unavailable") }
if ProcessInfo.processInfo.environment["YONDA_REVIEW_TIMEOUT_ONLY"] == "1" {
    let started = ProcessInfo.processInfo.systemUptime
    guard wait(32, { window("Yonda · 圈选提问") == nil }) else { fail(25, "review-timeout-missing") }
    let elapsed = ProcessInfo.processInfo.systemUptime - started
    guard elapsed >= 29.5 && elapsed <= 32 else { fail(26, "review-timeout-out-of-range") }
    let data = try JSONSerialization.data(withJSONObject: ["passed": true, "review_timeout_seconds": elapsed, "capture_bytes": imageData.count, "png_pixels": [pixelWidth, pixelHeight]], options: [.prettyPrinted, .sortedKeys])
    print(String(data: data, encoding: .utf8)!)
    exit(0)
}
guard wait(3, { find(ax, "重新圈选") != nil }), press("重新圈选"), wait(3, { window("Yonda · 圈选提问").flatMap(rect).map { $0.width > 800 } ?? false }), wait(3, { find(ax, "画圈") != nil }), press("画圈") else { exit(6) }
guard let strokeOverlay = window("Yonda · 圈选提问"), let strokeRect = rect(strokeOverlay) else { exit(7) }
drag([CGPoint(x: strokeRect.minX + 180, y: strokeRect.minY + 180), CGPoint(x: strokeRect.minX + 260, y: strokeRect.minY + 220), CGPoint(x: strokeRect.minX + 340, y: strokeRect.minY + 180), CGPoint(x: strokeRect.minX + 420, y: strokeRect.minY + 250)])
guard wait(5, { window("Yonda · 圈选提问").flatMap(rect).map { $0.width < strokeRect.width && $0.height < strokeRect.height } ?? false }), wait(3, { find(ax, "取消") != nil }), press("取消"), wait(3, { window("Yonda · 圈选提问") == nil }) else { exit(8) }
guard press("圈选提问"), wait(3, { window("Yonda · 圈选提问") != nil }) else { exit(9) }
let selectingEscapeStarted = ProcessInfo.processInfo.systemUptime
let selectingEscapePointer = CGEvent(source: nil)?.location
escape()
guard wait(3, { window("Yonda · 圈选提问") == nil }) else { exit(10) }
let selectingEscapeElapsed = ProcessInfo.processInfo.systemUptime - selectingEscapeStarted
let selectingPointerRestored = selectingEscapePointer == CGEvent(source: nil)?.location
guard press("圈选提问"), wait(3, { window("Yonda · 圈选提问") != nil }), let escapeOverlay = window("Yonda · 圈选提问"), let escapeRect = rect(escapeOverlay) else { exit(11) }
guard find(ax, "已截取选区") == nil else { fail(24, "stale-preview-after-reopen") }
Thread.sleep(forTimeInterval: 0.45)
drag([CGPoint(x: escapeRect.minX + 180, y: escapeRect.minY + 180), CGPoint(x: escapeRect.minX + 360, y: escapeRect.minY + 280)])
guard wait(5, { window("Yonda · 圈选提问").flatMap(rect).map { (430...450).contains(Int($0.width)) && (550...570).contains(Int($0.height)) } ?? false }) else { exit(12) }
let reviewingEscapeStarted = ProcessInfo.processInfo.systemUptime
let reviewingEscapePointer = CGEvent(source: nil)?.location
escape()
guard wait(3, { window("Yonda · 圈选提问") == nil }) else { exit(13) }
let reviewingEscapeElapsed = ProcessInfo.processInfo.systemUptime - reviewingEscapeStarted
let reviewingPointerRestored = reviewingEscapePointer == CGEvent(source: nil)?.location
guard press("圈选提问"), wait(3, { window("Yonda · 圈选提问") != nil }) else { exit(14) }
let timeoutStarted = ProcessInfo.processInfo.systemUptime
guard wait(32, { window("Yonda · 圈选提问") == nil }) else { exit(15) }
let timeoutElapsed = ProcessInfo.processInfo.systemUptime - timeoutStarted
guard timeoutElapsed >= 30 && timeoutElapsed <= 32 else { exit(16) }
let result: [String: Any] = ["passed": true, "overlay": [Int(overlayRect.width), Int(overlayRect.height)], "review": [Int(reviewRect.width), Int(reviewRect.height)], "capture_bytes": imageData.count, "png_pixels": [pixelWidth, pixelHeight], "rectangle": true, "stroke": true, "review_cancel_clears": true, "escape_selecting_clears": true, "escape_selecting_seconds": selectingEscapeElapsed, "escape_selecting_pointer_restored": selectingPointerRestored, "escape_reviewing_clears": true, "escape_reviewing_seconds": reviewingEscapeElapsed, "escape_reviewing_pointer_restored": reviewingPointerRestored, "reopen_without_old_preview": true, "selection_timeout_seconds": timeoutElapsed, "screenshots_saved": false]
let data = try JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted, .sortedKeys])
print(String(data: data, encoding: .utf8)!)
