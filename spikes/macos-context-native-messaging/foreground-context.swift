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

    func replaceObserver(for processIdentifier: pid_t) {
        releaseObserver()
        lock.lock()
        observerAttempts += 1
        lock.unlock()
        var created: AXObserver?
        let result = AXObserverCreate(processIdentifier, { _, _, _, context in
            guard let context else { return }
            Unmanaged<ProbeState>.fromOpaque(context).takeUnretainedValue().recordWindowEvent()
        }, &created)
        guard result == .success, let created else { return }
        let application = AXUIElementCreateApplication(processIdentifier)
        let context = Unmanaged.passUnretained(self).toOpaque()
        guard AXObserverAddNotification(
            created,
            application,
            kAXFocusedWindowChangedNotification as CFString,
            context
        ) == .success else { return }
        _ = AXObserverAddNotification(
            created,
            application,
            kAXWindowCreatedNotification as CFString,
            context
        )
        CFRunLoopAddSource(CFRunLoopGetCurrent(), AXObserverGetRunLoopSource(created), .defaultMode)
        lock.lock()
        observer = created
        observerRegistrations += 1
        observerReleased = false
        lock.unlock()
    }

    func releaseObserver() {
        lock.lock()
        let current = observer
        observer = nil
        observerReleased = true
        lock.unlock()
        if let current {
            CFRunLoopRemoveSource(CFRunLoopGetCurrent(), AXObserverGetRunLoopSource(current), .defaultMode)
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
if trusted, let frontmost {
    state.replaceObserver(for: frontmost.processIdentifier)
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
    state.replaceObserver(for: application.processIdentifier)
}

let fixturePaths = arguments.indices.compactMap { index -> String? in
    guard arguments[index] == "--fixture-app", arguments.indices.contains(index + 1) else { return nil }
    return arguments[index + 1]
}
for (index, fixturePath) in fixturePaths.enumerated() {
    let fixtureURL = URL(fileURLWithPath: fixturePath)
    Timer.scheduledTimer(withTimeInterval: 1 + Double(index * 2), repeats: false) { _ in
        if let identifier = Bundle(url: fixtureURL)?.bundleIdentifier,
           let running = NSRunningApplication.runningApplications(withBundleIdentifier: identifier).first {
            running.activate(options: [.activateAllWindows])
            return
        }
        let configuration = NSWorkspace.OpenConfiguration()
        configuration.activates = true
        NSWorkspace.shared.openApplication(at: fixtureURL, configuration: configuration) { _, _ in }
    }
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
