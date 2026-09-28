// 隔离 macOS WebKit 界面夹具：验证当前 running 任务步骤在形象上显示并及时清理。
import AppKit
import WebKit

guard CommandLine.arguments.count == 2 else { exit(2) }
let output = URL(fileURLWithPath: CommandLine.arguments[1], isDirectory: true)
guard !FileManager.default.fileExists(atPath: output.path) else { exit(2) }
let ui = URL(fileURLWithPath: #filePath).deletingLastPathComponent().appendingPathComponent("ui", isDirectory: true)
let config = WKWebViewConfiguration()
let fixture = #"""
window.__TAURI_INTERNALS__ = {invoke:async command => {
  if (command === 'pet_is_visible') return true;
  if (command === 'pet_task_state') return [true, 'executing', '打开企业微信'];
  if (command === 'pet_agent_connected') return true;
  if (command === 'task_menu_show') return true;
  if (command === 'pet_dock') return 'bottom';
  throw new Error('fixture-unavailable');
}};
"""#
config.userContentController.addUserScript(WKUserScript(source: fixture, injectionTime: .atDocumentStart, forMainFrameOnly: true))
let app = NSApplication.shared
app.setActivationPolicy(.regular)
let window = NSWindow(contentRect: NSRect(x: 100, y: 100, width: 200, height: 200), styleMask: [.titled, .closable], backing: .buffered, defer: false)
window.title = "Yonda · 当前步骤隔离夹具"
let web = WKWebView(frame: window.contentView!.bounds, configuration: config)
web.autoresizingMask = [.width, .height]
window.contentView = web
window.makeKeyAndOrderFront(nil)
app.activate(ignoringOtherApps: true)
web.loadFileURL(ui.appendingPathComponent("pet.html"), allowingReadAccessTo: ui)

func evaluate(_ script: String, until predicate: @escaping ([String: Any]) -> Bool) -> [String: Any]? {
    var matched: [String: Any]?
    let deadline = Date().addingTimeInterval(20)
    while matched == nil && Date() < deadline {
        web.evaluateJavaScript(script) { value, _ in
            if let result = value as? [String: Any], predicate(result) { matched = result }
        }
        RunLoop.current.run(until: Date().addingTimeInterval(0.1))
    }
    return matched
}

let stateScript = "({text:document.getElementById('task-step-status')?.textContent ?? '',display:getComputedStyle(document.getElementById('task-step-status')).display,hasStep:document.getElementById('pet')?.dataset.hasStep ?? '',aria:document.getElementById('pet')?.getAttribute('aria-label') ?? ''})"
guard let running = evaluate(stateScript, until: { result in
    (result["text"] as? String) == "正在：打开企业微信"
        && (result["display"] as? String) == "block"
        && (result["aria"] as? String)?.contains("当前步骤：打开企业微信") == true
}) else { exit(3) }

var image: NSImage?
web.takeSnapshot(with: nil) { snapshot, _ in image = snapshot }
let snapshotDeadline = Date().addingTimeInterval(10)
while image == nil && Date() < snapshotDeadline { RunLoop.current.run(until: Date().addingTimeInterval(0.1)) }
guard let image, let tiff = image.tiffRepresentation, let bitmap = NSBitmapImageRep(data: tiff),
      let png = bitmap.representation(using: .png, properties: [:]) else { exit(4) }

web.evaluateJavaScript("window.dispatchEvent(new CustomEvent('yonda-presentation',{detail:{hasTasks:true,state:'waiting_for_user',stepLabel:'不得残留'}}))")
guard let cleared = evaluate(stateScript, until: { result in
    (result["text"] as? String) == ""
        && (result["display"] as? String) == "none"
        && (result["aria"] as? String)?.contains("当前步骤") == false
}) else { exit(5) }

try FileManager.default.createDirectory(at: output, withIntermediateDirectories: true)
try png.write(to: output.appendingPathComponent("native-webkit-current-step.png"))
let report: [String: Any] = [
    "platform": "macOS WebKit fixture",
    "running": running,
    "cleared_after_running": cleared,
    "task_store_mutated": false,
    "passed": true
]
let data = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys])
try data.write(to: output.appendingPathComponent("result.json"))
print(String(data: data, encoding: .utf8)!)
