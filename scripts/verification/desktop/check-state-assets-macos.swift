// 独立原生证据：仅找回并截取唯一Yonda小龙，不伪造任务或切换工作状态。
import AppKit
import ApplicationServices
let apps = NSRunningApplication.runningApplications(withBundleIdentifier: "com.yonder.desktop")
let blinkCheck = CommandLine.arguments.count == 3 && CommandLine.arguments[2] == "--blink"
let immediate = CommandLine.arguments.count == 3 && CommandLine.arguments[2] == "--immediate"
guard (CommandLine.arguments.count == 2 || blinkCheck || immediate), apps.count == 1, AXIsProcessTrusted() else { exit(2) }
let app = apps[0], ax = AXUIElementCreateApplication(apps[0].processIdentifier)
func attr(_ element: AXUIElement, _ name: String) -> CFTypeRef? {
 var value: CFTypeRef?
 return AXUIElementCopyAttributeValue(element, name as CFString, &value) == .success ? value : nil
}
func find(_ element: AXUIElement, _ depth: Int = 0) -> AXUIElement? {
 if ["AXDescription", "AXTitle", "AXValue"].contains(where: { attr(element, $0) as? String == "点击趴在边缘的 Yonda 唤醒它" }) { return element }
 guard depth < 16 else { return nil }
 for child in attr(element, "AXChildren") as? [AXUIElement] ?? [] { if let result = find(child, depth+1) { return result } }
 return nil
}
for window in attr(ax, "AXWindows") as? [AXUIElement] ?? [] {
 if let eyes = find(window) { guard AXUIElementPerformAction(eyes, "AXPress" as CFString) == .success else { exit(3) } }
}
Thread.sleep(forTimeInterval: immediate ? 0.1 : 1)
let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
print("target_window_bounds=\(windows.filter { ($0[kCGWindowOwnerPID as String] as? Int) == Int(app.processIdentifier) }.map { $0[kCGWindowBounds as String] ?? [:] })")
guard let pet = windows.first(where: { ($0[kCGWindowOwnerPID as String] as? Int) == Int(app.processIdentifier) && ($0[kCGWindowName as String] as? String) == "Yonda" }),
 let id = pet[kCGWindowNumber as String] as? Int,
 let bounds = pet[kCGWindowBounds as String] as? [String: CGFloat],
 let width = bounds["Width"], let height = bounds["Height"],
 (width == 200 && height >= 200 && height <= 201) || (blinkCheck && ((width == 56 && height == 112) || (width == 112 && height == 56))) else { print("pet_not_on_visible_desktop=true"); exit(4) }
let output = URL(fileURLWithPath: CommandLine.arguments[1], isDirectory: true)
guard !FileManager.default.fileExists(atPath: output.path) else { exit(2) }
try FileManager.default.createDirectory(at: output, withIntermediateDirectories: true)
let captures = blinkCheck ? 60 : 3
let started = ProcessInfo.processInfo.systemUptime
for i in 0..<captures {
 let capture = Process(); capture.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture")
 capture.arguments = ["-x", "-l", String(id), output.appendingPathComponent("native-\(i).png").path]
 try capture.run(); capture.waitUntilExit(); guard capture.terminationStatus == 0 else { exit(5) }
 Thread.sleep(forTimeInterval: blinkCheck ? 0.02 : 0.6)
}
let report: [String: Any] = ["pid": Int(app.processIdentifier), "single_host": true, "pet_on_visible_desktop": true, "captures": captures, "elapsed_seconds": ProcessInfo.processInfo.systemUptime-started, "bounds": pet[kCGWindowBounds as String]!, "no_task_mutation_by_probe": true]
try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys]).write(to: output.appendingPathComponent("native-result.json"))
print("native_pet_captured=true")
