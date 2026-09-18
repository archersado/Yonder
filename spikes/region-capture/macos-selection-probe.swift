import AppKit
import CoreGraphics
import Foundation

final class SelectionView: NSView {
    var start: CGPoint?
    var completed: ((CGRect) -> Void)?

    override var acceptsFirstResponder: Bool { true }
    override func mouseDown(with event: NSEvent) { start = convert(event.locationInWindow, from: nil) }
    override func mouseDragged(with event: NSEvent) { needsDisplay = true }
    override func mouseUp(with event: NSEvent) {
        guard let start else { return }
        let end = convert(event.locationInWindow, from: nil)
        completed?(CGRect(x: min(start.x, end.x), y: min(start.y, end.y), width: abs(end.x - start.x), height: abs(end.y - start.y)))
    }
}

@main struct SelectionProbe {
    @MainActor static func main() {
        NSApplication.shared.setActivationPolicy(.accessory)
        let process = getpid()
        let previousPointer = CGEvent(source: nil)?.location
        let screen = NSScreen.main!.visibleFrame
        let size = CGSize(width: 480, height: 320)
        let frame = CGRect(x: screen.midX - size.width / 2, y: screen.midY - size.height / 2, width: size.width, height: size.height)
        let panel = NSPanel(contentRect: frame, styleMask: [.borderless, .nonactivatingPanel], backing: .buffered, defer: false)
        panel.level = .floating
        panel.isOpaque = false
        panel.backgroundColor = NSColor.systemBlue.withAlphaComponent(0.12)
        panel.hasShadow = false
        let view = SelectionView(frame: CGRect(origin: .zero, size: size))
        panel.contentView = view

        var result: [String: Any] = ["passed": false, "error": "timeout"]
        view.completed = { rect in
            panel.orderOut(nil)
            let probeFrontmost = NSWorkspace.shared.frontmostApplication?.processIdentifier == process
            result = [
                "event_path": "cghid-native-synthetic",
                "selection_width": Int(rect.width.rounded()),
                "selection_height": Int(rect.height.rounded()),
                "selection_valid": abs(rect.width - 160) <= 2 && abs(rect.height - 100) <= 2,
                "probe_became_frontmost": probeFrontmost,
                "global_monitor_installed": false,
                "screenshot_attempted": false,
                "overlay_closed": !panel.isVisible,
                "pointer_restored": previousPointer != nil,
                "passed": abs(rect.width - 160) <= 2 && abs(rect.height - 100) <= 2 && !probeFrontmost && !panel.isVisible,
            ]
            if let previousPointer, let move = CGEvent(mouseEventSource: nil, mouseType: .mouseMoved, mouseCursorPosition: previousPointer, mouseButton: .left) {
                move.post(tap: .cghidEventTap)
            }
            NSApp.stop(nil)
        }

        panel.orderFrontRegardless()
        panel.displayIfNeeded()
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.25) {
            guard let info = (CGWindowListCopyWindowInfo([.optionIncludingWindow], CGWindowID(panel.windowNumber)) as? [[String: Any]])?.first,
                  let dictionary = info[kCGWindowBounds as String] as? NSDictionary,
                  let bounds = CGRect(dictionaryRepresentation: dictionary) else {
                result = ["passed": false, "error": "window-bounds-unavailable"]
                panel.orderOut(nil)
                NSApp.stop(nil)
                return
            }
            let source = CGEventSource(stateID: .hidSystemState)
            let start = CGPoint(x: bounds.minX + 80, y: bounds.minY + 70)
            let end = CGPoint(x: start.x + 160, y: start.y + 100)
            CGEvent(mouseEventSource: source, mouseType: .leftMouseDown, mouseCursorPosition: start, mouseButton: .left)?.post(tap: .cghidEventTap)
            CGEvent(mouseEventSource: source, mouseType: .leftMouseDragged, mouseCursorPosition: CGPoint(x: (start.x + end.x) / 2, y: (start.y + end.y) / 2), mouseButton: .left)?.post(tap: .cghidEventTap)
            CGEvent(mouseEventSource: source, mouseType: .leftMouseDragged, mouseCursorPosition: end, mouseButton: .left)?.post(tap: .cghidEventTap)
            CGEvent(mouseEventSource: source, mouseType: .leftMouseUp, mouseCursorPosition: end, mouseButton: .left)?.post(tap: .cghidEventTap)
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + 3) {
            guard panel.isVisible else { return }
            panel.orderOut(nil)
            NSApp.stop(nil)
        }
        NSApp.run()
        panel.close()
        output(result)
    }

    static func output(_ result: [String: Any]) {
        let data = try! JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted, .sortedKeys])
        FileHandle.standardOutput.write(data)
        FileHandle.standardOutput.write(Data("\n".utf8))
        if (result["passed"] as? Bool) != true { exit(2) }
    }
}
