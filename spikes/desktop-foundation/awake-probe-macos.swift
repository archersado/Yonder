// 仅供性能验证：限定 Yonda，读取窗口边界；wake 模式通过既有托盘入口重置闲置计时。
import AppKit
import ApplicationServices
import ImageIO

guard CommandLine.arguments.count >= 3, let pid = pid_t(CommandLine.arguments[1]),
      let app = NSRunningApplication(processIdentifier:pid), app.bundleIdentifier == "com.yonder.e0-spike" else { exit(2) }
let mode=CommandLine.arguments[2]
guard mode == "wake" || mode == "check" || mode == "quit" else { exit(2) }
func attr(_ e:AXUIElement,_ key:String)->CFTypeRef? {var v:CFTypeRef?;return AXUIElementCopyAttributeValue(e,key as CFString,&v) == .success ? v : nil}
func find(_ e:AXUIElement,_ depth:Int=0)->AXUIElement? {
 if attr(e,"AXTitle") as? String == (mode == "quit" ? "退出 Yonda" : "显示小龙") {return e}
 guard depth<5 else{return nil}
 for child in attr(e,"AXChildren") as? [AXUIElement] ?? [] {if let result=find(child,depth+1){return result}}
 return nil
}
if mode == "wake" || mode == "quit" {
 let ax=AXUIElementCreateApplication(pid)
 guard let menu=attr(ax,"AXExtrasMenuBar"),CFGetTypeID(menu)==AXUIElementGetTypeID(),let show=find(menu as! AXUIElement),AXUIElementPerformAction(show,"AXPress" as CFString) == .success else {exit(3)}
 if mode == "quit" { print("{\"quit_ax_result\":0}"); exit(0) }
 Thread.sleep(forTimeInterval:0.8)
}
let all=CGWindowListCopyWindowInfo([.optionAll,.excludeDesktopElements],kCGNullWindowID) as? [[String:Any]] ?? []
guard let pet=all.first(where:{($0[kCGWindowOwnerPID as String] as? Int)==Int(pid) && ($0[kCGWindowName as String] as? String)=="Yonder E0"}),let b=pet[kCGWindowBounds as String] as? NSDictionary,let r=CGRect(dictionaryRepresentation:b),let id=pet[kCGWindowNumber as String] as? Int else{exit(4)}
var result:[String:Any]=["window_id":id,"width":r.width,"height":r.height,"onscreen":pet[kCGWindowIsOnscreen as String] as? Bool ?? false,"app_hidden":app.isHidden,"wake_requested":mode == "wake"]
if CommandLine.arguments.count == 4 {
 let path=CommandLine.arguments[3]
 let p=Process();p.executableURL=URL(fileURLWithPath:"/usr/sbin/screencapture");p.arguments=["-x","-l",String(id),path];try p.run();p.waitUntilExit()
 guard p.terminationStatus == 0,let source=CGImageSourceCreateWithURL(URL(fileURLWithPath:path) as CFURL,nil),let image=CGImageSourceCreateImageAtIndex(source,0,nil) else{exit(5)}
 result["image_width"]=image.width;result["image_height"]=image.height
 guard image.width>=200,image.height>=200 else{exit(6)}
}
let data=try JSONSerialization.data(withJSONObject:result,options:[.sortedKeys]);print(String(data:data,encoding:.utf8)!)
// CG 边界可能因缩放取整为201；onscreen 曾与目视不一致，仅记录，不单独作门禁。
guard (200...201).contains(r.width),(200...201).contains(r.height),!app.isHidden else{exit(7)}
