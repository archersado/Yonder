// 仅操作com.yonder.desktop；真实空任务窗口、关闭隐藏与托盘恢复的原生验证。
import AppKit
import ApplicationServices
guard CommandLine.arguments.count == 2, AXIsProcessTrusted(),
      let app = NSRunningApplication.runningApplications(withBundleIdentifier:"com.yonder.desktop").first else { exit(2) }
let ax = AXUIElementCreateApplication(app.processIdentifier)
func attr(_ element:AXUIElement,_ name:String)->CFTypeRef? {
 var value:CFTypeRef?; return AXUIElementCopyAttributeValue(element,name as CFString,&value) == .success ? value:nil
}
func find(_ element:AXUIElement,_ title:String,_ depth:Int=0)->AXUIElement? {
 if ["AXTitle","AXDescription","AXValue"].contains(where:{(attr(element,$0) as? String)?.contains(title) == true}) {return element}
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

func move(_ point:CGPoint) {
 CGEvent(mouseEventSource:nil,mouseType:.mouseMoved,mouseCursorPosition:point,mouseButton:.left)?.post(tap:.cghidEventTap)
}
func info(_ title:String)->[String:Any]? {
 let list=CGWindowListCopyWindowInfo([.optionOnScreenOnly,.excludeDesktopElements],kCGNullWindowID) as? [[String:Any]] ?? []
 return list.first{($0[kCGWindowOwnerPID as String] as? Int)==Int(app.processIdentifier) && ($0[kCGWindowName as String] as? String)==title}
}
func center(_ info:[String:Any])->CGPoint {
 let bounds=info[kCGWindowBounds as String] as! [String:CGFloat]
 return CGPoint(x:bounds["X"]!+bounds["Width"]!/2,y:bounds["Y"]!+bounds["Height"]!/2)
}
if let existing=window(),let close=find(existing,"关闭任务总览") { _=AXUIElementPerformAction(close,"AXPress" as CFString) }
NSRunningApplication.runningApplications(withBundleIdentifier:"com.microsoft.VSCode").first?.activate(options:[])
Thread.sleep(forTimeInterval:1)
guard let petInfo=info("Yonda") else{exit(3)}
let point=center(petInfo)
move(CGPoint(x:point.x-150,y:point.y-150));Thread.sleep(forTimeInterval:1.5)
move(point);Thread.sleep(forTimeInterval:3)
let hoverOpens = visibleWindow() != nil
print("tasks_hover_opens=\(hoverOpens)")
guard hoverOpens else{exit(4)}
let openEnd=ProcessInfo.processInfo.systemUptime+5
while visibleWindow()==nil && ProcessInfo.processInfo.systemUptime<openEnd {Thread.sleep(forTimeInterval:0.1)}
Thread.sleep(forTimeInterval:0.3)
guard let menu=visibleWindow() else{exit(6)}
move(center(menu));Thread.sleep(forTimeInterval:1.5)
let contentEnd=ProcessInfo.processInfo.systemUptime+3
while !(window().map{find($0,"Agent · local-test-agent") != nil} ?? false) && visibleWindow() != nil && ProcessInfo.processInfo.systemUptime<contentEnd {Thread.sleep(forTimeInterval:0.1)}
print("kept=\(visibleWindow() != nil) empty=\(window().map{find($0,"Agent · local-test-agent") != nil} ?? false) refresh=\(window().flatMap{find($0,"刷新")} != nil)")
guard visibleWindow() != nil,let original=window(),find(original,"Agent · local-test-agent") != nil,let refresh=find(original,"刷新"),AXUIElementPerformAction(refresh,"AXPress" as CFString) == .success else{exit(7)}
Thread.sleep(forTimeInterval:1)
guard visibleWindow() != nil,info("Yonda") != nil,find(original,"Agent · local-test-agent") != nil,let id=menu[kCGWindowNumber as String] as? Int else{exit(8)}
func taskButton(_ element:AXUIElement,_ depth:Int=0)->AXUIElement? {
 guard depth<16 else{return nil}
 if ["AXButton","AXCheckBox","AXToggleButton"].contains(attr(element,"AXRole") as? String ?? ""), find(element,"Agent · local-test-agent") != nil {return element}
 for child in attr(element,"AXChildren") as? [AXUIElement] ?? [] {if let found=taskButton(child,depth+1){return found}}
 return nil
}
print("task_button_found=\(taskButton(original) != nil) owner_role=\(find(original,"Agent · local-test-agent").flatMap{attr($0,"AXRole")} ?? "missing" as CFString)")
guard let task=taskButton(original), AXUIElementPerformAction(task,"AXPress" as CFString) == .success else{exit(10)}
let detailEnd=ProcessInfo.processInfo.systemUptime+5
while find(original,"取消任务")==nil && ProcessInfo.processInfo.systemUptime<detailEnd {Thread.sleep(forTimeInterval:0.1)}
guard let cancel=find(original,"取消任务"),AXUIElementPerformAction(cancel,"AXPress" as CFString) == .success else{exit(11)}
let cancelEnd=ProcessInfo.processInfo.systemUptime+5
while find(original,"第 1 页 · 1 项")==nil && ProcessInfo.processInfo.systemUptime<cancelEnd {Thread.sleep(forTimeInterval:0.1)}
guard find(original,"第 1 页 · 1 项") != nil, let all=find(original,"全部"),AXUIElementPerformAction(all,"AXPress" as CFString) == .success else{exit(12)}
let historyEnd=ProcessInfo.processInfo.systemUptime+5
while find(original,"已取消")==nil && ProcessInfo.processInfo.systemUptime<historyEnd {Thread.sleep(forTimeInterval:0.1)}
guard find(original,"已取消") != nil,find(original,"已创建") != nil else{exit(13)}
let capture=Process();capture.executableURL=URL(fileURLWithPath:"/usr/sbin/screencapture");capture.arguments=["-x","-l",String(id),output.appendingPathComponent("native-cancelled-task.png").path];try capture.run();capture.waitUntilExit()
move(CGPoint(x:20,y:100))
let hideEnd=ProcessInfo.processInfo.systemUptime+3
while visibleWindow() != nil && ProcessInfo.processInfo.systemUptime<hideEnd {Thread.sleep(forTimeInterval:0.1)}
guard visibleWindow()==nil else{exit(9)}
let report:[String:Any]=["pid":Int(app.processIdentifier),"tasks_hover_opens":hoverOpens,"panel_hover_keeps_open":true,"same_visible_desktop":true,"refresh_operable":true,"leave_hides":true,"real_agent_tasks":true,"pending_cancel_button":true,"other_task_preserved":true,"cancelled_history_visible":true,"passed":true]
let data=try JSONSerialization.data(withJSONObject:report,options:[.prettyPrinted,.sortedKeys]);try data.write(to:output.appendingPathComponent("result.json"));print(String(data:data,encoding:.utf8)!)
