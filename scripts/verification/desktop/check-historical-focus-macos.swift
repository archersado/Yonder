// 隔离 macOS WebKit 界面夹具；不连接正式 Yonder 宿主或真实任务库。
import AppKit
import WebKit

guard CommandLine.arguments.count == 2 else { exit(2) }
let output = URL(fileURLWithPath: CommandLine.arguments[1], isDirectory: true)
guard !FileManager.default.fileExists(atPath: output.path) else { exit(2) }
let ui = URL(fileURLWithPath: #filePath).deletingLastPathComponent().appendingPathComponent("ui", isDirectory: true)
let config = WKWebViewConfiguration()
let fixture = #"""
const task = {task_id:'fixture-focus',name:'隔离定位失败任务',owner_agent_id:'a1',source:'local-agent',status:'paused',sequence:'8',current_step:{step_id:'step-one',label:'验证接管定位',accepted_sequence:'2'}};
const history = [
  {previous:'paused',status:'paused',sequence:'7',focus_event:{control_id:'control_3',phase:'locating'}},
  {previous:'paused',status:'paused',sequence:'8',focus_event:{control_id:'control_3',phase:'failed',failure:'permission-unavailable'}}
];
window.__TAURI_INTERNALS__ = {invoke:async (command,args) => {
  if (command !== 'task_query') return null;
  const request = JSON.parse(args.request);
  const result = request.method === 'task.list' ? {kind:'tasks',tasks:[task],next_after_task_id:null}
    : request.method === 'task.step.get' ? {kind:'step',task,step:task.current_step}
    : request.method === 'task.events' ? {kind:'events',task_id:task.task_id,events:history.filter(item=>BigInt(item.sequence)>BigInt(request.params.after_sequence))}
    : {kind:'browser-state',task,reference:null};
  return JSON.stringify({jsonrpc:'2.0',id:request.id,result});
}};
window.addEventListener('load', () => {
  const timer = setInterval(() => { const button = document.querySelector('.task'); if (button) { clearInterval(timer); button.click(); } }, 50);
});
"""#
config.userContentController.addUserScript(WKUserScript(source: fixture, injectionTime: .atDocumentStart, forMainFrameOnly: true))
let app = NSApplication.shared
app.setActivationPolicy(.regular)
let window = NSWindow(contentRect: NSRect(x: 100, y: 100, width: 1060, height: 760), styleMask: [.titled, .closable], backing: .buffered, defer: false)
window.title = "Yonda · Task Space · 隔离定位夹具"
let web = WKWebView(frame: window.contentView!.bounds, configuration: config)
web.autoresizingMask = [.width, .height]
window.contentView = web
window.makeKeyAndOrderFront(nil)
app.activate(ignoringOtherApps: true)
web.loadFileURL(ui.appendingPathComponent("index.html"), allowingReadAccessTo: ui)

var found = false
let deadline = Date().addingTimeInterval(20)
while !found && Date() < deadline {
    web.evaluateJavaScript("document.querySelector('.timeline')?.textContent ?? ''") { value, _ in
        guard let text = value as? String else { return }
        found = text.contains("接管：正在定位任务工作") && text.contains("接管：定位失败 · 缺少辅助功能权限")
    }
    RunLoop.current.run(until: Date().addingTimeInterval(0.1))
}
guard found else { exit(3) }
web.evaluateJavaScript("document.querySelector('.timeline')?.scrollIntoView({block:'center'})")
RunLoop.current.run(until: Date().addingTimeInterval(0.4))
var image: NSImage?
web.takeSnapshot(with: nil) { snapshot, _ in image = snapshot }
let snapshotDeadline = Date().addingTimeInterval(10)
while image == nil && Date() < snapshotDeadline { RunLoop.current.run(until: Date().addingTimeInterval(0.1)) }
guard let image, let tiff = image.tiffRepresentation, let bitmap = NSBitmapImageRep(data: tiff),
      let png = bitmap.representation(using: .png, properties: [:]) else { exit(4) }
try FileManager.default.createDirectory(at: output, withIntermediateDirectories: true)
try png.write(to: output.appendingPathComponent("native-webkit-focus.png"))
let report: [String: Any] = [
    "platform": "macOS WebKit fixture",
    "focus_locating_visible": true,
    "focus_failure_reason_visible": true,
    "real_task_database_used": false,
    "passed": true
]
let data = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys])
try data.write(to: output.appendingPathComponent("result.json"))
print(String(data: data, encoding: .utf8)!)
