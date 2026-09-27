// EX-S2 专用隔离窗口：单一可访问窗口，避免通用焦点夹具的歧义/替换分支。
import AppKit

func emit(_ value: [String: Any]) {
  let data = try! JSONSerialization.data(withJSONObject: value)
  FileHandle.standardOutput.write(data + Data([10]))
}

final class FixtureField: NSView {
  var value = ""
  let label = NSTextField(labelWithString: "")
  override init(frame: NSRect) {
    super.init(frame: frame)
    setAccessibilityElement(true)
    setAccessibilityRole(.textField)
    setAccessibilityEnabled(true)
    setAccessibilityIdentifier("yonder-ex-s2-plan-input")
    label.frame = bounds
    addSubview(label)
  }
  required init?(coder: NSCoder) { fatalError("仅程序构造") }
  override func isAccessibilityElement() -> Bool { true }
  override func accessibilityRole() -> NSAccessibility.Role? { .textField }
  override func accessibilityLabel() -> String? { "Yonder EX-S2 隔离输入" }
  override func accessibilityValue() -> Any? { value }
  override func accessibilityChildren() -> [Any]? { [] }
  override func isAccessibilityEnabled() -> Bool { true }
  override func isAccessibilitySelectorAllowed(_ selector: Selector) -> Bool { true }
  override func setAccessibilityValue(_ input: Any?) {
    guard let text = input as? String, text == "YONDER_SDK_INPUT_A" else { return }
    value = text
    label.stringValue = text
    emit(["input_matches": true])
  }
}

let app = NSApplication.shared
app.setActivationPolicy(.regular)
let processID = ProcessInfo.processInfo.processIdentifier
let window = NSWindow(
  contentRect: NSRect(x: 160, y: 200, width: 440, height: 140),
  styleMask: [.titled, .closable, .miniaturizable],
  backing: .buffered,
  defer: false
)
window.title = "Yonder EX-S2 Fixture \(processID)"
window.isReleasedWhenClosed = false
let field = FixtureField(frame: NSRect(x: 24, y: 50, width: 390, height: 32))
window.contentView!.addSubview(field)
window.makeKeyAndOrderFront(nil)

var launched = false
final class Delegate: NSObject, NSApplicationDelegate {
  func applicationDidFinishLaunching(_: Notification) {
    launched = true
    app.activate(ignoringOtherApps: true)
  }
}
let delegate = Delegate()
app.delegate = delegate

func report() {
  emit([
    "ready": true,
    "launched": launched,
    "pid": processID,
    "window_id": window.windowNumber,
    "target_key": window.isKeyWindow,
    "target_on_active_space": window.isOnActiveSpace,
    "input_matches": field.value == "YONDER_SDK_INPUT_A",
  ])
}

report()
Timer.scheduledTimer(withTimeInterval: 0.1, repeats: true) { _ in report() }
Timer.scheduledTimer(withTimeInterval: 90, repeats: false) { _ in app.terminate(nil) }
app.run()
