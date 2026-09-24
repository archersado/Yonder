import AppKit
import ApplicationServices
import Foundation

private final class ProbeState: @unchecked Sendable {
    private let lock = NSLock()
    private(set) var activationEvents = 0
    private(set) var windowEvents = 0
    private(set) var observerAttempts = 0
    private(set) var observerRegistrations = 0
    private(set) var observerReleased = true
    private var observer: AXObserver?

    func recordWindowEvent() {
        lock.lock()
        windowEvents += 1
        lock.unlock()
    }

    func recordActivation() {
        lock.lock()
        activationEvents += 1
        lock.unlock()
    }

    @discardableResult
    func replaceObserver(for processIdentifier: pid_t) -> Bool {
        releaseObserver()
        lock.lock()
        observerAttempts += 1
        lock.unlock()
        var created: AXObserver?
        let result = AXObserverCreate(processIdentifier, { _, _, _, context in
            guard let context else { return }
            Unmanaged<ProbeState>.fromOpaque(context).takeUnretainedValue().recordWindowEvent()
        }, &created)
        guard result == .success, let created else { return false }
        let application = AXUIElementCreateApplication(processIdentifier)
        let context = Unmanaged.passUnretained(self).toOpaque()
        let focusedWindowResult = AXObserverAddNotification(
            created,
            application,
            kAXFocusedWindowChangedNotification as CFString,
            context
        )
        let windowCreatedResult = AXObserverAddNotification(
            created,
            application,
            kAXWindowCreatedNotification as CFString,
            context
        )
        guard focusedWindowResult == .success || windowCreatedResult == .success else { return false }
        CFRunLoopAddSource(CFRunLoopGetCurrent(), AXObserverGetRunLoopSource(created), .commonModes)
        lock.lock()
        observer = created
        observerRegistrations += 1
        observerReleased = false
        lock.unlock()
        return true
    }

    func releaseObserver() {
        lock.lock()
        let current = observer
        observer = nil
        observerReleased = true
        lock.unlock()
        if let current {
            CFRunLoopRemoveSource(CFRunLoopGetCurrent(), AXObserverGetRunLoopSource(current), .commonModes)
        }
    }

    func snapshot() -> (Int, Int, Int, Int, Bool) {
        lock.lock()
        defer { lock.unlock() }
        return (activationEvents, windowEvents, observerAttempts, observerRegistrations, observerReleased)
    }
}

private func output(_ value: [String: Any]) -> Never {
    let data = try! JSONSerialization.data(withJSONObject: value, options: [.sortedKeys])
    FileHandle.standardOutput.write(data)
    FileHandle.standardOutput.write(Data([0x0a]))
    exit(0)
}

private let arguments = Array(CommandLine.arguments.dropFirst())
private let mode = arguments.first ?? "--check"
private let probeApplication = NSApplication.shared
probeApplication.setActivationPolicy(.prohibited)
private let trusted = AXIsProcessTrusted()
private let frontmost = NSWorkspace.shared.frontmostApplication
private let base: [String: Any] = [
    "platform": "macos",
    "workspace_available": true,
    "accessibility_trusted": trusted,
    "frontmost_bundle_id_present": frontmost?.bundleIdentifier?.isEmpty == false,
    "frontmost_pid_valid": (frontmost?.processIdentifier ?? 0) > 0,
]

if mode == "--check" {
    var result = base
    result["status"] = trusted ? "ready" : "capability_unavailable"
    output(result)
}

guard mode == "--observe" else {
    FileHandle.standardError.write(Data("usage_error\n".utf8))
    exit(2)
}
let seconds = arguments.dropFirst().first.flatMap(Double.init) ?? 8
guard seconds > 0, seconds <= 30 else {
    FileHandle.standardError.write(Data("duration_invalid\n".utf8))
    exit(2)
}

private let state = ProbeState()
private func bindObserver(for processIdentifier: pid_t) {
    guard !state.replaceObserver(for: processIdentifier) else { return }
    Timer.scheduledTimer(withTimeInterval: 0.5, repeats: false) { _ in
        guard NSWorkspace.shared.frontmostApplication?.processIdentifier == processIdentifier else { return }
        state.replaceObserver(for: processIdentifier)
    }
}
if trusted, let frontmost {
    bindObserver(for: frontmost.processIdentifier)
}
let notificationCenter = NSWorkspace.shared.notificationCenter
let token = notificationCenter.addObserver(
    forName: NSWorkspace.didActivateApplicationNotification,
    object: nil,
    queue: nil
) { notification in
    state.recordActivation()
    guard trusted,
          let application = notification.userInfo?[NSWorkspace.applicationUserInfoKey]
            as? NSRunningApplication else { return }
    bindObserver(for: application.processIdentifier)
}

RunLoop.current.run(until: Date().addingTimeInterval(seconds))
notificationCenter.removeObserver(token)
state.releaseObserver()
let snapshot = state.snapshot()
var result = base
result["status"] = trusted ? "stopped" : "capability_unavailable"
result["activation_events"] = snapshot.0
result["window_events"] = snapshot.1
result["observer_attempts"] = snapshot.2
result["observer_registrations"] = snapshot.3
result["observer_released"] = snapshot.4
result["workspace_notification_removed"] = true
output(result)
