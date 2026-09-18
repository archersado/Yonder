import AppKit
import ApplicationServices

guard CommandLine.arguments.count == 3 else { fatalError("参数：Yonda PID 证据目录") }
let petPID = pid_t(CommandLine.arguments[1])!
guard let petApp = NSRunningApplication(processIdentifier:petPID), petApp.bundleIdentifier == "com.yonder.e0-spike" else { fatalError("非目标桌宠") }
let output = URL(fileURLWithPath:CommandLine.arguments[2],isDirectory:true)
try FileManager.default.createDirectory(at:output,withIntermediateDirectories:true)
let app = NSApplication.shared
app.setActivationPolicy(.regular)
let test = NSWindow(contentRect:NSRect(x:0,y:0,width:900,height:700),styleMask:[.titled,.closable,.resizable,.miniaturizable],backing:.buffered,defer:false)
test.title = "Yonda 全屏验证（自动关闭）"
test.backgroundColor = NSColor(calibratedRed:0.10,green:0.45,blue:0.50,alpha:1)
test.collectionBehavior = [.fullScreenPrimary]
func emit(_ row:[String:Any]) {
 let data = try! JSONSerialization.data(withJSONObject:row,options:[.sortedKeys])
 print(String(data:data,encoding:.utf8)!); fflush(stdout)
 let url = output.appendingPathComponent("fullscreen.jsonl")
 if !FileManager.default.fileExists(atPath:url.path) { FileManager.default.createFile(atPath:url.path,contents:nil) }
 let file = try! FileHandle(forWritingTo:url); file.seekToEndOfFile(); file.write(data); file.write(Data([10])); try! file.close()
}
func sample(_ phase:String) {
 let windows = CGWindowListCopyWindowInfo([.optionAll,.excludeDesktopElements],kCGNullWindowID) as? [[String:Any]] ?? []
 guard let pet = windows.first(where:{ ($0[kCGWindowOwnerPID as String] as? Int) == Int(petPID) && ($0[kCGWindowName as String] as? String) == "Yonder E0" }), let b = pet[kCGWindowBounds as String] as? NSDictionary, let r = CGRect(dictionaryRepresentation:b) else { emit(["phase":phase,"error":"没有桌宠窗口"]); return }
 let capture = Process(); capture.executableURL = URL(fileURLWithPath:"/usr/sbin/screencapture")
 capture.arguments = ["-x","-R","\(Int(r.minX)),\(Int(r.minY)),\(Int(r.width)),\(Int(r.height))",output.appendingPathComponent(phase+"-screen.png").path]
 try! capture.run(); capture.waitUntilExit()
 emit(["phase":phase,"pid":Int(petPID),"window_id":pet[kCGWindowNumber as String] ?? 0,"pet_bounds":[r.minX,r.minY,r.width,r.height],"onscreen":pet[kCGWindowIsOnscreen as String] as? Bool ?? false,"test_fullscreen":test.styleMask.contains(.fullScreen),"test_active":app.isActive,"test_frame":[test.frame.minX,test.frame.minY,test.frame.width,test.frame.height],"capture_exit":capture.terminationStatus])
}
class Delegate:NSObject,NSWindowDelegate {
 func windowDidEnterFullScreen(_ notification:Notification) {
  DispatchQueue.main.asyncAfter(deadline:.now()+1) {
   sample("fullscreen-1s")
   DispatchQueue.main.asyncAfter(deadline:.now()+3) {
    sample("fullscreen-4s")
    test.toggleFullScreen(nil)
   }
  }
 }
 func windowDidExitFullScreen(_ notification:Notification) {
  DispatchQueue.main.asyncAfter(deadline:.now()+1) {
   sample("restored")
   test.orderOut(nil)
   emit(["result":"全屏与返回取证完成；截图需人工核对"])
   app.terminate(nil)
  }
 }
}
emit(["pet_activation_policy":petApp.activationPolicy.rawValue,"meaning":"0=regular, 1=accessory, 2=prohibited"])
let delegate = Delegate(); test.delegate = delegate
test.center(); test.makeKeyAndOrderFront(nil)
app.activate(ignoringOtherApps:true)
DispatchQueue.main.asyncAfter(deadline:.now()+2) {
 sample("normal")
 test.toggleFullScreen(nil)
}
DispatchQueue.main.asyncAfter(deadline:.now()+25) {
 emit(["result":"等待全屏回调超时，关闭临时窗口"])
 test.orderOut(nil); app.terminate(nil)
}
app.run()
