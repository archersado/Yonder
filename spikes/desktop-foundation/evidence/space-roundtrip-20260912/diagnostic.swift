import AppKit
import ApplicationServices
let pid = pid_t(CommandLine.arguments[1])!
let output = URL(fileURLWithPath:CommandLine.arguments[2],isDirectory:true)
guard let petApp = NSRunningApplication(processIdentifier:pid), petApp.bundleIdentifier == "com.yonder.e0-spike" else { fatalError("非目标桌宠") }
try FileManager.default.createDirectory(at:output,withIntermediateDirectories:true)
func emit(_ row:[String:Any]) { let data=try! JSONSerialization.data(withJSONObject:row,options:[.sortedKeys]); print(String(data:data,encoding:.utf8)!); fflush(stdout); let url=output.appendingPathComponent("spaces.jsonl"); if !FileManager.default.fileExists(atPath:url.path) { FileManager.default.createFile(atPath:url.path,contents:nil) }; let f=try! FileHandle(forWritingTo:url); f.seekToEndOfFile(); f.write(data); f.write(Data([10])); try! f.close() }
func fullscreen() -> Bool? {
 guard let front=NSWorkspace.shared.frontmostApplication else { return nil }
 let app=AXUIElementCreateApplication(front.processIdentifier)
 var value:CFTypeRef?
 guard AXUIElementCopyAttributeValue(app,"AXFocusedWindow" as CFString,&value) == .success, let value, CFGetTypeID(value) == AXUIElementGetTypeID() else { return nil }
 var state:CFTypeRef?
 guard AXUIElementCopyAttributeValue(value as! AXUIElement,"AXFullScreen" as CFString,&state) == .success else { return nil }
 return state as? Bool
}
func sample(_ phase:String) {
 let all=CGWindowListCopyWindowInfo([.optionAll,.excludeDesktopElements],kCGNullWindowID) as? [[String:Any]] ?? []
 guard let pet=all.first(where:{ ($0[kCGWindowOwnerPID as String] as? Int) == Int(pid) && ($0[kCGWindowName as String] as? String) == "Yonder E0" }), let b=pet[kCGWindowBounds as String] as? NSDictionary, let r=CGRect(dictionaryRepresentation:b) else { emit(["error":"桌宠窗口丢失","phase":phase]); return }
 let p=Process(); p.executableURL=URL(fileURLWithPath:"/usr/sbin/screencapture"); p.arguments=["-x","-R","\(Int(r.minX)),\(Int(r.minY)),\(Int(r.width)),\(Int(r.height))",output.appendingPathComponent(phase+".png").path]; try! p.run();p.waitUntilExit()
 emit(["phase":phase,"window_id":pet[kCGWindowNumber as String] ?? 0,"bounds":[r.minX,r.minY,r.width,r.height],"front_window_fullscreen":fullscreen() as Any? ?? "unknown","capture_exit":p.terminationStatus])
}
func switchSpace(_ key:CGKeyCode) {
 for down in [true,false] { let e=CGEvent(keyboardEventSource:nil,virtualKey:key,keyDown:down)!; e.flags = .maskControl; e.post(tap:.cghidEventTap); Thread.sleep(forTimeInterval:0.05) }
}
var changes=0
let token=NSWorkspace.shared.notificationCenter.addObserver(forName:NSWorkspace.activeSpaceDidChangeNotification,object:nil,queue:.main) { _ in changes += 1 }
defer { NSWorkspace.shared.notificationCenter.removeObserver(token) }
func wait(_ seconds:Double) { RunLoop.current.run(until:Date().addingTimeInterval(seconds)) }
sample("before")
switchSpace(123) // Control+Left；仅在确实收到切换通知后才向右返回。
wait(2)
if changes == 0 { emit(["result":"未观察到切屏，未发送反向快捷键，不算通过"]); exit(2) }
sample("left")
let previous=changes
switchSpace(124)
wait(2)
sample("returned")
emit(["space_notifications":changes,"return_notification":changes>previous,"result":"切屏往返取证；普通/全屏属性及图像需核对"])
exit(changes>previous ? 0 : 3)
