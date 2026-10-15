// 仅操作com.yonder.desktop；真实空任务窗口、关闭隐藏与托盘恢复的原生验证。
import AppKit
import ApplicationServices
// 清醒入口仅鼠标移动，不点击；关闭和托盘恢复使用辅助功能。
guard CommandLine.arguments.count == 2, AXIsProcessTrusted(),
      let app = NSRunningApplication.runningApplications(withBundleIdentifier:"com.yonder.desktop").first else { exit(2) }
// 结束可能遗留的原生拖动，后续仅发送移动。
let cursor=CGEvent(source:nil)?.location ?? .zero
let release=CGEvent(mouseEventSource:nil,mouseType:.leftMouseUp,mouseCursorPosition:cursor,mouseButton:.left)
release?.flags=[];release?.post(tap:.cghidEventTap)
Thread.sleep(forTimeInterval:0.3)
let ax = AXUIElementCreateApplication(app.processIdentifier)
func attr(_ element:AXUIElement,_ name:String)->CFTypeRef? {
 var value:CFTypeRef?; return AXUIElementCopyAttributeValue(element,name as CFString,&value) == .success ? value:nil
}
func find(_ element:AXUIElement,_ title:String,_ depth:Int=0)->AXUIElement? {
 if attr(element,"AXTitle") as? String == title || attr(element,"AXDescription") as? String == title {return element}
 guard depth<16 else{return nil}
 for child in attr(element,"AXChildren") as? [AXUIElement] ?? [] {if let found=find(child,title,depth+1){return found}}
 return nil
}
if CommandLine.arguments[1] == "--quit" {
 guard let extras=attr(ax,"AXExtrasMenuBar"), CFGetTypeID(extras)==AXUIElementGetTypeID(),
       let quit=find(extras as! AXUIElement,"退出任务面板"), AXUIElementPerformAction(quit,"AXPress" as CFString) == .success else{exit(3)}
 print("task_space_quit=true");exit(0)
}
let output=URL(fileURLWithPath:CommandLine.arguments[1],isDirectory:true)
guard !FileManager.default.fileExists(atPath:output.path) else{exit(2)}
try FileManager.default.createDirectory(at:output,withIntermediateDirectories:true)
func window()->AXUIElement? {
 (attr(ax,"AXWindows") as? [AXUIElement] ?? []).first{attr($0,"AXTitle") as? String == "Yonda · Task Space"}
}
func hasEmpty(_ element:AXUIElement,_ depth:Int=0)->Bool {
 guard depth<16 else{return false}
 if ["AXValue","AXDescription"].contains(where:{(attr(element,$0) as? String)?.contains("暂无任务") == true}) {return true}
 return (attr(element,"AXChildren") as? [AXUIElement] ?? []).contains{hasEmpty($0,depth+1)}
}
func visibleWindow()->[String:Any]? {
 let list=CGWindowListCopyWindowInfo([.optionOnScreenOnly,.excludeDesktopElements],kCGNullWindowID) as? [[String:Any]] ?? []
 return list.first{($0[kCGWindowOwnerPID as String] as? Int)==Int(app.processIdentifier) && ($0[kCGWindowName as String] as? String)=="Yonda · Task Space"}
}
let end=ProcessInfo.processInfo.systemUptime+15
guard let petWindow=(attr(ax,"AXWindows") as? [AXUIElement] ?? []).first(where:{attr($0,"AXTitle") as? String == "Yonda"}) else{print("pet_window_missing=true");exit(3)}
while find(petWindow,"轻点 Yonda 小龙，按住移动可拖动")==nil && find(petWindow,"点击趴在边缘的 Yonda 唤醒它")==nil && ProcessInfo.processInfo.systemUptime<end {
 Thread.sleep(forTimeInterval:0.2)
}
if let eyes=find(petWindow,"点击趴在边缘的 Yonda 唤醒它") {
 guard AXUIElementPerformAction(eyes,"AXPress" as CFString) == .success else{exit(3)}
 Thread.sleep(forTimeInterval:0.8)
}
guard let pet=find(petWindow,"轻点 Yonda 小龙，按住移动可拖动"),
      let position=attr(pet,"AXPosition"),let size=attr(pet,"AXSize") else{exit(3)}
var point=CGPoint.zero;var dimensions=CGSize.zero
AXValueGetValue(position as! AXValue,.cgPoint,&point);AXValueGetValue(size as! AXValue,.cgSize,&dimensions)
if let existing=window(),let close=find(existing,"关闭任务总览") { _=AXUIElementPerformAction(close,"AXPress" as CFString) }
let other=NSRunningApplication.runningApplications(withBundleIdentifier:"com.microsoft.VSCode").first
print("other_app_present=\(other != nil) activate_other=\(other?.activate(options:[.activateIgnoringOtherApps]) ?? false)")
Thread.sleep(forTimeInterval:1.0)
let screenWindows=CGWindowListCopyWindowInfo([.optionOnScreenOnly,.excludeDesktopElements],kCGNullWindowID) as? [[String:Any]] ?? []
print("pet_on_screen=\(screenWindows.contains{($0[kCGWindowOwnerPID as String] as? Int)==Int(app.processIdentifier) && ($0[kCGWindowName as String] as? String)=="Yonda"})")
print("event_access=\(CGPreflightPostEventAccess()) display=\(CGDisplayBounds(CGMainDisplayID())) pet_bounds=\(screenWindows.filter{($0[kCGWindowOwnerPID as String] as? Int)==Int(app.processIdentifier)})")
let inactive = !app.isActive
guard let petInfo=screenWindows.first(where:{($0[kCGWindowOwnerPID as String] as? Int)==Int(app.processIdentifier) && ($0[kCGWindowName as String] as? String)=="Yonda"}),
      let bounds=petInfo[kCGWindowBounds as String] as? [String:CGFloat],
      let x=bounds["X"],let y=bounds["Y"],let width=bounds["Width"],let height=bounds["Height"] else {exit(3)}
let center=CGPoint(x:x+width/2,y:y+height/2)
print("inactive_before_click=\(inactive) center=\(center)")
CGEvent(mouseEventSource:nil,mouseType:.mouseMoved,mouseCursorPosition:CGPoint(x:x-30,y:y-30),mouseButton:.left)?.post(tap:.cghidEventTap)
Thread.sleep(forTimeInterval:1.5)
CGEvent(mouseEventSource:nil,mouseType:.mouseMoved,mouseCursorPosition:center,mouseButton:.left)?.post(tap:.cghidEventTap)
Thread.sleep(forTimeInterval:0.8)
var hoverPreservedInactive = !app.isActive
print("active_after_hover=\(app.isActive) visible_menu=\(visibleWindow() != nil)")
while !(window().map{hasEmpty($0)} ?? false) && ProcessInfo.processInfo.systemUptime<end {Thread.sleep(forTimeInterval:0.2)}
hoverPreservedInactive = !app.isActive
guard inactive,hoverPreservedInactive,let original=window(),hasEmpty(original),let visible=visibleWindow(),let id=visible[kCGWindowNumber as String] as? Int else{exit(4)}
let capture=Process();capture.executableURL=URL(fileURLWithPath:"/usr/sbin/screencapture");capture.arguments=["-x","-l",String(id),output.appendingPathComponent("native-empty.png").path];try capture.run();capture.waitUntilExit()
guard capture.terminationStatus==0 else{exit(5)}
guard let button=find(original,"关闭任务总览"),AXUIElementPerformAction(button,"AXPress" as CFString) == .success else{exit(6)}
Thread.sleep(forTimeInterval:0.8)
guard visibleWindow()==nil else{exit(7)}
guard let extras=attr(ax,"AXExtrasMenuBar"),CFGetTypeID(extras)==AXUIElementGetTypeID(),let show=find(extras as! AXUIElement,"任务总览"),AXUIElementPerformAction(show,"AXPress" as CFString) == .success else{exit(8)}
let restoreEnd=ProcessInfo.processInfo.systemUptime+3
while visibleWindow()==nil && ProcessInfo.processInfo.systemUptime<restoreEnd {Thread.sleep(forTimeInterval:0.2)}
guard let restored=visibleWindow(),(restored[kCGWindowNumber as String] as? Int)==id else{exit(9)}
let report:[String:Any]=["pid":Int(app.processIdentifier),"window_id":id,"pet_hover_opens_menu":true,"hover_preserves_inactive":hoverPreservedInactive,"inactive_before_click":inactive,"real_empty_state":true,"close_hides":true,"tray_restores_same_window":true,"passed":true]
let data=try JSONSerialization.data(withJSONObject:report,options:[.prettyPrinted,.sortedKeys]);try data.write(to:output.appendingPathComponent("result.json"));print(String(data:data,encoding:.utf8)!)
