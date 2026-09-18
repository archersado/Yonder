import AppKit
import ApplicationServices
guard (3...4).contains(CommandLine.arguments.count),let pid=pid_t(CommandLine.arguments[1]),let app=NSRunningApplication(processIdentifier:pid),app.bundleIdentifier == "com.yonder.e0-spike",AXIsProcessTrusted() else {exit(2)}
let input=CommandLine.arguments.count == 4 ? CommandLine.arguments[3] : "--press"
guard ["--press","--enter","--space"].contains(input) else {exit(2)}
let output=URL(fileURLWithPath:CommandLine.arguments[2])
guard !FileManager.default.fileExists(atPath:output.path) else {exit(2)}
FileManager.default.createFile(atPath:output.path,contents:nil)
let log=try FileHandle(forWritingTo:output)
func emit(_ value:[String:Any]) {let d=try! JSONSerialization.data(withJSONObject:value,options:[.sortedKeys]);log.write(d);log.write(Data([10]));print(String(data:d,encoding:.utf8)!);fflush(stdout)}
func attr(_ e:AXUIElement,_ key:String)->CFTypeRef? {var v:CFTypeRef?;return AXUIElementCopyAttributeValue(e,key as CFString,&v) == .success ? v:nil}
func button(_ e:AXUIElement,_ label:String,_ depth:Int=0)->AXUIElement? {
 guard depth<16 else{return nil}
 if attr(e,"AXRole") as? String == "AXButton",attr(e,"AXDescription") as? String == label {return e}
 for c in attr(e,"AXChildren") as? [AXUIElement] ?? [] {if let b=button(c,label,depth+1){return b}}
 return nil
}
let ax=AXUIElementCreateApplication(pid)
func petWindow()->AXUIElement? {(attr(ax,"AXWindows") as? [AXUIElement] ?? []).first{attr($0,"AXTitle") as? String == "Yonder E0"}}
let deadline=ProcessInfo.processInfo.systemUptime+210
var target:AXUIElement?
var count=0
while ProcessInfo.processInfo.systemUptime < deadline {
 if let w=petWindow(),let b=button(w,"点击趴在边缘的 Yonda 唤醒它") {target=b;break}
 Thread.sleep(forTimeInterval:0.5);count += 1
 if count % 40 == 0 {emit(["waiting_seconds":Double(count)/2])}
}
guard let target else {emit(["result":"未观察到休眠按钮，不发送激活"]);exit(3)}
emit(["before_role":"AXButton","before_label":"点击趴在边缘的 Yonda 唤醒它","app_active":app.isActive])
let start=ProcessInfo.processInfo.systemUptime
let pressed:AXError
if input == "--press" {
 pressed=AXUIElementPerformAction(target,"AXPress" as CFString)
} else {
 if attr(target,"AXFocused") as? Bool != true {
  let focused=AXUIElementSetAttributeValue(target,"AXFocused" as CFString,kCFBooleanTrue)
  emit(["focus_result":focused.rawValue])
 }
 guard attr(target,"AXFocused") as? Bool == true else {emit(["result":"目标按钮未获焦点，不发送按键"]);exit(4)}
 // 使用当前公开CG接口定向目标PID，不向全局事件流投递按键。
 let key:CGKeyCode=input == "--enter" ? 36 : 49
 guard CGPreflightPostEventAccess(),let down=CGEvent(keyboardEventSource:nil,virtualKey:key,keyDown:true),let up=CGEvent(keyboardEventSource:nil,virtualKey:key,keyDown:false) else {emit(["result":"无法构造或发送键盘事件"]);exit(4)}
 down.postToPid(pid);Thread.sleep(forTimeInterval:0.05);up.postToPid(pid)
 pressed = .success
 emit(["input":input,"posted_to_pid":Int(pid),"target_focused":true,"post_access":true])
}
guard pressed == .success else {emit(["press_result":pressed.rawValue]);exit(4)}
while ProcessInfo.processInfo.systemUptime-start < 3 {
 if let w=petWindow(),button(w,"轻点 Yonda 小龙，按住移动可拖动") != nil {
  let all=CGWindowListCopyWindowInfo([.optionAll,.excludeDesktopElements],kCGNullWindowID) as? [[String:Any]] ?? []
  if let pet=all.first(where:{($0[kCGWindowOwnerPID as String] as? Int)==Int(pid) && ($0[kCGWindowName as String] as? String)=="Yonder E0"}),let b=pet[kCGWindowBounds as String] as? NSDictionary,let r=CGRect(dictionaryRepresentation:b),(200...201).contains(r.width),(200...201).contains(r.height) {
   emit(["input":input,"ax_press_result":input == "--press" ? 0 : -1,"after_label":"轻点 Yonda 小龙，按住移动可拖动","width":r.width,"height":r.height,"window_id":pet[kCGWindowNumber as String] ?? 0,"latency_ms":(ProcessInfo.processInfo.systemUptime-start)*1000,"passed":true]);exit(0)
  }
 }
 Thread.sleep(forTimeInterval:0.02)
}
emit(["input":input,"ax_press_result":input == "--press" ? 0 : -1,"passed":false,"result":"3秒内未确认清醒按钮及展开尺寸"]);exit(5)
