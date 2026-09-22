// 仅操作com.yonder.desktop；验证生产Agent任务的原生悬停、详情与隐藏。
import AppKit
import ApplicationServices

guard CommandLine.arguments.count == 2, AXIsProcessTrusted(),
      let app = NSRunningApplication.runningApplications(withBundleIdentifier:"com.yonder.desktop").first else { exit(2) }
let ax = AXUIElementCreateApplication(app.processIdentifier)

func attr(_ element:AXUIElement,_ name:String)->CFTypeRef? {
    var value:CFTypeRef?
    return AXUIElementCopyAttributeValue(element,name as CFString,&value) == .success ? value:nil
}
func find(_ element:AXUIElement,_ title:String,_ depth:Int=0)->AXUIElement? {
    guard depth < 16 else { return nil }
    if ["AXTitle","AXDescription","AXValue"].contains(where:{(attr(element,$0) as? String)?.contains(title) == true}) { return element }
    for child in attr(element,"AXChildren") as? [AXUIElement] ?? [] { if let found = find(child,title,depth + 1) { return found } }
    return nil
}
func findButton(_ element:AXUIElement,_ title:String,_ depth:Int=0)->AXUIElement? {
    guard depth < 16 else { return nil }
    if ["AXButton","AXCheckBox","AXToggleButton"].contains(attr(element,"AXRole") as? String ?? ""),
       ["AXTitle","AXDescription","AXValue"].contains(where:{(attr(element,$0) as? String)?.contains(title) == true}) { return element }
    for child in attr(element,"AXChildren") as? [AXUIElement] ?? [] { if let found = findButton(child,title,depth + 1) { return found } }
    return nil
}

let output = URL(fileURLWithPath:CommandLine.arguments[1],isDirectory:true)
guard !FileManager.default.fileExists(atPath:output.path) else { exit(2) }
try FileManager.default.createDirectory(at:output,withIntermediateDirectories:true)

func window()->AXUIElement? {
    (attr(ax,"AXWindows") as? [AXUIElement] ?? []).first{attr($0,"AXTitle") as? String == "Yonda · Task Space"}
}
func visibleWindow()->[String:Any]? {
    let list = CGWindowListCopyWindowInfo([.optionOnScreenOnly,.excludeDesktopElements],kCGNullWindowID) as? [[String:Any]] ?? []
    return list.first{($0[kCGWindowOwnerPID as String] as? Int) == Int(app.processIdentifier) && ($0[kCGWindowName as String] as? String) == "Yonda · Task Space"}
}
func info(_ title:String)->[String:Any]? {
    let list = CGWindowListCopyWindowInfo([.optionOnScreenOnly,.excludeDesktopElements],kCGNullWindowID) as? [[String:Any]] ?? []
    return list.first{($0[kCGWindowOwnerPID as String] as? Int) == Int(app.processIdentifier) && ($0[kCGWindowName as String] as? String) == title}
}
func center(_ item:[String:Any])->CGPoint {
    let bounds = item[kCGWindowBounds as String] as! [String:CGFloat]
    return CGPoint(x:bounds["X"]! + bounds["Width"]!/2,y:bounds["Y"]! + bounds["Height"]!/2)
}
func rectangle(_ item:[String:Any])->CGRect {
    let bounds = item[kCGWindowBounds as String] as! [String:CGFloat]
    return CGRect(x:bounds["X"]!,y:bounds["Y"]!,width:bounds["Width"]!,height:bounds["Height"]!)
}
func move(_ point:CGPoint) {
    CGEvent(mouseEventSource:nil,mouseType:.mouseMoved,mouseCursorPosition:point,mouseButton:.left)?.post(tap:.cghidEventTap)
}

let readyEnd = ProcessInfo.processInfo.systemUptime + 15
while (attr(ax,"AXWindows") as? [AXUIElement] ?? []).first(where:{attr($0,"AXTitle") as? String == "Yonda"}) == nil,
      ProcessInfo.processInfo.systemUptime < readyEnd { Thread.sleep(forTimeInterval:0.2) }
guard let petWindow = (attr(ax,"AXWindows") as? [AXUIElement] ?? []).first(where:{attr($0,"AXTitle") as? String == "Yonda"}) else { exit(3) }
while find(petWindow,"轻点 Yonda 小龙，按住移动可拖动") == nil && find(petWindow,"点击趴在边缘的 Yonda 唤醒它") == nil,
      ProcessInfo.processInfo.systemUptime < readyEnd { Thread.sleep(forTimeInterval:0.2) }
if let eyes = find(petWindow,"点击趴在边缘的 Yonda 唤醒它") {
    guard AXUIElementPerformAction(eyes,"AXPress" as CFString) == .success else { exit(3) }
    Thread.sleep(forTimeInterval:0.8)
}

if let existing = window(),let close = find(existing,"关闭任务总览") { _ = AXUIElementPerformAction(close,"AXPress" as CFString) }
NSRunningApplication.runningApplications(withBundleIdentifier:"com.microsoft.VSCode").first?.activate(options:[])
Thread.sleep(forTimeInterval:1)
guard let petInfo = info("Yonda") else { exit(3) }
let petPoint = center(petInfo)
move(CGPoint(x:petPoint.x - 150,y:petPoint.y - 150))
Thread.sleep(forTimeInterval:1.5)
move(petPoint)
Thread.sleep(forTimeInterval:3)
guard let menu = visibleWindow() else { print("tasks_hover_opens=false"); exit(4) }
move(center(menu))
Thread.sleep(forTimeInterval:1.5)

guard visibleWindow() != nil,let panel = window(),findButton(panel,"全部") != nil else { exit(5) }
let refreshPanel = window()
guard let allButton = refreshPanel.flatMap{findButton($0,"全部")},AXUIElementPerformAction(allButton,"AXPress" as CFString) == .success else { exit(5) }
let allEnd = ProcessInfo.processInfo.systemUptime + 5
while !(window().map{find($0,"Agent · local-test-agent") != nil} ?? false),visibleWindow() != nil,ProcessInfo.processInfo.systemUptime < allEnd { Thread.sleep(forTimeInterval:0.2) }
guard visibleWindow() != nil,let shownPanel = window(),find(shownPanel,"Agent · local-test-agent") != nil,
      find(shownPanel,"整理桌面测试") != nil,find(shownPanel,"检查文档测试") != nil else { exit(5) }
let takeoverUnavailable = findButton(shownPanel,"接管").map{attr($0,"AXEnabled") as? Bool == false} ?? false
let cancelAvailable = findButton(shownPanel,"取消任务") != nil

guard let card = findButton(shownPanel,"整理桌面测试"),AXUIElementPerformAction(card,"AXPress" as CFString) == .success else { exit(6) }
let detailEnd = ProcessInfo.processInfo.systemUptime + 5
while find(panel,"任务 ID") == nil,visibleWindow() != nil,ProcessInfo.processInfo.systemUptime < detailEnd { Thread.sleep(forTimeInterval:0.2) }
guard visibleWindow() != nil,find(panel,"任务 ID") != nil,find(panel,"整理桌面测试") != nil else { exit(7) }

guard let visible = visibleWindow(),let windowID = visible[kCGWindowNumber as String] as? Int else { exit(8) }
let capture = Process()
capture.executableURL = URL(fileURLWithPath:"/usr/sbin/screencapture")
capture.arguments = ["-x","-l",String(windowID),output.appendingPathComponent("native-agent-panel.png").path]
try capture.run()
capture.waitUntilExit()

let display = CGDisplayBounds(CGMainDisplayID())
let candidates = [CGPoint(x:display.minX + 5,y:display.minY + 30),CGPoint(x:display.maxX - 5,y:display.maxY - 5)]
guard let outside = candidates.first(where:{!rectangle(petInfo).contains($0) && !rectangle(menu).contains($0)}) else { exit(9) }
move(outside)
let hideEnd = ProcessInfo.processInfo.systemUptime + 5
while visibleWindow() != nil,ProcessInfo.processInfo.systemUptime < hideEnd { Thread.sleep(forTimeInterval:0.2) }
guard visibleWindow() == nil else { exit(10) }

let report:[String:Any] = ["pid":Int(app.processIdentifier),"window_id":windowID,"tasks_hover_opens":true,
                           "panel_hover_keeps_open":true,"same_visible_desktop":true,"real_agent_tasks":true,
                           "two_named_tasks_visible":true,"task_id_detail_visible":true,
                           "takeover_disabled_before_execution":takeoverUnavailable,"cancel_available_before_execution":cancelAvailable,
                           "leave_hides":true,"passed":true]
let data = try JSONSerialization.data(withJSONObject:report,options:[.prettyPrinted,.sortedKeys])
try data.write(to:output.appendingPathComponent("result.json"))
print(String(data:data,encoding:.utf8)!)
