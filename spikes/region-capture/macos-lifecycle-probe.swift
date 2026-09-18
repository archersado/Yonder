import AppKit
import Foundation

@main struct SelectionLifecycleProbe {
    @MainActor static func main() async {
        NSApplication.shared.setActivationPolicy(.accessory)
        let foregroundBefore = NSWorkspace.shared.frontmostApplication?.processIdentifier

        let escapePanel = panel()
        var escapeHandled = false
        let monitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { event in
            guard event.keyCode == 53 else { return event }
            escapeHandled = true
            escapePanel.orderOut(nil)
            return nil
        }
        escapePanel.orderFrontRegardless()
        escapePanel.displayIfNeeded()
        let escape = NSEvent.keyEvent(with: .keyDown, location: .zero, modifierFlags: [], timestamp: ProcessInfo.processInfo.systemUptime,
                                      windowNumber: escapePanel.windowNumber, context: nil, characters: "\u{1b}", charactersIgnoringModifiers: "\u{1b}", isARepeat: false, keyCode: 53)!
        NSApp.sendEvent(escape)
        if let monitor { NSEvent.removeMonitor(monitor) }
        let escapeClosed = escapeHandled && !escapePanel.isVisible
        escapePanel.close()

        let timeoutPanel = panel()
        timeoutPanel.orderFrontRegardless()
        timeoutPanel.displayIfNeeded()
        try? await Task.sleep(for: .milliseconds(120))
        timeoutPanel.orderOut(nil)
        let timeoutClosed = !timeoutPanel.isVisible
        timeoutPanel.close()
        try? await Task.sleep(for: .milliseconds(100))

        let foregroundAfter = NSWorkspace.shared.frontmostApplication?.processIdentifier
        let probeNeverFrontmost = foregroundBefore != getpid() && foregroundAfter != getpid()
        let escapeOnScreen = windowOnScreen(escapePanel.windowNumber)
        let timeoutOnScreen = windowOnScreen(timeoutPanel.windowNumber)
        let windowsGone = !escapeOnScreen && !timeoutOnScreen
        let result: [String: Any] = [
            "escape_event": "synthetic-native",
            "escape_handled": escapeHandled,
            "escape_closed": escapeClosed,
            "timeout_closed": timeoutClosed,
            "probe_never_frontmost": probeNeverFrontmost,
            "local_monitor_removed": true,
            "windows_gone": windowsGone,
            "escape_on_screen_after_close": escapeOnScreen,
            "timeout_on_screen_after_close": timeoutOnScreen,
            "global_monitor_installed": false,
            "screenshot_attempted": false,
            "passed": escapeClosed && timeoutClosed && probeNeverFrontmost && windowsGone,
        ]
        let data = try! JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted, .sortedKeys])
        FileHandle.standardOutput.write(data)
        FileHandle.standardOutput.write(Data("\n".utf8))
        exit((result["passed"] as? Bool) == true ? 0 : 2)
    }

    @MainActor static func panel() -> NSPanel {
        let screen = NSScreen.main!.visibleFrame
        let size = CGSize(width: 320, height: 180)
        let frame = CGRect(x: screen.midX - size.width / 2, y: screen.midY - size.height / 2, width: size.width, height: size.height)
        let panel = NSPanel(contentRect: frame, styleMask: [.borderless, .nonactivatingPanel], backing: .buffered, defer: false)
        panel.level = .floating
        panel.isOpaque = false
        panel.backgroundColor = NSColor.black.withAlphaComponent(0.08)
        panel.hasShadow = false
        panel.ignoresMouseEvents = false
        return panel
    }

    static func windowOnScreen(_ number: Int) -> Bool {
        guard number > 0 else { return false }
        let window = (CGWindowListCopyWindowInfo([.optionIncludingWindow], CGWindowID(number)) as? [[String: Any]])?.first
        return (window?[kCGWindowIsOnscreen as String] as? NSNumber)?.boolValue == true
    }
}
