import AppKit

func emit(_ active: Bool) {
    let value: [String: Any] = ["ready": true, "pid": ProcessInfo.processInfo.processIdentifier, "active": active]
    let data = try! JSONSerialization.data(withJSONObject: value)
    FileHandle.standardOutput.write(data + Data([10]))
}

let app = NSApplication.shared
app.setActivationPolicy(.regular)
let window = NSWindow(
    contentRect: NSRect(x: 720, y: 220, width: 320, height: 120),
    styleMask: [.titled, .closable],
    backing: .buffered,
    defer: false
)
window.title = "Yonder Foreground Delivery Decoy"
window.contentView!.addSubview(NSTextField(labelWithString: "decoy"))
window.makeKeyAndOrderFront(nil)
app.activate(ignoringOtherApps: true)
Timer.scheduledTimer(withTimeInterval: 0.05, repeats: true) { _ in emit(app.isActive) }
Timer.scheduledTimer(withTimeInterval: 60, repeats: false) { _ in app.terminate(nil) }
emit(app.isActive)
app.run()
