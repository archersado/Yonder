// 原生验证交回后新计划已汇总到顶部浮窗；只读，不触发接管。
import AppKit
import ApplicationServices

guard CommandLine.arguments.count == 2,
      AXIsProcessTrusted(),
      let app = NSRunningApplication.runningApplications(withBundleIdentifier: "com.yonder.desktop").first
else { exit(2) }

let output = URL(fileURLWithPath: CommandLine.arguments[1], isDirectory: true)
try FileManager.default.createDirectory(at: output, withIntermediateDirectories: true)
let application = AXUIElementCreateApplication(app.processIdentifier)

func attribute(_ element: AXUIElement, _ name: String) -> CFTypeRef? {
    var value: CFTypeRef?
    return AXUIElementCopyAttributeValue(element, name as CFString, &value) == .success ? value : nil
}

func contains(_ element: AXUIElement, _ expected: String, depth: Int = 0) -> Bool {
    if attribute(element, "AXTitle") as? String == expected
        || attribute(element, "AXDescription") as? String == expected
        || attribute(element, "AXValue") as? String == expected { return true }
    guard depth < 16 else { return false }
    return (attribute(element, "AXChildren") as? [AXUIElement] ?? []).contains {
        contains($0, expected, depth: depth + 1)
    }
}

func controlWindow() -> AXUIElement? {
    (attribute(application, "AXWindows") as? [AXUIElement] ?? []).first {
        attribute($0, "AXTitle") as? String == "Yonda · 任务执行"
    }
}

let expected = [
    "Yonder 正在控制您的电脑",
    "慢脑已提交重新规划：2 个步骤",
    "重新规划：打开安全验证窗口",
    "重新规划：核验窗口并推进焦点",
]
let deadline = ProcessInfo.processInfo.systemUptime + 20
while controlWindow().map({ window in expected.allSatisfy { contains(window, $0) } }) != true
    && ProcessInfo.processInfo.systemUptime < deadline {
    Thread.sleep(forTimeInterval: 0.02)
}
guard let window = controlWindow() else { exit(3) }
let matches = Dictionary(uniqueKeysWithValues: expected.map { ($0, contains(window, $0)) })
let list = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
if let info = list.first(where: { ($0[kCGWindowOwnerPID as String] as? Int) == Int(app.processIdentifier) && ($0[kCGWindowName as String] as? String) == "Yonda · 任务执行" }),
   let id = info[kCGWindowNumber as String] as? Int {
    let capture = Process()
    capture.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture")
    capture.arguments = ["-x", "-l", String(id), output.appendingPathComponent("replan-control.png").path]
    try capture.run()
    capture.waitUntilExit()
}
let passed = matches.values.allSatisfy { $0 }
let report: [String: Any] = ["pid": Int(app.processIdentifier), "visible": true, "matches": matches, "passed": passed]
let data = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys])
try data.write(to: output.appendingPathComponent("window-result.json"))
print(String(data: data, encoding: .utf8)!)
if !passed { exit(4) }
