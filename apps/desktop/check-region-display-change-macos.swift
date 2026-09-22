// 仅验证确认卡在显示器参数变化后清场；分辨率立即恢复，不保存屏幕内容。
import AppKit
import CoreGraphics
import Foundation

func fail(_ code: Int32, _ reason: String) -> Never {
    print("{\"passed\":false,\"reason\":\"\(reason)\"}")
    exit(code)
}

let bundleID = ProcessInfo.processInfo.environment["YONDA_BUNDLE_ID"] ?? "com.yonder.desktop"
guard let app = NSRunningApplication.runningApplications(withBundleIdentifier: bundleID).first else { fail(4, "preview-not-running") }

func window(_ title: String) -> [String: Any]? {
    let all = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
    return all.first { ($0[kCGWindowOwnerPID as String] as? Int) == Int(app.processIdentifier) && ($0[kCGWindowName as String] as? String) == title }
}

let cleanupPrefix = "Yonda · 圈选提问 [idle image=0 selection=0 stroke=0 "
func cleanupInfo() -> (reason: String, latencyMs: Int)? {
    let all = CGWindowListCopyWindowInfo(.optionAll, kCGNullWindowID) as? [[String: Any]] ?? []
    guard let title = all.first(where: {
        ($0[kCGWindowOwnerPID as String] as? Int) == Int(app.processIdentifier)
            && (($0[kCGWindowName as String] as? String)?.hasPrefix(cleanupPrefix) ?? false)
            && ($0[kCGWindowIsOnscreen as String] as? Int) != 1
    })?[kCGWindowName as String] as? String else { return nil }
    guard let reason = title.split(separator: " ").first(where: { $0.hasPrefix("reason=") })?.split(separator: "=").last,
          let latency = title.split(separator: " ").first(where: { $0.hasPrefix("latency_ms=") })?.split(separator: "=").last?.dropLast(),
          let latencyMs = Int(latency) else { return nil }
    return (String(reason), latencyMs)
}

func wait(_ seconds: TimeInterval, _ condition: () -> Bool) -> Bool {
    let end = ProcessInfo.processInfo.systemUptime + seconds
    while ProcessInfo.processInfo.systemUptime < end { if condition() { return true }; Thread.sleep(forTimeInterval: 0.1) }
    return condition()
}

let display = CGMainDisplayID()
guard let original = CGDisplayCopyDisplayMode(display),
      let modes = CGDisplayCopyAllDisplayModes(display, nil) as? [CGDisplayMode],
      let candidate = modes.first(where: { $0.pixelWidth != original.pixelWidth || $0.pixelHeight != original.pixelHeight }) else { fail(6, "display-mode-unavailable") }
defer { _ = CGDisplaySetDisplayMode(display, original, nil); Thread.sleep(forTimeInterval: 1) }

guard window("Yonda · 圈选提问") != nil else { fail(7, "review-not-open") }
let started = ProcessInfo.processInfo.systemUptime
guard CGDisplaySetDisplayMode(display, candidate, nil) == .success else { fail(8, "display-change-failed") }
guard wait(5, { CGDisplayCopyDisplayMode(display)?.pixelWidth == candidate.pixelWidth }) else { fail(9, "display-mode-not-applied") }
guard wait(5, {
    guard window("Yonda · 圈选提问") == nil, let cleanup = cleanupInfo() else { return false }
    return ["app-switch", "close", "display-change"].contains(cleanup.reason) && cleanup.latencyMs <= 3000
}), let cleanup = cleanupInfo() else { fail(10, "display-change-cleanup-failed") }
let latency = Int((ProcessInfo.processInfo.systemUptime - started) * 1000)

let result: [String: Any] = [
    "passed": true,
    "reason": cleanup.reason,
    "cleanup_latency_ms": cleanup.latencyMs,
    "observed_latency_ms": latency,
    "original_mode_pixels": [original.pixelWidth, original.pixelHeight],
    "changed_mode_pixels": [candidate.pixelWidth, candidate.pixelHeight],
    "review_closed": true
]
let data = try! JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted, .sortedKeys])
print(String(data: data, encoding: .utf8)!)
