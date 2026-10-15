// 正式 Tauri 宿主的隔离标识原生证据：仅检查已存在的测试任务，不操作用户库。
import AppKit
import ApplicationServices

guard (CommandLine.arguments.count == 2 || (CommandLine.arguments.count == 3 && ["--control", "--focus", "--creation", "--attempt-start"].contains(CommandLine.arguments[2]))), AXIsProcessTrusted() else { exit(2) }
let mode = CommandLine.arguments.count == 3 ? CommandLine.arguments[2] : "--observation"
let controlMode = mode == "--control"
let focusMode = mode == "--focus"
let creationMode = mode == "--creation"
let attemptStartMode = mode == "--attempt-start"
let bundleIdentifier = attemptStartMode ? "com.yonder.attempt-start.fixture" : (creationMode ? "com.yonder.creation.fixture" : (focusMode ? "com.yonder.focus.fixture" : (controlMode ? "com.yonder.control.fixture" : "com.yonder.observation.fixture")))
let taskName = attemptStartMode ? "原生尝试开始验证任务" : (creationMode ? "原生创建来源验证任务" : (focusMode ? "原生定位历史验证任务" : (controlMode ? "原生控制历史验证任务" : "原生 Observe 验证任务")))
let firstText = attemptStartMode ? "执行尝试已准备（步骤 step-one · 尝试 attempt-one）" : (creationMode ? "任务创建：云端 Agent · Agent fixture-agent" : (focusMode ? "接管：正在定位任务工作" : (controlMode ? "接管：停止请求已登记（尝试 attempt-one）" : "Observe（步骤 step-one）：已匹配")))
let secondText = attemptStartMode ? "动作已观察：成功" : (creationMode ? firstText : (focusMode ? "接管：任务工作定位成功" : (controlMode ? "接管：步骤边界停止已确认（尝试 attempt-one）" : "Observe（步骤 step-two）：未知")))
guard let app = NSRunningApplication.runningApplications(withBundleIdentifier: bundleIdentifier).first else { exit(2) }
let root = AXUIElementCreateApplication(app.processIdentifier)
func attr(_ element: AXUIElement, _ name: String) -> CFTypeRef? {
    var value: CFTypeRef?
    return AXUIElementCopyAttributeValue(element, name as CFString, &value) == .success ? value : nil
}
func find(_ element: AXUIElement, _ text: String, _ depth: Int = 0) -> AXUIElement? {
    guard depth < 20 else { return nil }
    if ["AXTitle", "AXDescription", "AXValue"].contains(where: { (attr(element, $0) as? String)?.contains(text) == true }) { return element }
    for child in attr(element, "AXChildren") as? [AXUIElement] ?? [] {
        if let found = find(child, text, depth + 1) { return found }
    }
    return nil
}
func button(_ element: AXUIElement, _ text: String, _ depth: Int = 0) -> AXUIElement? {
    guard depth < 20 else { return nil }
    if ["AXButton", "AXCheckBox"].contains(attr(element, "AXRole") as? String ?? ""), find(element, text) != nil { return element }
    for child in attr(element, "AXChildren") as? [AXUIElement] ?? [] {
        if let found = button(child, text, depth + 1) { return found }
    }
    return nil
}
func taskWindow() -> AXUIElement? {
    (attr(root, "AXWindows") as? [AXUIElement] ?? []).first { (attr($0, "AXTitle") as? String) == "Yonda · Task Space" }
}
func windowInfo() -> [String: Any]? {
    let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
    return windows.first { ($0[kCGWindowOwnerPID as String] as? Int) == Int(app.processIdentifier) && ($0[kCGWindowName as String] as? String) == "Yonda · Task Space" }
}
guard let pet = (attr(root, "AXWindows") as? [AXUIElement] ?? []).first(where: { (attr($0, "AXTitle") as? String) == "Yonda" }) else { print("pet_ax_missing=true"); exit(3) }
if let eyes = find(pet, "点击趴在边缘的 Yonda 唤醒它") {
    guard AXUIElementPerformAction(eyes, "AXPress" as CFString) == .success else { print("pet_wake_failed=true"); exit(3) }
    Thread.sleep(forTimeInterval: 0.8)
}
if windowInfo() == nil {
    app.activate(options: [.activateAllWindows])
    Thread.sleep(forTimeInterval: 0.5)
    guard let trigger = find(pet, "轻点Yonda小龙"),
          AXUIElementSetAttributeValue(trigger, kAXFocusedAttribute as CFString, kCFBooleanTrue) == .success else { print("pet_keyboard_focus_missing=true"); exit(3) }
    CGEvent(keyboardEventSource: nil, virtualKey: 36, keyDown: true)?.post(tap: .cghidEventTap)
    CGEvent(keyboardEventSource: nil, virtualKey: 36, keyDown: false)?.post(tap: .cghidEventTap)
    let openingDeadline = Date().addingTimeInterval(3)
    while windowInfo() == nil && Date() < openingDeadline { Thread.sleep(forTimeInterval: 0.02) }
}
guard let panelInfo = windowInfo(), let windowID = panelInfo[kCGWindowNumber as String] as? Int else {
    let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
    print("panel_keyboard_missing=true yonda_windows=\(windows.filter { ($0[kCGWindowName as String] as? String)?.contains("Yonda") == true }.map { "\($0[kCGWindowOwnerPID as String] ?? ""):\($0[kCGWindowName as String] ?? ""):\($0[kCGWindowBounds as String] ?? "")" })")
    exit(3)
}
Thread.sleep(forTimeInterval: 0.2)
let deadline = Date().addingTimeInterval(12)
while taskWindow().flatMap({ button($0, "全部") }) == nil && Date() < deadline { Thread.sleep(forTimeInterval: 0.1) }
if taskWindow().flatMap({ button($0, "全部") }) == nil {
    if let panel = taskWindow() {
        func inspect(_ element: AXUIElement, _ depth: Int = 0) {
            guard depth < 8 else { return }
            print("\(String(repeating: " ", count: depth * 2))\(attr(element, "AXRole") as? String ?? "") \(attr(element, "AXTitle") as? String ?? "") \(attr(element, "AXDescription") as? String ?? "")")
            for child in attr(element, "AXChildren") as? [AXUIElement] ?? [] { inspect(child, depth + 1) }
        }
        inspect(panel)
    }
    if let info = windowInfo(), let id = info[kCGWindowNumber as String] as? Int {
        let capture = Process()
        capture.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture")
        capture.arguments = ["-x", "-l", String(id), "/private/tmp/yonda-observation-fixture-diagnostic.png"]
        try? capture.run(); capture.waitUntilExit()
        print("diagnostic_capture_status=\(capture.terminationStatus)")
    }
    print("task_window_ax=\(taskWindow() != nil) task_window_visible=\(windowInfo() != nil)")
}
guard let panel = taskWindow(), let all = button(panel, "全部"),
      AXUIElementPerformAction(all, "AXPress" as CFString) == .success else { exit(4) }
while button(panel, taskName) == nil && Date() < deadline { Thread.sleep(forTimeInterval: 0.1) }
guard let task = button(panel, taskName),
      AXUIElementPerformAction(task, "AXPress" as CFString) == .success else { exit(5) }
while (find(panel, firstText) == nil || find(panel, secondText) == nil) && Date() < deadline {
    Thread.sleep(forTimeInterval: 0.1)
}
guard let first = find(panel, firstText),
      let second = find(panel, secondText),
      windowInfo() != nil else { exit(6) }
let output = URL(fileURLWithPath: CommandLine.arguments[1], isDirectory: true)
guard !FileManager.default.fileExists(atPath: output.path) else { exit(7) }
try FileManager.default.createDirectory(at: output, withIntermediateDirectories: true)
func capture(_ element: AXUIElement, _ filename: String) throws {
    guard AXUIElementPerformAction(element, "AXScrollToVisible" as CFString) == .success else { throw NSError(domain: "observe-fixture", code: 8) }
    Thread.sleep(forTimeInterval: 0.4)
    let process = Process()
    process.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture")
    process.arguments = ["-x", "-l", String(windowID), output.appendingPathComponent(filename).path]
    try process.run(); process.waitUntilExit()
    guard process.terminationStatus == 0 else { throw NSError(domain: "observe-fixture", code: 8) }
}
try capture(first, attemptStartMode ? "native-host-attempt-start.png" : (creationMode ? "native-host-creation.png" : (focusMode ? "native-host-focus-locating.png" : (controlMode ? "native-host-control-pending.png" : "native-host-observation-step-one.png"))))
if !creationMode {
    try capture(second, attemptStartMode ? "native-host-attempt-result.png" : (focusMode ? "native-host-focus-succeeded.png" : (controlMode ? "native-host-control-stopped.png" : "native-host-observation-step-two.png")))
}
let report: [String: Any] = [
    "pid": Int(app.processIdentifier),
    "bundle_identifier": bundleIdentifier,
    "formal_tauri_host": true,
    "first_fact_visible": firstText,
    "second_fact_visible": secondText,
    "real_user_task_database_used": false,
    "passed": true
]
let data = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys])
try data.write(to: output.appendingPathComponent("result.json"))
print(String(data: data, encoding: .utf8)!)
