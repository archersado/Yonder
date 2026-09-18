// 原生闲置计时验证：只通过既有托盘交互重置计时，不修改前端时钟或任务状态。
import AppKit

guard CommandLine.arguments.count == 4, let pid=pid_t(CommandLine.arguments[1]),
      let app=NSRunningApplication(processIdentifier:pid), app.bundleIdentifier == "com.yonder.e0-spike" else {exit(2)}
let log=URL(fileURLWithPath:CommandLine.arguments[2])
guard !FileManager.default.fileExists(atPath:log.path) else {exit(2)}
FileManager.default.createFile(atPath:log.path,contents:nil)
let handle=try FileHandle(forWritingTo:log)
func emit(_ value:[String:Any]) {
    let data=try! JSONSerialization.data(withJSONObject:value,options:[.sortedKeys])
    handle.write(data); handle.write(Data([10]))
    print(String(data:data,encoding:.utf8)!); fflush(stdout)
}
let start=ProcessInfo.processInfo.systemUptime
let wake=Process();wake.executableURL=URL(fileURLWithPath:CommandLine.arguments[3]);wake.arguments=[String(pid),"wake"]
try wake.run();wake.waitUntilExit()
guard wake.terminationStatus == 0 else {emit(["result":"托盘重置失败"]);exit(3)}
emit(["wake_completed_seconds":ProcessInfo.processInfo.systemUptime-start])
var windowID:Int?
var lastAwake=0.0
var count=0
while ProcessInfo.processInfo.systemUptime-start < 190 {
    let all=CGWindowListCopyWindowInfo([.optionAll,.excludeDesktopElements],kCGNullWindowID) as? [[String:Any]] ?? []
    guard let w=all.first(where:{($0[kCGWindowOwnerPID as String] as? Int)==Int(pid) && ($0[kCGWindowName as String] as? String)=="Yonder E0"}),
          let b=w[kCGWindowBounds as String] as? NSDictionary,let r=CGRect(dictionaryRepresentation:b),let id=w[kCGWindowNumber as String] as? Int else {emit(["result":"窗口不可读"]);exit(4)}
    if windowID == nil {windowID=id}
    guard windowID == id,!app.isHidden else {emit(["result":"窗口身份或可见状态改变"]);exit(4)}
    let elapsed=ProcessInfo.processInfo.systemUptime-start
    if (200...201).contains(r.width) && (200...201).contains(r.height) {
        lastAwake=elapsed
        if count % 80 == 0 {emit(["awake_seconds":elapsed,"window_id":id])}
    } else if (r.width == 56 && r.height == 112) || (r.width == 112 && r.height == 56) {
        let passed=lastAwake >= 179 && elapsed >= 180 && elapsed < 185
        emit(["last_awake_seconds":lastAwake,"first_docked_seconds":elapsed,"window_id":id,"width":r.width,"height":r.height,"passed":passed,"scope":"托盘调用开始至几何收起，250ms轮询；未扣除IPC及缩身动画"])
        exit(passed ? 0 : 5)
    } else {emit(["result":"非预期窗口尺寸","width":r.width,"height":r.height]);exit(4)}
    count += 1;Thread.sleep(forTimeInterval:0.25)
}
emit(["result":"190秒未观察到收起，可能有交互或其他门禁；不判通过"]);exit(6)
