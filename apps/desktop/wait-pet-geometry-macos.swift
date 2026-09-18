// 等待正式 Yonda 小龙进入指定几何并保存窗口截图；不触发窗口或任务操作。
import AppKit

guard CommandLine.arguments.count == 5, let pid = pid_t(CommandLine.arguments[1]),
      ["docked", "awake"].contains(CommandLine.arguments[2]) else { exit(2) }
let mode = CommandLine.arguments[2], report = URL(fileURLWithPath: CommandLine.arguments[3]), captureURL = URL(fileURLWithPath: CommandLine.arguments[4])
let deadline = ProcessInfo.processInfo.systemUptime + (mode == "docked" ? 190 : 10)
while ProcessInfo.processInfo.systemUptime < deadline {
    let windows = CGWindowListCopyWindowInfo([.optionAll, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
    if let window = windows.first(where: { ($0[kCGWindowOwnerPID as String] as? Int) == Int(pid) && ($0[kCGWindowName as String] as? String) == "Yonda" }),
       let id = window[kCGWindowNumber as String] as? Int,
       let values = window[kCGWindowBounds as String] as? NSDictionary,
       let bounds = CGRect(dictionaryRepresentation: values) {
        let matched = mode == "docked" ? ((bounds.width == 56 && bounds.height == 112) || (bounds.width == 112 && bounds.height == 56)) : ((200...201).contains(bounds.width) && (200...201).contains(bounds.height))
        if matched {
            let shot = Process(); shot.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture"); shot.arguments = ["-x", "-l", String(id), captureURL.path]
            try shot.run(); shot.waitUntilExit(); guard shot.terminationStatus == 0 else { exit(4) }
            let result: [String: Any] = ["mode":mode,"pid":Int(pid),"window_id":id,"width":bounds.width,"height":bounds.height,"x":bounds.origin.x,"y":bounds.origin.y,"passed":true]
            try JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted, .sortedKeys]).write(to: report)
            print(String(data: try JSONSerialization.data(withJSONObject: result, options: [.sortedKeys]), encoding: .utf8)!)
            exit(0)
        }
    }
    Thread.sleep(forTimeInterval: 0.25)
}
exit(3)
