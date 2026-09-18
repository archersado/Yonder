import AppKit
import ApplicationServices
let app = NSApplication.shared
app.setActivationPolicy(.prohibited)
let pid = pid_t(CommandLine.arguments[1])!
let output = URL(fileURLWithPath:CommandLine.arguments[2],isDirectory:true)
guard let petApp = NSRunningApplication(processIdentifier:pid),petApp.bundleIdentifier == "com.yonder.e0-spike" else { exit(1) }
try FileManager.default.createDirectory(at:output,withIntermediateDirectories:true)
let originalApp=NSWorkspace.shared.frontmostApplication
func attr(_ e:AXUIElement,_ key:String)->CFTypeRef? {var v:CFTypeRef?;return AXUIElementCopyAttributeValue(e,key as CFString,&v) == .success ? v : nil}
let originalWindow=originalApp.flatMap { attr(AXUIElementCreateApplication($0.processIdentifier),"AXFocusedWindow") }
func emit(_ row:[String:Any]){let d=try! JSONSerialization.data(withJSONObject:row,options:[.sortedKeys]);print(String(data:d,encoding:.utf8)!);fflush(stdout);let u=output.appendingPathComponent("normal-spaces.jsonl");if !FileManager.default.fileExists(atPath:u.path){FileManager.default.createFile(atPath:u.path,contents:nil)};let f=try! FileHandle(forWritingTo:u);f.seekToEndOfFile();f.write(d);f.write(Data([10]));try! f.close()}
func wait(_ s:Double){RunLoop.current.run(until:Date().addingTimeInterval(s))}
func find(_ e:AXUIElement,_ id:String,_ depth:Int=0)->AXUIElement? {if attr(e,"AXIdentifier") as? String == id{return e};guard depth<8 else{return nil};for c in attr(e,"AXChildren") as? [AXUIElement] ?? []{if let r=find(c,id,depth+1){return r}};return nil}
func dock()->AXUIElement {AXUIElementCreateApplication(NSRunningApplication.runningApplications(withBundleIdentifier:"com.apple.dock").first!.processIdentifier)}
func mission()->Bool {if find(dock(),"mc.spaces.list") != nil{return true};let p=Process();p.executableURL=URL(fileURLWithPath:"/usr/bin/open");p.arguments=["/System/Applications/Mission Control.app"];try! p.run();p.waitUntilExit();for _ in 0..<15{wait(0.2);if find(dock(),"mc.spaces.list") != nil{return true}};return false}
func spaces()->[AXUIElement] {guard let list=find(dock(),"mc.spaces.list") else{return []};return attr(list,"AXChildren") as? [AXUIElement] ?? []}
func ordinary(_ e:AXUIElement)->Bool {let d=attr(e,"AXDescription") as? String ?? "";return d.contains("退出到“桌面") || (d.lowercased().contains("desktop") && !d.lowercased().contains("full"))}
func title(_ e:AXUIElement)->String {attr(e,"AXTitle") as? String ?? ""}
func select(_ name:String)->Bool {guard mission(),let e=spaces().first(where:{title($0)==name}) else{return false};let r=AXUIElementPerformAction(e,"AXPress" as CFString);wait(2);return r == .success && find(dock(),"mc.spaces.list") == nil}
func sample(_ phase:String){let all=CGWindowListCopyWindowInfo([.optionAll,.excludeDesktopElements],kCGNullWindowID) as? [[String:Any]] ?? [];guard let w=all.first(where:{($0[kCGWindowOwnerPID as String] as? Int)==Int(pid) && ($0[kCGWindowName as String] as? String)=="Yonder E0"}),let b=w[kCGWindowBounds as String] as? NSDictionary,let r=CGRect(dictionaryRepresentation:b) else{emit(["error":"小龙窗口缺失"]);return};let p=Process();p.executableURL=URL(fileURLWithPath:"/usr/sbin/screencapture");p.arguments=["-x","-R","\(Int(r.minX)),\(Int(r.minY)),\(Int(r.width)),\(Int(r.height))",output.appendingPathComponent(phase+".png").path];try! p.run();p.waitUntilExit();emit(["phase":phase,"window_id":w[kCGWindowNumber as String] ?? 0,"bounds":[r.minX,r.minY,r.width,r.height],"capture_exit":p.terminationStatus])}
guard mission() else{emit(["error":"Mission Control 不可用"]);exit(2)}
let before=spaces().filter(ordinary)
guard let original=before.first else{emit(["error":"没有可识别普通桌面"]);exit(3)}
let originalName=title(original)
// 本轮续作：上一轮已创建且原生列表确认唯一新增“桌面2”。
var names=Set(spaces().map(title))
var temporary:String?
if before.count == 2 && title(before.last!) == "桌面2" { temporary="桌面2"; names.remove("桌面2") }
defer {
 if let temporary,mission(),let created=spaces().first(where:{ordinary($0) && title($0)==temporary && !names.contains(title($0))}) {emit(["remove_temporary_ax":AXUIElementPerformAction(created,"AXRemoveDesktop" as CFString).rawValue]);wait(0.5);emit(["ordinary_desktops_after_cleanup":spaces().filter(ordinary).count])}
 if mission(){_ = select(originalName)}
 if let originalApp {_ = originalApp.activate(options:[])}
 if let originalWindow,CFGetTypeID(originalWindow)==AXUIElementGetTypeID(){emit(["restore_original_window_ax":AXUIElementPerformAction(originalWindow as! AXUIElement,"AXRaise" as CFString).rawValue])}
}
if before.count < 2 {
 guard let add=find(dock(),"mc.spaces.add") else{emit(["error":"无添加普通桌面入口"]);exit(4)}
 let r=AXUIElementPerformAction(add,"AXPress" as CFString);wait(1)
 let new=spaces().filter{ordinary($0) && !names.contains(title($0))}
 guard r == .success,new.count==1 else{emit(["error":"无法确认唯一新增普通桌面"]);exit(5)}
 temporary=title(new[0]);emit(["created_temporary_desktop":true])
}
let normal=spaces().filter(ordinary)
guard let other=normal.first(where:{title($0) != originalName}) else{emit(["error":"第二普通桌面缺失"]);exit(6)}
let otherName=title(other)
emit(["ordinary_desktops":normal.count])
if select(originalName) {sample("ordinary-a")} else {emit(["error":"进入普通桌面 A 失败"])}
if select(otherName) {sample("ordinary-b")} else {emit(["error":"进入普通桌面 B 失败"])}
if select(originalName) {sample("ordinary-a-returned")} else {emit(["error":"返回普通桌面 A 失败"])}
emit(["result":"普通桌面往返采样完成；截图需人工核对"])
