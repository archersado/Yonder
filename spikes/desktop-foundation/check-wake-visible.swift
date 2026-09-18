import AppKit
import ApplicationServices
import ImageIO
guard (3...4).contains(CommandLine.arguments.count), let pid = pid_t(CommandLine.arguments[1]) else { fatalError("参数：PID 证据目录 [--click-right-eye]") }
let clickEyes = CommandLine.arguments.count == 4
let activateForControl = clickEyes && CommandLine.arguments[3] == "--click-right-eye-active"
guard !clickEyes || CommandLine.arguments[3] == "--click-right-eye" || activateForControl else { fatalError("未知验证模式") }
let requestedOutput = URL(fileURLWithPath: CommandLine.arguments[2], isDirectory: true)
let output = FileManager.default.fileExists(atPath: requestedOutput.path)
    ? requestedOutput.appendingPathComponent("run-\(Int(Date().timeIntervalSince1970 * 1000))", isDirectory:true) : requestedOutput
print("证据目录：\(output.path)")
guard let app = NSRunningApplication(processIdentifier: pid), app.bundleIdentifier == "com.yonder.e0-spike" else { fatalError("非目标桌宠") }
try FileManager.default.createDirectory(at: output, withIntermediateDirectories: true)
// 只采目标窗口，不录制桌面；自然等待最多 210 秒，不修改产品休眠计时。
func emit(_ value: [String: Any]) {
    let data = try! JSONSerialization.data(withJSONObject: value, options: [.sortedKeys])
    print(String(data: data, encoding: .utf8)!)
    fflush(stdout)
    let log = output.appendingPathComponent("observations.jsonl")
    if !FileManager.default.fileExists(atPath: log.path) { FileManager.default.createFile(atPath: log.path, contents: nil) }
    if let handle = try? FileHandle(forWritingTo: log) {
        handle.seekToEndOfFile(); handle.write(data); handle.write(Data([10])); try? handle.close()
    }
}
func window() -> [String: Any]? {
    let all = CGWindowListCopyWindowInfo([.optionAll, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
    return all.first { ($0[kCGWindowOwnerPID as String] as? Int) == Int(pid) && ($0[kCGWindowName as String] as? String) == "Yonder E0" }
}
func snapshot(_ phase: String) -> Bool {
    guard let w = window(), let bounds = w[kCGWindowBounds as String] as? NSDictionary,
          let rect = CGRect(dictionaryRepresentation: bounds), let id = w[kCGWindowNumber as String] as? Int else { return false }
    let onscreen = w[kCGWindowIsOnscreen as String] as? Bool ?? false
    let file = output.appendingPathComponent(phase + ".png")
    let capture = Process(); capture.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture")
    capture.arguments = ["-x", "-l", String(id), file.path]
    try! capture.run(); capture.waitUntilExit()
    var pixels: [Int] = []
    if let source = CGImageSourceCreateWithURL(file as CFURL, nil),
       let image = CGImageSourceCreateImageAtIndex(source, 0, nil) { pixels = [image.width, image.height] }
    emit(["phase":phase,"pid":Int(pid),"window_id":id,"onscreen":onscreen,"bounds":[rect.minX,rect.minY,rect.width,rect.height],"pixels":pixels,"capture_exit":capture.terminationStatus])
    // onscreen 仅记证据：已有目视及屏幕区域证据证明 false 不足以判定不可见。
    let screen = Process(); screen.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture")
    screen.arguments = ["-x", "-R", "\(Int(rect.minX)),\(Int(rect.minY)),\(Int(rect.width)),\(Int(rect.height))", output.appendingPathComponent(phase + "-screen.png").path]
    try! screen.run(); screen.waitUntilExit()
    emit(["phase":phase,"screen_capture_exit":screen.terminationStatus])
    return capture.terminationStatus == 0 && screen.terminationStatus == 0 && pixels.count == 2 && pixels[0] >= Int(rect.width) && pixels[1] >= Int(rect.height)
        && pixels[0] * Int(rect.height) == pixels[1] * Int(rect.width)
}
// 不额外激活应用，避免掩盖托盘本身的唤醒行为。
guard snapshot("before") else { emit(["result":"前置窗口捕获或截图尺寸不满足，停止"]); exit(2) }
let deadline = Date().addingTimeInterval(210)
var docked = false
var ticks = 0
while Date() < deadline {
    if let w = window(), let b = w[kCGWindowBounds as String] as? NSDictionary, let r = CGRect(dictionaryRepresentation: b), r.width < 200 || r.height < 200 { docked = true; break }
    Thread.sleep(forTimeInterval: 1); ticks += 1
    if ticks % 20 == 0 { emit(["waiting_seconds":ticks]) }
}
guard docked, snapshot("docked") else { emit(["result":"休眠未出现或捕获不满足，停止"]); exit(3) }
let ax = AXUIElementCreateApplication(pid)
func attr(_ e: AXUIElement, _ key: String) -> CFTypeRef? { var v: CFTypeRef?; return AXUIElementCopyAttributeValue(e, key as CFString, &v) == .success ? v : nil }
func find(_ e: AXUIElement, _ depth: Int) -> AXUIElement? {
    if attr(e,"AXTitle") as? String == "显示小龙" { return e }
    guard depth < 5 else { return nil }
    for c in attr(e,"AXChildren") as? [AXUIElement] ?? [] { if let item = find(c, depth + 1) { return item } }
    return nil
}
let started = Date()
let clockStarted = ProcessInfo.processInfo.systemUptime
if clickEyes {
    emit(["active_before_input":app.isActive,"activate_for_control":activateForControl])
    if activateForControl {
        let activated = app.activate(options:[])
        Thread.sleep(forTimeInterval:0.3)
        emit(["activation_requested":activated,"active_after_activation":app.isActive])
        guard app.isActive else { exit(4) }
    }
    guard CGPreflightPostEventAccess(), let w = window(), w[kCGWindowIsOnscreen as String] as? Bool == true,
          let b = w[kCGWindowBounds as String] as? NSDictionary, let r = CGRect(dictionaryRepresentation:b),
          r.width == 56 && r.height == 112,
          CGDisplayBounds(CGMainDisplayID()).contains(r),
          let down = CGEvent(mouseEventSource:nil, mouseType:.leftMouseDown, mouseCursorPosition:CGPoint(x:r.minX + 49,y:r.minY + 38), mouseButton:.left),
          let up = CGEvent(mouseEventSource:nil, mouseType:.leftMouseUp, mouseCursorPosition:CGPoint(x:r.minX + 49,y:r.minY + 38), mouseButton:.left)
    else { emit(["result":"目标眼睛入口不可确认，不发送点击"]); exit(4) }
    // 仅用于已人工确认的右侧形态，按素材实拍点击眼睛；矩形中心可能透明。
    let point = down.location
    CGEvent(mouseEventSource:nil, mouseType:.mouseMoved, mouseCursorPosition:point, mouseButton:.left)?.post(tap:.cghidEventTap)
    Thread.sleep(forTimeInterval:0.1)
    down.setIntegerValueField(.mouseEventClickState, value:1)
    up.setIntegerValueField(.mouseEventClickState, value:1)
    down.post(tap:.cghidEventTap)
    Thread.sleep(forTimeInterval:0.05)
    up.post(tap:.cghidEventTap)
    emit(["wake_input":"native_pointer_right_eye","x":r.minX + 49,"y":r.minY + 38,"post_access":true])
} else {
guard let menu = attr(ax,"AXExtrasMenuBar"), CFGetTypeID(menu) == AXUIElementGetTypeID(), let item = find(menu as! AXUIElement,0) else { emit(["result":"原生菜单项缺失"]); exit(4) }
let pressed = AXUIElementPerformAction(item,"AXPress" as CFString)
emit(["show_ax_result":pressed.rawValue])
guard pressed == .success else { exit(5) }
}
// 只测原生窗口几何恢复；不把尺寸恢复冒充图片已绘制或核心状态事件延迟。
var restoredAfter: Double?
while ProcessInfo.processInfo.systemUptime - clockStarted < 3 {
    if let w = window(), let b = w[kCGWindowBounds as String] as? NSDictionary,
       let r = CGRect(dictionaryRepresentation:b), r.width == 200, r.height == 200 {
        restoredAfter = (ProcessInfo.processInfo.systemUptime - clockStarted) * 1000
        break
    }
    Thread.sleep(forTimeInterval:0.01)
}
emit(["geometry_restored":restoredAfter != nil,"geometry_latency_ms":restoredAfter ?? -1,"poll_interval_ms":10])
Thread.sleep(forTimeInterval: 1)
let first = snapshot("awake-1s")
Thread.sleep(forTimeInterval: 2)
let second = snapshot("awake-3s")
let w = window()!
let b = w[kCGWindowBounds as String] as! NSDictionary
let r = CGRect(dictionaryRepresentation:b)!
let passed = restoredAfter != nil && first && second && r.width == 200 && r.height == 200
emit(["result":passed ? "休眠与唤醒尺寸检查通过；屏幕区域内容须人工核对" : "唤醒尺寸检查失败","observation_seconds":Date().timeIntervalSince(started)])
exit(passed ? 0 : 6)
