// 原生窗口定位Spike；仅由隔离夹具传PID/WindowServer ID，不进入产品。
import AppKit
import ApplicationServices
import Darwin
func finish(_ value:[String:Any],_ code:Int32=0)->Never {
 let data=try! JSONSerialization.data(withJSONObject:value,options:[.sortedKeys])
 FileHandle.standardOutput.write(data+Data([10]));exit(code)
}
let retainedMode = CommandLine.arguments.count==4 && CommandLine.arguments[3]=="--retain"
guard (CommandLine.arguments.count==3 || retainedMode),let pid=Int32(CommandLine.arguments[1]),let id=UInt32(CommandLine.arguments[2]),pid>0,id>0,AXIsProcessTrusted() else{finish(["refused":true,"invalid_or_permission":true],2)}
func info()->[String:Any]? {
 let windows=CGWindowListCopyWindowInfo(.optionAll,kCGNullWindowID) as? [[String:Any]] ?? []
 return windows.first{($0[kCGWindowNumber as String] as? UInt32)==id && ($0[kCGWindowOwnerPID as String] as? Int32)==pid && ($0[kCGWindowLayer as String] as? Int)==0}
}
func frame(_ item:[String:Any])->CGRect? {
 guard let b=item[kCGWindowBounds as String] as? [String:CGFloat],let x=b["X"],let y=b["Y"],let w=b["Width"],let h=b["Height"] else{return nil}
 return CGRect(x:x,y:y,width:w,height:h)
}
var axErrors:[String:Int]=[:]
func attr(_ element:AXUIElement,_ name:String)->CFTypeRef? {
 var value:CFTypeRef?;let code=AXUIElementCopyAttributeValue(element,name as CFString,&value);if code != .success {axErrors[name]=Int(code.rawValue)};return code == .success ? value:nil
}
func axFrame(_ element:AXUIElement)->CGRect? {
 guard let p=attr(element,"AXPosition"),let s=attr(element,"AXSize"),CFGetTypeID(p)==AXValueGetTypeID(),CFGetTypeID(s)==AXValueGetTypeID() else{return nil}
 var point=CGPoint.zero;var size=CGSize.zero
 guard AXValueGetValue(p as! AXValue,.cgPoint,&point),AXValueGetValue(s as! AXValue,.cgSize,&size) else{return nil}
 return CGRect(origin:point,size:size)
}
func same(_ a:CGRect,_ b:CGRect)->Bool {abs(a.minX-b.minX)<1 && abs(a.minY-b.minY)<1 && abs(a.width-b.width)<1 && abs(a.height-b.height)<1}
guard let original=info(),let initialBounds=frame(original),let title=original[kCGWindowName as String] as? String,!title.isEmpty,let app=NSRunningApplication(processIdentifier:pid) else{finish(["refused":true,"target_missing":true],3)}
var bounds=initialBounds
let ax=AXUIElementCreateApplication(pid)
// WindowServer先于AX注册就绪；只轮询只读映射，尚不派发任何焦点操作。
let readyEnd=ProcessInfo.processInfo.systemUptime+2
var axWindows:[AXUIElement]=[]
var candidates:[AXUIElement]=[]
repeat {
 if let fresh=info(),let freshBounds=frame(fresh) {bounds=freshBounds}
 axWindows=attr(ax,"AXWindows") as? [AXUIElement] ?? []
 candidates=axWindows.filter{element in
  guard attr(element,"AXTitle") as? String==title,let rect=axFrame(element) else{return false}
  return same(rect,bounds)
 }
 if !candidates.isEmpty || info()==nil {break}
 _=RunLoop.current.run(mode:.default,before:Date(timeIntervalSinceNow:0.05))
} while ProcessInfo.processInfo.systemUptime<readyEnd
guard candidates.count==1,let target=candidates.first,info() != nil else{finish(["refused":true,"mapping_not_unique":true,"ax_errors":axErrors,"ax_windows":axWindows.count,"title_matches":axWindows.filter{attr($0,"AXTitle") as? String==title}.count,"frame_matches":axWindows.filter{axFrame($0).map{same($0,bounds)}==true}.count],4)}
func processStart(_ value: Int32) -> [UInt64]? {
 var data=proc_bsdinfo()
 let size=Int32(MemoryLayout<proc_bsdinfo>.size)
 guard proc_pidinfo(value,PROC_PIDTBSDINFO,0,&data,size)==size, data.pbi_start_tvsec>0 else{return nil}
 return [data.pbi_start_tvsec,data.pbi_start_tvusec]
}
func emit(_ value:[String:Any]) {
 let data=try! JSONSerialization.data(withJSONObject:value,options:[.sortedKeys])
 FileHandle.standardOutput.write(data+Data([10]))
}
func activate(_ target:AXUIElement,_ originalBounds:CGRect)->[String:Any] {
 let minimized=attr(target,"AXMinimized") as? Bool ?? false
 if minimized && AXUIElementSetAttributeValue(target,"AXMinimized" as CFString,kCFBooleanFalse) != .success {return ["refused":true,"restore_failed":true]}
 guard AXUIElementSetAttributeValue(target,"AXMain" as CFString,kCFBooleanTrue) == .success,
       AXUIElementPerformAction(target,"AXRaise" as CFString) == .success,
       AXUIElementSetAttributeValue(ax,"AXFrontmost" as CFString,kCFBooleanTrue) == .success else{return ["refused":true,"activation_failed":true]}
 let end=ProcessInfo.processInfo.systemUptime+3
 var focused=false
 while ProcessInfo.processInfo.systemUptime<end {
  let focusedWindow=attr(ax,"AXFocusedWindow")
  focused=(attr(ax,"AXFrontmost") as? Bool)==true && focusedWindow.map{CFEqual($0,target)}==true && (attr(target,"AXMinimized") as? Bool)==false
  if focused {break};Thread.sleep(forTimeInterval:0.05)
 }
 guard let final=info(),let current=frame(final) else{return ["refused":true,"target_missing_after_focus":true]}
 return ["refused":!focused,"focused":focused,"restored":!minimized || (attr(target,"AXMinimized") as? Bool)==false,"geometry_unchanged":same(originalBounds,current),"unique_mapping":true]
}
if !retainedMode {
 let result=activate(target,bounds)
 finish(result,(result["focused"] as? Bool)==true && (result["geometry_unchanged"] as? Bool)==true ? 0:8)
}
guard let capturedStart=processStart(pid) else{finish(["refused":true,"reason":"process_identity_unavailable"],9)}
// 仅此持久探针保留原AX对象；每次操作重新复核，不重新按名称获取替代目标。
func rejection(_ expectedStart:[UInt64],_ permitted:Bool=true)->String? {
 guard permitted && AXIsProcessTrusted() else{return "permission_unavailable"}
 guard processStart(pid)==expectedStart else{return "process_identity_changed"}
 guard let current=info(),let currentBounds=frame(current) else{return "window_missing"}
 let currentWindows=attr(ax,"AXWindows") as? [AXUIElement] ?? []
 guard currentWindows.contains(where:{CFEqual($0,target)}) else{return "retained_object_missing"}
 guard let retainedBounds=axFrame(target),same(retainedBounds,currentBounds),attr(target,"AXTitle") as? String==current[kCGWindowName as String] as? String else{return "retained_object_invalid"}
 let matches=currentWindows.filter { element in
  attr(element,"AXTitle") as? String==current[kCGWindowName as String] as? String && axFrame(element).map{same($0,currentBounds)}==true
 }
 guard matches.count==1,CFEqual(matches[0],target) else{return "mapping_not_unique"}
 return nil
}
emit(["ready":true,"retained_object":true,"process_start_available":true])
while let command=readLine() {
 if command=="quit" {break}
 let expected=command=="changed-start" ? [capturedStart[0]+1,capturedStart[1]]:capturedStart
 if let reason=rejection(expected,command != "deny-permission") {emit(["refused":true,"reason":reason,"side_effect_dispatched":false]);continue}
 if command=="focus" {
  guard let current=info(),let rect=frame(current),rejection(capturedStart)==nil else{emit(["refused":true,"reason":"changed_before_focus","side_effect_dispatched":false]);continue}
  var result=activate(target,rect);result["side_effect_dispatched"]=true;emit(result)
 } else if command=="verify" {emit(["refused":false,"valid":true,"side_effect_dispatched":false])}
 else {emit(["refused":true,"reason":"invalid_command","side_effect_dispatched":false])}
}
