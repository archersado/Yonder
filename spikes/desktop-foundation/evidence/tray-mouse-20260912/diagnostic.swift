import AppKit
import ApplicationServices
import ImageIO

guard CommandLine.arguments.count == 3, let pid = pid_t(CommandLine.arguments[1]),
      let app = NSRunningApplication(processIdentifier: pid), app.bundleIdentifier == "com.yonder.e0-spike" else { fatalError("参数必须为 Yonda PID 和证据目录") }
let directory = URL(fileURLWithPath: CommandLine.arguments[2], isDirectory: true)
try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
func log(_ row: [String: Any]) {
    let data = try! JSONSerialization.data(withJSONObject: row, options: [.sortedKeys])
    print(String(data:data,encoding:.utf8)!); fflush(stdout)
    let url = directory.appendingPathComponent("mouse.jsonl")
    if !FileManager.default.fileExists(atPath:url.path) { FileManager.default.createFile(atPath:url.path,contents:nil) }
    let file = try! FileHandle(forWritingTo:url); file.seekToEndOfFile(); file.write(data); file.write(Data([10])); try! file.close()
}
func attr(_ e: AXUIElement, _ key: String) -> CFTypeRef? {
    var value: CFTypeRef?; return AXUIElementCopyAttributeValue(e,key as CFString,&value) == .success ? value : nil
}
func rect(_ element: AXUIElement) -> CGRect? {
    guard let p = attr(element,"AXPosition"), let s = attr(element,"AXSize"), CFGetTypeID(p) == AXValueGetTypeID(), CFGetTypeID(s) == AXValueGetTypeID() else { return nil }
    var point = CGPoint.zero; var size = CGSize.zero
    guard AXValueGetValue(p as! AXValue,.cgPoint,&point), AXValueGetValue(s as! AXValue,.cgSize,&size), size.width > 0, size.height > 0 else { return nil }
    return CGRect(origin:point,size:size)
}
func click(_ element: AXUIElement, _ label: String) {
    guard let bounds = rect(element) else { fatalError("无法定位目标菜单") }
    let point = CGPoint(x:bounds.midX,y:bounds.midY)
    var hit: AXUIElement?
    let result = AXUIElementCopyElementAtPosition(AXUIElementCreateSystemWide(),Float(point.x),Float(point.y),&hit)
    log(["target":label,"bounds":[bounds.minX,bounds.minY,bounds.width,bounds.height],"hit_result":result.rawValue])
    if result != .success { _ = AXUIElementCopyElementAtPosition(AXUIElementCreateApplication(pid),Float(point.x),Float(point.y),&hit) }
    guard let hit else { log(["result":"无法确认目标命中，未点击"]); exit(3) }
    var owner: pid_t = 0
    guard AXUIElementGetPid(hit,&owner) == .success, owner == pid else { fatalError("命中其他进程，停止点击") }
    CGWarpMouseCursorPosition(point)
    for type in [CGEventType.leftMouseDown,.leftMouseUp] {
        CGEvent(mouseEventSource:nil,mouseType:type,mouseCursorPosition:point,mouseButton:.left)?.post(tap:.cghidEventTap)
        Thread.sleep(forTimeInterval:0.08)
    }
    log(["clicked":label,"point":[point.x,point.y],"hit_pid":Int(owner)])
}
func find(_ e: AXUIElement, _ depth: Int = 0) -> AXUIElement? {
    if attr(e,"AXTitle") as? String == "显示小龙" { return e }
    guard depth < 5 else { return nil }
    for child in attr(e,"AXChildren") as? [AXUIElement] ?? [] { if let result = find(child,depth+1) { return result } }
    return nil
}
let application = AXUIElementCreateApplication(pid)
guard let raw = attr(application,"AXExtrasMenuBar"), CFGetTypeID(raw) == AXUIElementGetTypeID() else { fatalError("无托盘菜单栏") }
let bar = raw as! AXUIElement
guard let icon = (attr(bar,"AXChildren") as? [AXUIElement])?.first else { fatalError("无托盘图标") }
let previous = CGEvent(source:nil)?.location
defer { if let previous { CGWarpMouseCursorPosition(previous) } }
click(icon,"Yonda 托盘图标")
Thread.sleep(forTimeInterval:0.5)
guard let show = find(bar) else { fatalError("菜单未出现") }
click(show,"显示小龙")
Thread.sleep(forTimeInterval:1)
let windows = CGWindowListCopyWindowInfo([.optionAll,.excludeDesktopElements],kCGNullWindowID) as? [[String:Any]] ?? []
guard let pet = windows.first(where:{ ($0[kCGWindowOwnerPID as String] as? Int) == Int(pid) && ($0[kCGWindowName as String] as? String) == "Yonder E0" }),
      let bounds = pet[kCGWindowBounds as String] as? NSDictionary, let frame = CGRect(dictionaryRepresentation:bounds), let id = pet[kCGWindowNumber as String] as? Int else { fatalError("找不到桌宠窗口") }
let output = directory.appendingPathComponent("after-mouse-show.png")
let process = Process(); process.executableURL = URL(fileURLWithPath:"/usr/sbin/screencapture"); process.arguments = ["-x","-l",String(id),output.path]
try process.run(); process.waitUntilExit()
guard process.terminationStatus == 0, let source = CGImageSourceCreateWithURL(output as CFURL,nil), let image = CGImageSourceCreateImageAtIndex(source,0,nil) else { fatalError("截图失败") }
log(["window_bounds":[frame.minX,frame.minY,frame.width,frame.height],"image_pixels":[image.width,image.height],"onscreen":pet[kCGWindowIsOnscreen as String] as? Bool ?? false])
guard frame.width == 200, frame.height == 200, image.width >= 200, image.height >= 200, image.width == image.height else { exit(2) }
