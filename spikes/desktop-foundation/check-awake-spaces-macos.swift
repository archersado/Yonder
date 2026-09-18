import AppKit
import ApplicationServices
func run() throws {
guard CommandLine.arguments.count == 4 else {throw NSError(domain:"参数：PID 输出目录 唤醒探针",code:1)}
let app = NSApplication.shared
app.setActivationPolicy(.prohibited)
let pid = pid_t(CommandLine.arguments[1])!
let output = URL(fileURLWithPath:CommandLine.arguments[2],isDirectory:true)
guard !FileManager.default.fileExists(atPath:output.path) else {throw NSError(domain:"证据目录已存在",code:1)}
guard let petApp = NSRunningApplication(processIdentifier:pid),petApp.bundleIdentifier == "com.yonder.e0-spike" else { throw NSError(domain:"非目标桌宠",code:1) }
try FileManager.default.createDirectory(at:output,withIntermediateDirectories:true)
let originalApp=NSWorkspace.shared.frontmostApplication
func attr(_ e:AXUIElement,_ key:String)->CFTypeRef? {var v:CFTypeRef?;return AXUIElementCopyAttributeValue(e,key as CFString,&v) == .success ? v : nil}
let originalWindow=originalApp.flatMap { attr(AXUIElementCreateApplication($0.processIdentifier),"AXFocusedWindow") }
func emit(_ row:[String:Any]){let d=try! JSONSerialization.data(withJSONObject:row,options:[.sortedKeys]);print(String(data:d,encoding:.utf8)!);fflush(stdout);let u=output.appendingPathComponent("normal-spaces.jsonl");if !FileManager.default.fileExists(atPath:u.path){FileManager.default.createFile(atPath:u.path,contents:nil)};let f=try! FileHandle(forWritingTo:u);f.seekToEndOfFile();f.write(d);f.write(Data([10]));try! f.close()}
defer {
 if let originalApp {_ = originalApp.activate(options:[])}
 if let originalWindow,CFGetTypeID(originalWindow)==AXUIElementGetTypeID(){emit(["restore_original_window_ax":AXUIElementPerformAction(originalWindow as! AXUIElement,"AXRaise" as CFString).rawValue])}
}
func wait(_ s:Double){RunLoop.current.run(until:Date().addingTimeInterval(s))}
func find(_ e:AXUIElement,_ id:String,_ depth:Int=0)->AXUIElement? {if attr(e,"AXIdentifier") as? String == id{return e};guard depth<8 else{return nil};for c in attr(e,"AXChildren") as? [AXUIElement] ?? []{if let r=find(c,id,depth+1){return r}};return nil}
func dock()->AXUIElement {AXUIElementCreateApplication(NSRunningApplication.runningApplications(withBundleIdentifier:"com.apple.dock").first!.processIdentifier)}
func mission()->Bool {if find(dock(),"mc.spaces.list") != nil{return true};let p=Process();p.executableURL=URL(fileURLWithPath:"/usr/bin/open");p.arguments=["/System/Applications/Mission Control.app"];try! p.run();p.waitUntilExit();for _ in 0..<15{wait(0.2);if find(dock(),"mc.spaces.list") != nil{return true}};return false}
func spaces()->[AXUIElement] {guard let list=find(dock(),"mc.spaces.list") else{return []};return attr(list,"AXChildren") as? [AXUIElement] ?? []}
func ordinary(_ e:AXUIElement)->Bool {let d=attr(e,"AXDescription") as? String ?? "";return d.contains("退出到“桌面") || (d.lowercased().contains("desktop") && !d.lowercased().contains("full"))}
func title(_ e:AXUIElement)->String {attr(e,"AXTitle") as? String ?? ""}
func select(_ name:String)->Bool {guard mission(),let e=spaces().first(where:{title($0)==name}) else{return false};let r=AXUIElementPerformAction(e,"AXPress" as CFString);wait(2);return r == .success && find(dock(),"mc.spaces.list") == nil}
func sample(_ phase:String) throws {let all=CGWindowListCopyWindowInfo([.optionAll,.excludeDesktopElements],kCGNullWindowID) as? [[String:Any]] ?? [];guard let w=all.first(where:{($0[kCGWindowOwnerPID as String] as? Int)==Int(pid) && ($0[kCGWindowName as String] as? String)=="Yonder E0"}),let b=w[kCGWindowBounds as String] as? NSDictionary,let r=CGRect(dictionaryRepresentation:b) else{emit(["error":"小龙窗口缺失"]);throw NSError(domain:"窗口不可读",code:7)};guard (200...201).contains(r.width),(200...201).contains(r.height),CGDisplayBounds(CGMainDisplayID()).contains(r) else {emit(["invalid_awake_bounds":[r.minX,r.minY,r.width,r.height]]);throw NSError(domain:"清醒尺寸或屏幕坐标不满足",code:8)};let p=Process();p.executableURL=URL(fileURLWithPath:"/usr/sbin/screencapture");p.arguments=["-x","-R","\(Int(r.minX)),\(Int(r.minY)),\(Int(r.width)),\(Int(r.height))",output.appendingPathComponent(phase+".png").path];try! p.run();p.waitUntilExit();emit(["phase":phase,"window_id":w[kCGWindowNumber as String] ?? 0,"bounds":[r.minX,r.minY,r.width,r.height],"capture_exit":p.terminationStatus]);guard p.terminationStatus == 0 else {throw NSError(domain:"截图失败",code:9)}}
guard mission() else{throw NSError(domain:"Mission Control 不可用",code:2)}
let before=spaces().filter(ordinary)
// 本轮只在一个普通桌面时创建临时桌面，避免猜测已有桌面的归属。
guard before.count == 1 else {
 if let originalWindow,CFGetTypeID(originalWindow)==AXUIElementGetTypeID(){_ = AXUIElementPerformAction(originalWindow as! AXUIElement,"AXRaise" as CFString)}
 throw NSError(domain:"非单普通桌面环境，未创建或删除桌面",code:3)
}
var originalName=title(before[0])
var temporary: String?
defer {
 if let temporary,mission(),let created=spaces().first(where:{ordinary($0) && title($0)==temporary}) {
  emit(["remove_temporary_ax":AXUIElementPerformAction(created,"AXRemoveDesktop" as CFString).rawValue]);wait(0.5)
  let remaining=spaces().filter(ordinary)
  emit(["ordinary_desktops_after_cleanup":remaining.count])
  if remaining.count == 1 {_ = select(title(remaining[0]))}
 }
}
guard let add=find(dock(),"mc.spaces.add") else{throw NSError(domain:"无添加入口",code:4)}
let added=AXUIElementPerformAction(add,"AXPress" as CFString);wait(1)
let normal=spaces().filter(ordinary)
// 系统可将原“桌面”重命名为“桌面1”；按唯一原桌面+尾部新增的原生列表辨识。
guard added == .success, normal.count == 2 else{throw NSError(domain:"创建结果不是两个普通桌面，需人工恢复",code:5)}
originalName=title(normal[0]);temporary=title(normal[1])
emit(["created_temporary_desktop":temporary!,"original_desktop":originalName])
guard select(originalName) else {throw NSError(domain:"唤醒前退出Mission Control失败",code:10)}
let wake=Process();wake.executableURL=URL(fileURLWithPath:CommandLine.arguments[3]);wake.arguments=[String(pid),"wake"];try wake.run();wake.waitUntilExit();guard wake.terminationStatus == 0 else {throw NSError(domain:"清醒前置失败",code:10)}
for (name,phase) in [(originalName,"ordinary-a"),(temporary!,"ordinary-b"),(originalName,"ordinary-a-returned")] {
 guard select(name) else {throw NSError(domain:"桌面切换失败",code:6)}
 wait(3);try sample(phase)
}
emit(["result":"清醒形态普通桌面往返采样完成，画面与尺寸须核对"])
}
do {try run()} catch {print("验证未通过：\(error)");exit(1)}
