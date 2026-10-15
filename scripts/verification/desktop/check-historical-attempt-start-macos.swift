// 隔离 macOS WebKit 界面夹具；不连接正式 Yonder 宿主或真实任务库。
import AppKit
import WebKit

guard CommandLine.arguments.count == 2 else { exit(2) }
let output = URL(fileURLWithPath: CommandLine.arguments[1], isDirectory: true)
guard !FileManager.default.fileExists(atPath: output.path) else { exit(2) }
let ui = URL(fileURLWithPath: #filePath).deletingLastPathComponent().appendingPathComponent("ui", isDirectory: true)
let config = WKWebViewConfiguration()
let fixture = #"""
const task = {task_id:'fixture-attempt-start',name:'隔离尝试开始任务',owner_agent_id:'fixture-agent',source:'local-agent',status:'running',sequence:'4'};
const history = [
  {previous:'created',status:'created',sequence:'1'},
  {previous:'created',status:'created',sequence:'2',step_declaration:{step_id:'step-one',label:'打开文档',accepted_sequence:'2'}},
  {previous:'created',status:'running',sequence:'3',attempt_started:{step_id:'step-one',attempt_id:'attempt-one',worker_instance_id:'worker-secret',host_session_id:'host-secret'}},
  {previous:'running',status:'running',sequence:'4',attempt_result:{step_id:'step-one',attempt_id:'attempt-one',worker_instance_id:'worker-secret',host_session_id:'host-secret',phase:'observed',action_succeeded:true,observe_valid:true}}
];
window.__TAURI_INTERNALS__ = {invoke:async (command,args) => {
  if (command !== 'task_query') return null;
  const request = JSON.parse(args.request);
  const result = request.method === 'task.list' ? {kind:'tasks',tasks:[task],next_after_task_id:null}
    : request.method === 'task.step.get' ? {kind:'step',task,step:{step_id:'step-one',label:'打开文档',accepted_sequence:'2'}}
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
window.title = "Yonda · Task Space · 隔离尝试开始夹具"
let web = WKWebView(frame: window.contentView!.bounds, configuration: config)
web.autoresizingMask = [.width, .height]
window.contentView = web
window.makeKeyAndOrderFront(nil)
app.activate(ignoringOtherApps: true)
web.loadFileURL(ui.appendingPathComponent("index.html"), allowingReadAccessTo: ui)

let expected = "执行尝试已准备（步骤 step-one · 尝试 attempt-one）"
var text = ""
let deadline = Date().addingTimeInterval(20)
while !text.contains(expected) && Date() < deadline {
    web.evaluateJavaScript("document.querySelector('.timeline')?.textContent ?? ''") { value, _ in text = value as? String ?? "" }
    RunLoop.current.run(until: Date().addingTimeInterval(0.1))
}
guard text.contains(expected), text.contains("动作已观察：成功"), !text.contains("worker-secret"), !text.contains("host-secret") else { exit(3) }
var image: NSImage?
web.takeSnapshot(with: nil) { snapshot, _ in image = snapshot }
let snapshotDeadline = Date().addingTimeInterval(10)
while image == nil && Date() < snapshotDeadline { RunLoop.current.run(until: Date().addingTimeInterval(0.1)) }
guard let image, let tiff = image.tiffRepresentation, let bitmap = NSBitmapImageRep(data: tiff),
      let png = bitmap.representation(using: .png, properties: [:]) else { exit(4) }
try FileManager.default.createDirectory(at: output, withIntermediateDirectories: true)
try png.write(to: output.appendingPathComponent("native-webkit-attempt-start.png"))
let report: [String: Any] = ["platform": "macOS WebKit fixture", "attempt_start_visible": true,
    "result_distinct": true, "worker_host_hidden": true, "real_task_database_used": false, "passed": true]
let data = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys])
try data.write(to: output.appendingPathComponent("result.json"))
print(String(data: data, encoding: .utf8)!)
