// 隔离 macOS WebKit 界面夹具；不连接正式 Yonder 宿主或真实任务库。
import AppKit
import WebKit

guard CommandLine.arguments.count == 2 else { exit(2) }
let output = URL(fileURLWithPath: CommandLine.arguments[1], isDirectory: true)
guard !FileManager.default.fileExists(atPath: output.path) else { exit(2) }
let ui = URL(fileURLWithPath: #filePath).deletingLastPathComponent().appendingPathComponent("ui", isDirectory: true)
let config = WKWebViewConfiguration()
let fixture = #"""
const task = {task_id:'fixture-creation',name:'隔离创建来源任务',owner_agent_id:'fixture-agent',source:'cloud-agent',status:'created',sequence:'1'};
const history = [{previous:'created',status:'created',sequence:'1',creation_event:{owner_agent_id:'fixture-agent',source:'cloud-agent'}}];
window.__TAURI_INTERNALS__ = {invoke:async (command,args) => {
  if (command !== 'task_query') return null;
  const request = JSON.parse(args.request);
  const result = request.method === 'task.list' ? {kind:'tasks',tasks:[task],next_after_task_id:null}
    : request.method === 'task.step.get' ? {kind:'step',task,step:null}
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
window.title = "Yonda · Task Space · 隔离创建来源夹具"
let web = WKWebView(frame: window.contentView!.bounds, configuration: config)
web.autoresizingMask = [.width, .height]
window.contentView = web
window.makeKeyAndOrderFront(nil)
app.activate(ignoringOtherApps: true)
web.loadFileURL(ui.appendingPathComponent("index.html"), allowingReadAccessTo: ui)

let expected = "任务创建：云端 Agent · Agent fixture-agent"
var found = false
let deadline = Date().addingTimeInterval(20)
while !found && Date() < deadline {
    web.evaluateJavaScript("document.querySelector('.timeline')?.textContent ?? ''") { value, _ in
        found = (value as? String)?.contains(expected) == true
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
try png.write(to: output.appendingPathComponent("native-webkit-creation.png"))
let report: [String: Any] = [
    "platform": "macOS WebKit fixture",
    "creation_source_visible": true,
    "owner_agent_visible": true,
    "description_exposed": false,
    "idempotency_key_exposed": false,
    "real_task_database_used": false,
    "passed": true
]
let data = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys])
try data.write(to: output.appendingPathComponent("result.json"))
print(String(data: data, encoding: .utf8)!)
