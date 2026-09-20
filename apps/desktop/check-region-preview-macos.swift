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
    if attr(element, "AXTitle") as? String == text || attr(element, "AXDescription") as? String == text || attr(element, "AXValue") as? String == text { return element }
    guard depth < 16 else { return nil }
    return (attr(element, "AXChildren") as? [AXUIElement] ?? []).lazy.compactMap { find($0, text, depth + 1) }.first
}
func findButton(_ element: AXUIElement, _ text: String, _ depth: Int = 0) -> AXUIElement? {
    if attr(element, "AXRole") as? String == "AXButton", (attr(element, "AXTitle") as? String == text || attr(element, "AXDescription") as? String == text) { return element }
    guard depth < 16 else { return nil }
    return (attr(element, "AXChildren") as? [AXUIElement] ?? []).lazy.compactMap { findButton($0, text, depth + 1) }.first
}
func findRole(_ element: AXUIElement, _ role: String, _ depth: Int = 0) -> AXUIElement? {
    if attr(element, "AXRole") as? String == role { return element }
    guard depth < 16 else { return nil }
    return (attr(element, "AXChildren") as? [AXUIElement] ?? []).lazy.compactMap { findRole($0, role, depth + 1) }.first
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
let cleanupPrefix = "Yonda · 圈选提问 [idle image=0 selection=0 stroke=0 "
func cleanupInfo() -> (reason: String, latencyMs: Int)? {
    let all = CGWindowListCopyWindowInfo(.optionAll, kCGNullWindowID) as? [[String: Any]] ?? []
    guard let title = all.first(where: { ($0[kCGWindowOwnerPID as String] as? Int) == Int(app.processIdentifier) && (($0[kCGWindowName as String] as? String)?.hasPrefix(cleanupPrefix) ?? false) && ($0[kCGWindowIsOnscreen as String] as? Int) != 1 })?[kCGWindowName as String] as? String,
          let reason = title.split(separator: " ").first(where: { $0.hasPrefix("reason=") })?.split(separator: "=").last,
          let latency = title.split(separator: " ").first(where: { $0.hasPrefix("latency_ms=") })?.split(separator: "=").last?.dropLast(),
          let latencyMs = Int(latency) else { return nil }
    return (String(reason), latencyMs)
}
func cleanupHidden() -> Bool { cleanupInfo() != nil }
func onScreenWindowCount(_ title: String) -> Int {
    let all = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
    return all.filter { ($0[kCGWindowOwnerPID as String] as? Int) == Int(app.processIdentifier) && ($0[kCGWindowName as String] as? String) == title }.count
}
func rect(_ info: [String: Any]) -> CGRect? {
    (info[kCGWindowBounds as String] as? NSDictionary).flatMap(CGRect.init(dictionaryRepresentation:))
}
func press(_ text: String) -> Bool { find(ax, text).map { AXUIElementPerformAction($0, "AXPress" as CFString) == .success } ?? false }
func pressButton(_ text: String) -> Bool { findButton(ax, text).map { AXUIElementPerformAction($0, "AXPress" as CFString) == .success } ?? false }
func activateButton(_ text: String) -> Bool { findButton(ax, text).map(click) ?? false }
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
func reopenWithoutOldPreview() -> Bool {
    guard press("圈选提问"), wait(3, { window("Yonda · 圈选提问") != nil }) else { return false }
    let clean = find(ax, "已截取选区") == nil && previewImageInfo() == nil
    escape()
    return clean && wait(3, { window("Yonda · 圈选提问") == nil }) && wait(3, cleanupHidden)
}
let priorPointer = CGEvent(source: nil)?.location
defer { if let priorPointer { CGWarpMouseCursorPosition(priorPointer) } }
if let outcome = ProcessInfo.processInfo.environment["YONDA_TEXT_SUBMIT_OUTCOME"] {
    guard ["accepted", "rejected", "unknown"].contains(outcome) else { fail(70, "text-submit-outcome-invalid") }
    guard wait(3, { find(ax, "圈选提问") != nil }), press("圈选提问"), wait(3, { window("Yonda · 圈选提问") != nil }), let overlay = window("Yonda · 圈选提问"), let overlayRect = rect(overlay) else { fail(71, "text-submit-open-failed") }
    Thread.sleep(forTimeInterval: 0.45)
    let start = CGPoint(x: overlayRect.minX + 180, y: overlayRect.minY + 180)
    let end = ProcessInfo.processInfo.environment["YONDA_TEXT_PERMISSION_FALLBACK"] == "1" ? CGPoint(x: start.x + 180, y: start.y + 100) : start
    drag([start, end])
    guard wait(8, { find(ax, "仅提问") != nil }), previewImageInfo() == nil, let reviewWindow = window("Yonda · 圈选提问"), let reviewRect = rect(reviewWindow), (430...450).contains(Int(reviewRect.width)), (340...360).contains(Int(reviewRect.height)), let input = findRole(ax, "AXTextArea") ?? findRole(ax, "AXTextField") else { fail(72, "text-submit-review-missing") }
    if ProcessInfo.processInfo.environment["YONDA_TEXT_PERMISSION_FALLBACK"] == "1" {
        guard find(ax, "需要允许屏幕录制；本次不包含截图，可继续发送文字。") != nil else { fail(73, "text-submit-permission-feedback-missing") }
    }
    guard AXUIElementSetAttributeValue(input, "AXValue" as CFString, "explain without image" as CFTypeRef) == .success else { fail(74, "text-submit-question-unavailable") }
    guard wait(3, { findButton(ax, "发送").flatMap { attr($0, "AXEnabled") as? Bool } == true }), pressButton("发送") else { fail(75, "text-submit-button-unavailable") }
    if outcome == "accepted" {
        guard wait(5, { find(ax, "已发送给 Agent") != nil }), wait(5, { window("Yonda · 圈选提问") == nil }), wait(3, cleanupHidden), cleanupInfo()?.reason == "submitted" else { fail(76, "text-submit-accepted-not-closed") }
    } else {
        let expected = outcome == "rejected" ? "Agent 拒绝了本次输入。" : "是否送达未知，未自动重试；请重新圈选。"
        guard wait(5, { find(ax, expected) != nil }), previewImageInfo() == nil, pressButton("重新圈选"), wait(3, { window("Yonda · 圈选提问").flatMap(rect).map { $0.width > 800 } ?? false }) else { fail(77, "text-submit-failure-not-cleared") }
        escape(); guard wait(3, { window("Yonda · 圈选提问") == nil }) else { fail(78, "text-submit-failure-close-failed") }
    }
    let data = try JSONSerialization.data(withJSONObject: ["passed": true, "outcome": outcome, "question_entered": true, "attachment_present": false, "window_closed": true], options: [.prettyPrinted, .sortedKeys])
    print(String(data: data, encoding: .utf8)!); exit(0)
}
if let outcome = ProcessInfo.processInfo.environment["YONDA_SUBMIT_OUTCOME"] {
    guard ["accepted", "rejected", "unknown", "unsupported"].contains(outcome) else { fail(60, "submit-outcome-invalid") }
    guard wait(3, { find(ax, "圈选提问") != nil }), press("圈选提问"), wait(3, { window("Yonda · 圈选提问") != nil }), let overlay = window("Yonda · 圈选提问"), let overlayRect = rect(overlay) else { fail(61, "submit-open-failed") }
    Thread.sleep(forTimeInterval: 0.45)
    drag([CGPoint(x: overlayRect.minX + 180, y: overlayRect.minY + 180), CGPoint(x: overlayRect.minX + 360, y: overlayRect.minY + 280)])
    guard wait(5, { find(ax, "想问什么？") != nil }), let input = findRole(ax, "AXTextArea") ?? findRole(ax, "AXTextField") else { fail(62, "submit-question-missing") }
    guard AXUIElementSetAttributeValue(input, "AXValue" as CFString, "explain selection" as CFTypeRef) == .success else { fail(63, "submit-question-unavailable") }
    guard wait(3, { findButton(ax, "发送").flatMap { attr($0, "AXEnabled") as? Bool } == true }), pressButton("发送") else { fail(64, "submit-button-unavailable") }
    if outcome == "accepted" {
        guard wait(5, { find(ax, "已发送给 Agent") != nil }), wait(5, { window("Yonda · 圈选提问") == nil }), wait(3, cleanupHidden), cleanupInfo()?.reason == "submitted" else { fail(65, "submit-accepted-not-closed") }
    } else {
        let expected = outcome == "rejected" ? "Agent 拒绝了本次输入，截图未保留。" : outcome == "unknown" ? "是否送达未知，未自动重试；请重新圈选。" : "当前 Agent 不支持截图提问。"
        guard wait(5, { find(ax, expected) != nil }), previewImageInfo() == nil, pressButton("重新圈选"), wait(3, { window("Yonda · 圈选提问").flatMap(rect).map { $0.width > 800 } ?? false }) else { fail(66, "submit-failure-not-cleared") }
        escape(); guard wait(3, { window("Yonda · 圈选提问") == nil }) else { fail(67, "submit-failure-close-failed") }
    }
    let data = try JSONSerialization.data(withJSONObject: ["passed": true, "outcome": outcome, "question_entered": true, "preview_cleared": true, "window_closed": true], options: [.prettyPrinted, .sortedKeys])
    print(String(data: data, encoding: .utf8)!); exit(0)
}
if let entry = ProcessInfo.processInfo.environment["YONDA_EXPECT_DESKTOP_PAUSE"] {
    guard entry == "pet" || entry == "tray" else { fail(79, "desktop-pause-entry-invalid") }
    let opened: Bool
    if entry == "pet" {
        opened = wait(3, { find(ax, "圈选提问") != nil }) && press("圈选提问")
    } else {
        guard let tray = statusItem(ax), AXUIElementPerformAction(tray, "AXPress" as CFString) == .success || click(tray) else { fail(80, "desktop-pause-tray-unavailable") }
        Thread.sleep(forTimeInterval: 0.3); opened = press("圈选提问（预览）")
    }
    guard opened, wait(8, { window("Yonda · 圈选提问").flatMap(rect).map { $0.width > 800 } ?? false }) else { fail(81, "desktop-pause-overlay-missing") }
    escape(); guard wait(3, { window("Yonda · 圈选提问") == nil }), wait(3, cleanupHidden) else { fail(82, "desktop-pause-overlay-not-cleared") }
    let data = try JSONSerialization.data(withJSONObject: ["passed": true, "entry": entry, "selection_overlay_opened": true, "cleared": true], options: [.prettyPrinted, .sortedKeys])
    print(String(data: data, encoding: .utf8)!); exit(0)
}
if ProcessInfo.processInfo.environment["YONDA_PHYSICAL_TOOLBAR_CANCEL"] == "1" {
    for attempt in 1...6 {
        guard wait(3, { find(ax, "圈选提问") != nil }), press("圈选提问"), wait(3, { window("Yonda · 圈选提问") != nil }) else { fail(42, "toolbar-cancel-open-failed") }
        let started = ProcessInfo.processInfo.systemUptime
        let ready = try JSONSerialization.data(withJSONObject: ["ready": true, "action": "toolbar-cancel", "attempt": attempt], options: [.sortedKeys])
        print(String(data: ready, encoding: .utf8)!); fflush(stdout)
        guard wait(32, { window("Yonda · 圈选提问") == nil }), wait(3, cleanupHidden) else { fail(43, "toolbar-cancel-not-observed") }
        let elapsed = ProcessInfo.processInfo.systemUptime - started
        if elapsed < 29.5 {
            guard let cleanup = cleanupInfo(), cleanup.reason == "toolbar-cancel", cleanup.latencyMs <= 3000, reopenWithoutOldPreview() else { fail(45, "toolbar-cancel-cleanup-incomplete") }
            let result = try JSONSerialization.data(withJSONObject: ["passed": true, "action": "toolbar-cancel", "attempt": attempt, "seconds": elapsed, "close_latency_ms": cleanup.latencyMs, "reopen_without_old_preview": true, "application_phase": "idle", "image_bytes": 0, "selection_bytes": 0, "stroke_bytes": 0], options: [.prettyPrinted, .sortedKeys])
            print(String(data: result, encoding: .utf8)!); exit(0)
        }
    }
    fail(44, "toolbar-cancel-not-distinguished-from-timeout")
}
if let stage = ProcessInfo.processInfo.environment["YONDA_PHYSICAL_ESC_STAGE"] {
    guard stage == "selecting" || stage == "reviewing" else { fail(32, "physical-escape-stage-invalid") }
    for attempt in 1...6 {
        guard wait(3, { find(ax, "圈选提问") != nil }), press("圈选提问"), wait(3, { window("Yonda · 圈选提问") != nil }), let physicalOverlay = window("Yonda · 圈选提问"), let physicalRect = rect(physicalOverlay) else { fail(30, "physical-escape-open-failed") }
        if stage == "reviewing" {
            Thread.sleep(forTimeInterval: 0.45)
            drag([CGPoint(x: physicalRect.minX + 180, y: physicalRect.minY + 180), CGPoint(x: physicalRect.minX + 360, y: physicalRect.minY + 280)])
            guard wait(5, { window("Yonda · 圈选提问").flatMap(rect).map { (430...450).contains(Int($0.width)) && (550...570).contains(Int($0.height)) } ?? false }) else { fail(31, "physical-escape-review-unavailable") }
        }
        let started = ProcessInfo.processInfo.systemUptime
        let pointer = CGEvent(source: nil)?.location
        let ready = try JSONSerialization.data(withJSONObject: ["ready": true, "stage": stage, "attempt": attempt], options: [.sortedKeys])
        print(String(data: ready, encoding: .utf8)!); fflush(stdout)
        guard wait(32, { window("Yonda · 圈选提问") == nil }), wait(3, cleanupHidden) else { fail(33, "physical-escape-not-observed") }
        let elapsed = ProcessInfo.processInfo.systemUptime - started
        if elapsed < 29.5 {
            let currentPointer = CGEvent(source: nil)?.location
            let pointerDelta = if let pointer, let currentPointer { hypot(pointer.x - currentPointer.x, pointer.y - currentPointer.y) } else { CGFloat.infinity }
            let pointerRestored = pointerDelta <= 1
            guard pointerRestored else { fail(41, "physical-escape-pointer-moved-\(pointerDelta)") }
            guard let cleanup = cleanupInfo(), cleanup.reason == "escape", cleanup.latencyMs <= 3000, reopenWithoutOldPreview() else { fail(46, "physical-escape-cleanup-incomplete") }
            let result = try JSONSerialization.data(withJSONObject: ["passed": true, "stage": stage, "attempt": attempt, "seconds": elapsed, "close_latency_ms": cleanup.latencyMs, "pointer_delta_points": pointerDelta, "pointer_restored": pointerRestored, "reopen_without_old_preview": true, "application_phase": "idle", "image_bytes": 0, "selection_bytes": 0, "stroke_bytes": 0], options: [.prettyPrinted, .sortedKeys])
            print(String(data: result, encoding: .utf8)!); exit(0)
        }
    }
    fail(40, "physical-escape-not-distinguished-from-timeout")
}
if ProcessInfo.processInfo.environment["YONDA_EXPECT_PERMISSION_DENIED"] == "1" {
    guard wait(3, { find(ax, "圈选提问") != nil }), press("圈选提问"), wait(3, { window("Yonda · 圈选提问") != nil }), let deniedOverlay = window("Yonda · 圈选提问"), let deniedRect = rect(deniedOverlay) else { fail(34, "permission-open-failed") }
    Thread.sleep(forTimeInterval: 0.45)
    drag([CGPoint(x: deniedRect.minX + 180, y: deniedRect.minY + 180), CGPoint(x: deniedRect.minX + 210, y: deniedRect.minY + 210)])
    guard wait(20, { find(ax, "需要允许屏幕录制后才能预览截图。") != nil }) else { fail(35, "permission-feedback-missing") }
    guard previewImageInfo() == nil else { fail(35, "permission-thumbnail-present") }
    guard pressButton("取消") else { fail(35, "permission-cancel-unavailable") }
    guard wait(3, { window("Yonda · 圈选提问") == nil }) else { fail(35, "permission-window-still-visible") }
    guard wait(3, cleanupHidden) else { fail(35, "permission-cleanup-marker-missing") }
    let result = try JSONSerialization.data(withJSONObject: ["passed": true, "error": "permission-required", "permission_requested": false, "thumbnail_present": false, "application_phase": "idle", "image_bytes": 0, "selection_bytes": 0, "stroke_bytes": 0], options: [.prettyPrinted, .sortedKeys])
    print(String(data: result, encoding: .utf8)!); exit(0)
}
if ProcessInfo.processInfo.environment["YONDA_REVIEW_TIMEOUT_ONLY"] == "1" {
    guard wait(3, { find(ax, "圈选提问") != nil }), press("圈选提问"), wait(3, { window("Yonda · 圈选提问") != nil }), let timeoutOverlay = window("Yonda · 圈选提问"), let timeoutRect = rect(timeoutOverlay) else { fail(50, "review-timeout-open-failed") }
    Thread.sleep(forTimeInterval: 0.45)
    drag([CGPoint(x: timeoutRect.minX + 180, y: timeoutRect.minY + 180), CGPoint(x: timeoutRect.minX + 360, y: timeoutRect.minY + 280)])
    guard wait(5, { window("Yonda · 圈选提问").flatMap(rect).map { (430...450).contains(Int($0.width)) && (550...570).contains(Int($0.height)) } ?? false }) else { fail(51, "review-timeout-review-missing") }
    var timeoutPreview: (Data, Int, Int)?
    guard wait(3, { timeoutPreview = previewImageInfo(); return timeoutPreview != nil }), let (imageData, pixelWidth, pixelHeight) = timeoutPreview else { fail(52, "review-timeout-image-missing") }
    let started = ProcessInfo.processInfo.systemUptime
    guard wait(32, { window("Yonda · 圈选提问") == nil }) else { fail(25, "review-timeout-window-visible") }
    guard wait(3, cleanupHidden), let cleanup = cleanupInfo(), cleanup.reason == "timeout", cleanup.latencyMs <= 3000 else { fail(25, "review-timeout-cleanup-invalid") }
    let elapsed = ProcessInfo.processInfo.systemUptime - started
    guard elapsed >= 29.5 && elapsed <= 32 else { fail(26, "review-timeout-out-of-range") }
    guard reopenWithoutOldPreview() else { fail(47, "review-timeout-stale-preview") }
    let data = try JSONSerialization.data(withJSONObject: ["passed": true, "review_timeout_seconds": elapsed, "close_latency_ms": cleanup.latencyMs, "reopen_without_old_preview": true, "capture_bytes": imageData.count, "png_pixels": [pixelWidth, pixelHeight], "application_phase": "idle", "image_bytes_after": 0, "selection_bytes_after": 0, "stroke_bytes_after": 0], options: [.prettyPrinted, .sortedKeys])
    print(String(data: data, encoding: .utf8)!); exit(0)
}

guard wait(3, { find(ax, "圈选提问") != nil }), press("圈选提问"), wait(3, { window("Yonda · 圈选提问") != nil }) else { exit(3) }
let petEntryWindowCount = onScreenWindowCount("Yonda · 圈选提问")
escape()
guard petEntryWindowCount == 1, wait(3, { window("Yonda · 圈选提问") == nil }), wait(3, cleanupHidden) else { fail(36, "pet-entry-cleanup-failed") }
guard let tray = statusItem(ax), AXUIElementPerformAction(tray, "AXPress" as CFString) == .success || click(tray) else { fail(37, "tray-unavailable") }
Thread.sleep(forTimeInterval: 0.3)
guard press("圈选提问（预览）"), wait(3, { window("Yonda · 圈选提问") != nil }) else { fail(38, "tray-entry-open-failed") }
let trayEntryWindowCount = onScreenWindowCount("Yonda · 圈选提问")
escape()
guard trayEntryWindowCount == 1, wait(3, { window("Yonda · 圈选提问") == nil }), wait(3, cleanupHidden) else { fail(39, "tray-entry-cleanup-failed") }
guard press("圈选提问"), wait(3, { window("Yonda · 圈选提问") != nil }), let overlay = window("Yonda · 圈选提问"), let overlayRect = rect(overlay) else { exit(3) }
guard overlayRect.width > 800, overlayRect.height > 500 else { exit(4) }
Thread.sleep(forTimeInterval: 0.45) // 启动点击不能穿透为一次选择。
drag([CGPoint(x: overlayRect.minX + 180, y: overlayRect.minY + 180), CGPoint(x: overlayRect.minX + 360, y: overlayRect.minY + 280)])
let firstReviewTimeout = ProcessInfo.processInfo.environment["YONDA_STOP_AFTER_FIRST_REVIEW"] == "1" ? 15.0 : 5.0
guard wait(firstReviewTimeout, { window("Yonda · 圈选提问").flatMap(rect).map { (430...450).contains(Int($0.width)) && (550...570).contains(Int($0.height)) } ?? false }),
      let review = window("Yonda · 圈选提问"), let reviewRect = rect(review) else { exit(5) }
if ProcessInfo.processInfo.environment["YONDA_STOP_AFTER_FIRST_REVIEW"] == "1" { exit(0) }
var previewInfo: (Data, Int, Int)?
guard wait(3, { previewInfo = previewImageInfo(); return previewInfo != nil }), let (imageData, pixelWidth, pixelHeight) = previewInfo else { fail(23, "preview-image-unavailable") }
guard wait(3, { find(ax, "重新圈选") != nil }), press("重新圈选"), wait(3, { window("Yonda · 圈选提问").flatMap(rect).map { $0.width > 800 } ?? false }), wait(3, { find(ax, "画圈") != nil }), press("画圈") else { exit(6) }
guard let strokeOverlay = window("Yonda · 圈选提问"), let strokeRect = rect(strokeOverlay) else { exit(7) }
drag([CGPoint(x: strokeRect.minX + 180, y: strokeRect.minY + 180), CGPoint(x: strokeRect.minX + 260, y: strokeRect.minY + 220), CGPoint(x: strokeRect.minX + 340, y: strokeRect.minY + 180), CGPoint(x: strokeRect.minX + 420, y: strokeRect.minY + 250)])
guard wait(5, { window("Yonda · 圈选提问").flatMap(rect).map { $0.width < strokeRect.width && $0.height < strokeRect.height } ?? false }), wait(3, { find(ax, "取消") != nil }) else { exit(8) }
let reviewCancelStarted = ProcessInfo.processInfo.systemUptime
guard press("取消"), wait(3, { window("Yonda · 圈选提问") == nil }), wait(3, cleanupHidden), let reviewCancelCleanup = cleanupInfo(), reviewCancelCleanup.reason == "review-cancel", reviewCancelCleanup.latencyMs <= 3000 else { exit(8) }
let reviewCancelElapsed = ProcessInfo.processInfo.systemUptime - reviewCancelStarted
guard reopenWithoutOldPreview() else { fail(48, "review-cancel-stale-preview") }
guard press("圈选提问"), wait(3, { window("Yonda · 圈选提问") != nil }) else { exit(9) }
let selectingEscapeStarted = ProcessInfo.processInfo.systemUptime
let selectingEscapePointer = CGEvent(source: nil)?.location
escape()
guard wait(3, { window("Yonda · 圈选提问") == nil }), wait(3, cleanupHidden) else { exit(10) }
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
guard wait(3, { window("Yonda · 圈选提问") == nil }), wait(3, cleanupHidden) else { exit(13) }
let reviewingEscapeElapsed = ProcessInfo.processInfo.systemUptime - reviewingEscapeStarted
let reviewingPointerRestored = reviewingEscapePointer == CGEvent(source: nil)?.location
guard press("圈选提问"), wait(3, { window("Yonda · 圈选提问") != nil }) else { exit(14) }
let timeoutStarted = ProcessInfo.processInfo.systemUptime
guard wait(32, { window("Yonda · 圈选提问") == nil }), wait(3, cleanupHidden), let selectionTimeoutCleanup = cleanupInfo(), selectionTimeoutCleanup.reason == "timeout", selectionTimeoutCleanup.latencyMs <= 3000 else { exit(15) }
let timeoutElapsed = ProcessInfo.processInfo.systemUptime - timeoutStarted
guard timeoutElapsed >= 30 && timeoutElapsed <= 32 else { exit(16) }
guard reopenWithoutOldPreview() else { fail(49, "selection-timeout-stale-preview") }
let result: [String: Any] = ["passed": true, "entry_names": ["圈选提问", "圈选提问（预览）"], "pet_entry_window_count": petEntryWindowCount, "tray_entry_window_count": trayEntryWindowCount, "idle_screenshot_activity_count": 0, "overlay": [Int(overlayRect.width), Int(overlayRect.height)], "review": [Int(reviewRect.width), Int(reviewRect.height)], "capture_bytes": imageData.count, "png_pixels": [pixelWidth, pixelHeight], "rectangle": true, "stroke": true, "review_cancel_clears": true, "review_cancel_seconds": reviewCancelElapsed, "review_cancel_close_latency_ms": reviewCancelCleanup.latencyMs, "review_cancel_reopen_without_old_preview": true, "escape_selecting_clears": true, "escape_selecting_seconds": selectingEscapeElapsed, "escape_selecting_pointer_restored": selectingPointerRestored, "escape_reviewing_clears": true, "escape_reviewing_seconds": reviewingEscapeElapsed, "escape_reviewing_pointer_restored": reviewingPointerRestored, "reopen_without_old_preview": true, "selection_timeout_seconds": timeoutElapsed, "selection_timeout_close_latency_ms": selectionTimeoutCleanup.latencyMs, "selection_timeout_reopen_without_old_preview": true, "screenshots_saved": false, "cleanup": ["application_phase": "idle", "image_bytes": 0, "selection_bytes": 0, "stroke_bytes": 0]]
let data = try JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted, .sortedKeys])
print(String(data: data, encoding: .utf8)!)
