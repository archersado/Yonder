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
guard let pet=find(petWindow,"轻点 Yonda 小龙，按住移动可拖动"),AXUIElementPerformAction(pet,"AXPress" as CFString) == .success else{print("awake_pet_activation_missing=true");exit(3)}
Thread.sleep(forTimeInterval:0.8)
while !(window().map{hasEmpty($0)} ?? false) && ProcessInfo.processInfo.systemUptime<end {Thread.sleep(forTimeInterval:0.2)}
guard let original=window(),hasEmpty(original),let visible=visibleWindow(),let id=visible[kCGWindowNumber as String] as? Int else{exit(4)}
let capture=Process();capture.executableURL=URL(fileURLWithPath:"/usr/sbin/screencapture");capture.arguments=["-x","-l",String(id),output.appendingPathComponent("native-empty.png").path];try capture.run();capture.waitUntilExit()
guard capture.terminationStatus==0 else{exit(5)}
guard let button=find(original,"关闭任务总览"),AXUIElementPerformAction(button,"AXPress" as CFString) == .success else{exit(6)}
Thread.sleep(forTimeInterval:0.8)
guard visibleWindow()==nil else{exit(7)}
guard let extras=attr(ax,"AXExtrasMenuBar"),CFGetTypeID(extras)==AXUIElementGetTypeID(),let show=find(extras as! AXUIElement,"任务总览"),AXUIElementPerformAction(show,"AXPress" as CFString) == .success else{exit(8)}
Thread.sleep(forTimeInterval:0.8)
guard let restored=visibleWindow(),(restored[kCGWindowNumber as String] as? Int)==id else{exit(9)}
let report:[String:Any]=["pid":Int(app.processIdentifier),"window_id":id,"pet_activation_opens_menu":true,"real_empty_state":true,"close_hides":true,"tray_restores_same_window":true,"passed":true]
let data=try JSONSerialization.data(withJSONObject:report,options:[.prettyPrinted,.sortedKeys]);try data.write(to:output.appendingPathComponent("result.json"));print(String(data:data,encoding:.utf8)!)
