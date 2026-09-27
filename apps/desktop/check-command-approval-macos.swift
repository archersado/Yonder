// 隔离 macOS WebKit 界面夹具；不连接正式 Yonder 宿主、真实任务库或命令执行器。
import AppKit
import WebKit

guard CommandLine.arguments.count == 2 else { exit(2) }
let output = URL(fileURLWithPath: CommandLine.arguments[1], isDirectory: true)
guard !FileManager.default.fileExists(atPath: output.path) else { exit(2) }
let ui = URL(fileURLWithPath: #filePath).deletingLastPathComponent().appendingPathComponent("ui", isDirectory: true)
let config = WKWebViewConfiguration()
let fixture = #"""
const task = {task_id:'fixture-command-approval',name:'命令批准验证任务',owner_agent_id:'agent-fixture',source:'local-agent',status:'created',sequence:'1'};
const command = {
  commandId:'command-native-1',
  state:'awaiting-user',
  expiresAtMs:Date.now() + 600000,
  preview:{
    commandId:'command-native-1',
    program:'/usr/bin/printf',
    args:['%s', 'native approval'],
    cwd:'/tmp/yonder-command-fixture',
    env:{LANG:'zh_CN.UTF-8'},
    timeoutMs:1000,
    expiresAtMs:0
  }
};
command.preview.expiresAtMs = command.expiresAtMs;
window.__commandFixtureCalls = [];
window.__TAURI_INTERNALS__ = {invoke:async (name,args) => {
  if (name === 'file_grant_list') return [];
  if (name === 'command_approval_list') return command.state === 'rejected' ? [] : [{commandId:command.commandId,state:command.state,expiresAtMs:command.expiresAtMs}];
  if (name === 'command_approval_preview') return command.preview;
  if (name === 'command_approval_approve') {
    window.__commandFixtureCalls.push('command_approval_approve');
    command.state = 'approved';
    return {commandId:command.commandId,state:command.state,expiresAtMs:command.expiresAtMs};
  }
  if (name === 'command_approval_reject') {
    window.__commandFixtureCalls.push('command_approval_reject');
    command.state = 'rejected';
    return null;
  }
  if (name !== 'task_query') return null;
  const request = JSON.parse(args.request);
  const result = request.method === 'task.list' ? {kind:'tasks',tasks:[task],next_after_task_id:null}
    : request.method === 'task.step.get' ? {kind:'step',task,step:null}
    : request.method === 'task.events' ? {kind:'events',task_id:task.task_id,events:[]}
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
let window = NSWindow(
    contentRect: NSRect(x: 100, y: 100, width: 1120, height: 860),
    styleMask: [.titled, .closable, .resizable],
    backing: .buffered,
    defer: false
)
window.title = "Yonda · Task Space · 隔离命令批准夹具"
let web = WKWebView(frame: window.contentView!.bounds, configuration: config)
web.autoresizingMask = [.width, .height]
window.contentView = web
window.makeKeyAndOrderFront(nil)
app.activate(ignoringOtherApps: true)
web.loadFileURL(ui.appendingPathComponent("index.html"), allowingReadAccessTo: ui)

func waitFor(_ script: String, timeout: TimeInterval = 20) -> Bool {
    var matched = false
    let deadline = Date().addingTimeInterval(timeout)
    while !matched && Date() < deadline {
        web.evaluateJavaScript(script) { value, _ in matched = value as? Bool == true }
        RunLoop.current.run(until: Date().addingTimeInterval(0.1))
    }
    return matched
}

func evaluate(_ script: String) {
    var finished = false
    web.evaluateJavaScript(script) { _, _ in finished = true }
    let deadline = Date().addingTimeInterval(10)
    while !finished && Date() < deadline { RunLoop.current.run(until: Date().addingTimeInterval(0.1)) }
}

func focusCommandApproval() -> Bool {
    evaluate(#"""
(() => {
const detail = document.querySelector('#detail');
const heading = [...detail.children].find(element => element.tagName === 'H3' && element.textContent === '命令批准');
const note = detail.querySelector('.command-approval-note');
const approvals = detail.querySelector('.command-approvals');
[...detail.children].forEach(element => { element.style.display = [heading, note, approvals].includes(element) ? '' : 'none'; });
detail.style.maxHeight = 'none';
})()
"""#)
    return waitFor(#"""
(() => {
  const detail = document.querySelector('#detail');
  const card = detail?.querySelector('.command-approval-card');
  const description = detail?.querySelector('dl');
  return card != null && description?.style.display === 'none' && detail?.style.maxHeight === 'none';
})()
"""#, timeout: 5)
}

func snapshot(named name: String) throws {
    var image: NSImage?
    web.takeSnapshot(with: nil) { value, _ in image = value }
    let deadline = Date().addingTimeInterval(10)
    while image == nil && Date() < deadline { RunLoop.current.run(until: Date().addingTimeInterval(0.1)) }
    guard let image, let tiff = image.tiffRepresentation, let bitmap = NSBitmapImageRep(data: tiff),
          let png = bitmap.representation(using: .png, properties: [:]) else { throw CocoaError(.fileWriteUnknown) }
    try png.write(to: output.appendingPathComponent(name))
}

let previewReady = waitFor(#"""
(() => {
  const card = document.querySelector('.command-approval-card');
  const text = card?.textContent ?? '';
  return text.includes('等待本机用户决定')
    && text.includes('/usr/bin/printf')
    && text.includes('native approval')
    && text.includes('/tmp/yonder-command-fixture')
    && text.includes('LANG="zh_CN.UTF-8"')
    && text.includes('1000 ms')
    && text.includes('批准执行一次')
    && text.includes('拒绝')
    && (document.querySelector('.command-approval-note')?.textContent ?? '').includes('批准后仅可执行一次');
})()
"""#)
guard previewReady else { exit(3) }
try FileManager.default.createDirectory(at: output, withIntermediateDirectories: true)
guard focusCommandApproval() else { exit(4) }
RunLoop.current.run(until: Date().addingTimeInterval(0.4))
try snapshot(named: "native-webkit-command-awaiting.png")

evaluate(#"""
[...document.querySelectorAll('.command-approval-actions button')].find(button => button.textContent === '批准执行一次')?.click()
"""#)
let approvedReady = waitFor(#"""
(() => {
  const card = document.querySelector('.command-approval-card');
  const text = card?.textContent ?? '';
  return text.includes('已批准，等待归属 Agent 执行')
    && text.includes('撤销批准')
    && window.__commandFixtureCalls.filter(value => value === 'command_approval_approve').length === 1;
})()
"""#)
guard approvedReady else { exit(5) }
RunLoop.current.run(until: Date().addingTimeInterval(0.4))
guard focusCommandApproval() else { exit(6) }
RunLoop.current.run(until: Date().addingTimeInterval(0.4))
try snapshot(named: "native-webkit-command-approved.png")

evaluate(#"""
[...document.querySelectorAll('.command-approval-actions button')].find(button => button.textContent === '撤销批准')?.click()
"""#)
let revokedReady = waitFor(#"""
document.querySelector('.command-approval-empty')?.textContent === '暂无待处理命令'
  && window.__commandFixtureCalls.join(',') === 'command_approval_approve,command_approval_reject'
"""#)
guard revokedReady else { exit(7) }

let report: [String: Any] = [
    "platform": "macOS WKWebView fixture",
    "full_preview_visible": true,
    "risk_notice_visible": true,
    "approve_once_visible": true,
    "approved_state_visible": true,
    "revoke_visible": true,
    "approval_calls": ["command_approval_approve", "command_approval_reject"],
    "real_task_database_used": false,
    "real_command_runtime_used": false,
    "passed": true
]
let data = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys])
try data.write(to: output.appendingPathComponent("result.json"))
print(String(data: data, encoding: .utf8)!)
