import AppKit
import ApplicationServices
import ImageIO
guard CommandLine.arguments.count == 3, let pid = pid_t(CommandLine.arguments[1]) else { fatalError("参数：PID 证据目录") }
let output = URL(fileURLWithPath: CommandLine.arguments[2], isDirectory: true)
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
    return onscreen && capture.terminationStatus == 0 && pixels == [Int(rect.width), Int(rect.height)]
}
_ = app.activate(options: [])
Thread.sleep(forTimeInterval: 0.8)
_ = snapshot("before") // 诊断续作：记录离屏状态，但继续唤醒以恢复窗口。
let deadline = Date().addingTimeInterval(210)
var docked = false
var ticks = 0
while Date() < deadline {
    if let w = window(), let b = w[kCGWindowBounds as String] as? NSDictionary, let r = CGRect(dictionaryRepresentation: b), r.width < 200 || r.height < 200 { docked = true; break }
    Thread.sleep(forTimeInterval: 1); ticks += 1
    if ticks % 20 == 0 { emit(["waiting_seconds":ticks]) }
}
guard docked else { emit(["result":"未观察到休眠"]); exit(3) }
_ = snapshot("docked")
let ax = AXUIElementCreateApplication(pid)
func attr(_ e: AXUIElement, _ key: String) -> CFTypeRef? { var v: CFTypeRef?; return AXUIElementCopyAttributeValue(e, key as CFString, &v) == .success ? v : nil }
func find(_ e: AXUIElement, _ depth: Int) -> AXUIElement? {
    if attr(e,"AXTitle") as? String == "显示小龙" { return e }
    guard depth < 5 else { return nil }
    for c in attr(e,"AXChildren") as? [AXUIElement] ?? [] { if let item = find(c, depth + 1) { return item } }
    return nil
}
guard let menu = attr(ax,"AXExtrasMenuBar"), CFGetTypeID(menu) == AXUIElementGetTypeID(), let item = find(menu as! AXUIElement,0) else { emit(["result":"原生菜单项缺失"]); exit(4) }
let started = Date()
let pressed = AXUIElementPerformAction(item,"AXPress" as CFString)
emit(["show_ax_result":pressed.rawValue])
guard pressed == .success else { exit(5) }
Thread.sleep(forTimeInterval: 1)
let first = snapshot("awake-1s")
Thread.sleep(forTimeInterval: 2)
let second = snapshot("awake-3s")
let w = window()!
let b = w[kCGWindowBounds as String] as! NSDictionary
let r = CGRect(dictionaryRepresentation:b)!
let passed = first && second && r.width == 200 && r.height == 200
emit(["result":passed ? "可见桌面休眠与唤醒尺寸检查通过" : "唤醒尺寸检查失败","observation_seconds":Date().timeIntervalSince(started)])
exit(passed ? 0 : 6)
