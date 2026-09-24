import AppKit
import Foundation

private final class FixtureDelegate: NSObject, NSApplicationDelegate {
    private var windows: [NSWindow] = []

    func applicationDidFinishLaunching(_ notification: Notification) {
        createWindow(offset: 0)
        NSApplication.shared.activate(ignoringOtherApps: true)
        Timer.scheduledTimer(withTimeInterval: 2, repeats: false) { [weak self] _ in
            self?.createWindow(offset: 40)
        }
        Timer.scheduledTimer(withTimeInterval: 3, repeats: false) { [weak self] _ in
            self?.windows.first?.makeKeyAndOrderFront(nil)
        }
        Timer.scheduledTimer(withTimeInterval: 4, repeats: false) { [weak self] _ in
            self?.windows.last?.makeKeyAndOrderFront(nil)
        }
        Timer.scheduledTimer(withTimeInterval: 6, repeats: false) { _ in
            NSApplication.shared.terminate(nil)
        }
    }

    private func createWindow(offset: CGFloat) {
        let window = NSWindow(
            contentRect: NSRect(x: 160 + offset, y: 160 + offset, width: 320, height: 180),
            styleMask: [.titled, .closable],
            backing: .buffered,
            defer: false
        )
        window.title = "Yonder CX-S1 Fixture"
        window.makeKeyAndOrderFront(nil)
        windows.append(window)
    }
}

private let application = NSApplication.shared
private let delegate = FixtureDelegate()
application.delegate = delegate
application.setActivationPolicy(.regular)
application.run()
