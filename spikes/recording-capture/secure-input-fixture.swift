import AppKit

let app = NSApplication.shared
app.setActivationPolicy(.regular)
let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 360, height: 150), styleMask: [.titled, .closable], backing: .buffered, defer: false)
window.title = "Yonder 安全输入测试"
let label = NSTextField(labelWithString: "点击下方密码框即可；无需输入任何内容")
label.frame = NSRect(x: 24, y: 90, width: 312, height: 24)
let field = NSSecureTextField(frame: NSRect(x: 24, y: 48, width: 312, height: 28))
field.placeholderString = "测试密码框"
window.contentView?.addSubview(label)
window.contentView?.addSubview(field)
window.center()
window.makeKeyAndOrderFront(nil)
app.activate(ignoringOtherApps: true)
app.run()
