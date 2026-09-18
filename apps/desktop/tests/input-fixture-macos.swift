// 临时原生测试窗；不是产品App，只输出固定样本匹配布尔与长度。
import AppKit
func event(_ value: [String: Any]) {
 let data = try! JSONSerialization.data(withJSONObject: value)
 FileHandle.standardOutput.write(data + Data([10]))
}
class TestField: NSTextField {
 override var stringValue: String {
 get { super.stringValue }
 set {
  if CommandLine.arguments.contains("--delay-ax"), newValue == "YONDER_SDK_INPUT_A" {
   event(["native_write_started": true, "time_ms": Date().timeIntervalSince1970 * 1000])
   Thread.sleep(forTimeInterval: 2)
   super.stringValue = newValue
   event(["native_write_applied": stringValue == "YONDER_SDK_INPUT_A", "time_ms": Date().timeIntervalSince1970 * 1000])
  } else { super.stringValue = newValue }
 }
 }
}
// 显式实现原生AX写入口；内置NSTextField的内部AX桥可能绕过公开setter。
class DelayedAXField: NSView {
 var value = ""
 let label = NSTextField(labelWithString: "")
 override init(frame: NSRect) {
  super.init(frame: frame)
  setAccessibilityElement(true)
  setAccessibilityRole(.textField)
  setAccessibilityEnabled(true)
  setAccessibilityIdentifier("yonder-sdk-test-field")
  label.frame = bounds
  addSubview(label)
 }
 required init?(coder: NSCoder) { fatalError("仅程序构造测试控件") }
 override func isAccessibilityElement() -> Bool { true }
 override func accessibilityRole() -> NSAccessibility.Role? { .textField }
 override func accessibilityLabel() -> String? { "Yonder SDK隔离输入" }
 override func accessibilityValue() -> Any? { value }
 override func accessibilityChildren() -> [Any]? { [] }
 override func isAccessibilityEnabled() -> Bool { true }
 override func isAccessibilitySelectorAllowed(_ selector: Selector) -> Bool { true }
 override func setAccessibilityValue(_ input: Any?) {
  guard let text = input as? String, text == "YONDER_SDK_INPUT_A" else { return }
  event(["native_write_started": true, "time_ms": Date().timeIntervalSince1970 * 1000])
  Thread.sleep(forTimeInterval: 2)
  value = text
  label.stringValue = text
  event(["native_write_applied": true, "time_ms": Date().timeIntervalSince1970 * 1000])
 }
}
let app = NSApplication.shared
app.setActivationPolicy(.regular)
let window = NSWindow(contentRect: NSRect(x: 160, y: 200, width: 440, height: 140), styleMask: [.titled, .closable, .miniaturizable], backing: .buffered, defer: false)
window.title = "Yonder SDK隔离测试"
window.isReleasedWhenClosed = false
let field = TestField(frame: NSRect(x: 24, y: 50, width: 390, height: 32))
field.setAccessibilityLabel("Yonder SDK隔离输入")
field.setAccessibilityIdentifier("yonder-sdk-test-field")
let delayed = CommandLine.arguments.contains("--delay-ax") ? DelayedAXField(frame: field.frame) : nil
window.contentView!.addSubview(delayed ?? field)
window.makeKeyAndOrderFront(nil)
app.activate(ignoringOtherApps: true)
let focusFixture = CommandLine.arguments.contains("--focus-fixture")
let decoy: NSWindow? = focusFixture ? NSWindow(contentRect:NSRect(x:680,y:200,width:440,height:140),styleMask:[.titled,.closable],backing:.buffered,defer:false) : nil
if let decoy { decoy.title=window.title; decoy.makeKeyAndOrderFront(nil) }
var launched = false
var targetClosed = false
var replacement: NSWindow?
class LaunchDelegate: NSObject, NSApplicationDelegate {
 func applicationDidFinishLaunching(_ notification:Notification) {
 launched=true
  if focusFixture { decoy?.makeKeyAndOrderFront(nil); app.activate(ignoringOtherApps:true) }
 }
}
let launchDelegate=LaunchDelegate()
app.delegate=launchDelegate
func report() {
 let value = delayed?.value ?? field.stringValue
 let state: [String: Any] = ["ready": true, "pid": ProcessInfo.processInfo.processIdentifier, "window_id": window.windowNumber, "length": value.utf8.count, "matches": value == "YONDER_SDK_INPUT_A", "target_key": window.isKeyWindow, "target_minimized": window.isMiniaturized, "target_on_active_space": window.isOnActiveSpace, "app_active": app.isActive, "decoy_key": decoy?.isKeyWindow ?? false, "decoy_visible": decoy?.isVisible ?? false, "launched": launched, "target_closed": targetClosed, "replacement_window_id": replacement?.windowNumber ?? 0, "replacement_visible": replacement?.isVisible ?? false, "replacement_key": replacement?.isKeyWindow ?? false, "overlapping": decoy.map { $0.frame == window.frame } ?? false]
 let data = try! JSONSerialization.data(withJSONObject: state)
 FileHandle.standardOutput.write(data + Data([10]))
}
if focusFixture {
 FileHandle.standardInput.readabilityHandler = { handle in
  let data=handle.availableData
  if data.isEmpty { handle.readabilityHandler=nil; return }
  let commands=String(data:data,encoding:.utf8)?.split(separator:"\n") ?? []
  DispatchQueue.main.async {
   for command in commands {
    if command == "minimize" { window.miniaturize(nil); decoy?.makeKeyAndOrderFront(nil) }
    if command == "close" { window.close(); targetClosed=true }
    if command == "background" { decoy?.makeKeyAndOrderFront(nil) }
    if command == "overlap" { decoy?.setFrame(window.frame, display:true); decoy?.makeKeyAndOrderFront(nil) }
    if command == "separate" { decoy?.setFrameOrigin(NSPoint(x:680,y:200)) }
    if command == "replace" {
     let rect=window.frame
     window.close(); targetClosed=true
     replacement=NSWindow(contentRect:NSRect(x:160,y:200,width:440,height:140),styleMask:[.titled,.closable,.miniaturizable],backing:.buffered,defer:false)
     replacement!.title=window.title
     replacement!.isReleasedWhenClosed=false
     replacement!.contentView!.addSubview(NSTextField(frame:field.frame))
     replacement!.setFrame(rect,display:true); replacement!.makeKeyAndOrderFront(nil)
    }
   }
   report()
  }
 }
}
report()
let timer = Timer.scheduledTimer(withTimeInterval: 0.1, repeats: true) { _ in report() }
let deadline = Timer.scheduledTimer(withTimeInterval: 90, repeats: false) { _ in app.terminate(nil) }
app.run()
