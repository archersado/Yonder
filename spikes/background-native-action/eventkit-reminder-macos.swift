import AppKit
import EventKit

func statusName(_ status: EKAuthorizationStatus) -> String {
    switch status {
    case .notDetermined: "not-determined"
    case .restricted: "restricted"
    case .denied: "denied"
    case .fullAccess: "full-access"
    case .writeOnly: "write-only"
    @unknown default: "unknown"
    }
}

@main
struct Probe {
    static func main() async {
        guard CommandLine.arguments.count == 3, ["--authorize", "--background"].contains(CommandLine.arguments[1]) else { exit(2) }
        let authorize = CommandLine.arguments[1] == "--authorize"
        let output = URL(fileURLWithPath: CommandLine.arguments[2])
        let frontmostBefore = NSWorkspace.shared.frontmostApplication?.processIdentifier
        let store = EKEventStore()
        let before = EKEventStore.authorizationStatus(for: .reminder)
        let granted: Bool
        if before == .fullAccess {
            granted = true
        } else if authorize && before == .notDetermined {
            granted = (try? await store.requestFullAccessToReminders()) == true
        } else {
            granted = false
        }

        var saved = false
        var observed = false
        var removed = false
        var failure = granted ? "eventkit-operation-failed" : "permission-unavailable"
        var created: EKReminder?
        if !authorize, granted, let calendar = store.defaultCalendarForNewReminders() {
            do {
                let reminder = EKReminder(eventStore: store)
                reminder.title = "Yonder EventKit Spike"
                reminder.calendar = calendar
                try store.save(reminder, commit: true)
                created = reminder
                saved = true
                observed = store.calendarItem(withIdentifier: reminder.calendarItemIdentifier) is EKReminder
                try store.remove(reminder, commit: true)
                removed = EKEventStore().calendarItem(withIdentifier: reminder.calendarItemIdentifier) == nil
                failure = removed ? "none" : "cleanup-not-confirmed"
            } catch {
                if let created { try? store.remove(created, commit: true) }
                removed = created.map { EKEventStore().calendarItem(withIdentifier: $0.calendarItemIdentifier) == nil } ?? true
            }
        } else if !authorize && granted {
            failure = "default-calendar-unavailable"
        }

        let foregroundUnchanged = frontmostBefore != nil && frontmostBefore == NSWorkspace.shared.frontmostApplication?.processIdentifier
        let passed = authorize ? granted : granted && before == .fullAccess && saved && observed && removed && foregroundUnchanged
        let facts: [[String: Any]] = authorize ? [
            ["sequence": 1, "kind": before == .fullAccess ? "permission-observed" : "permission-required", "result": statusName(before)],
            ["sequence": 2, "kind": "permission-observed", "result": statusName(EKEventStore.authorizationStatus(for: .reminder))]
        ] : granted ? [
            ["sequence": 1, "kind": "step-declared", "execution_class": "background-native", "capability": "eventkit.reminder"],
            ["sequence": 2, "kind": "attempt-started", "execution_class": "background-native", "capability": "eventkit.reminder"],
            ["sequence": 3, "kind": "observe", "result": observed ? "observed" : "unknown"],
            ["sequence": 4, "kind": "attempt-result", "result": passed ? "observed" : "unknown"]
        ] : [
            ["sequence": 1, "kind": "permission-required", "result": statusName(before)]
        ]
        let timelineMappable = facts.enumerated().allSatisfy { $0.element["sequence"] as? Int == $0.offset + 1 }
        let result: [String: Any] = [
            "platform": "macos",
            "native_api": "EventKit",
            "mode": authorize ? "authorize" : "background",
            "authorization_before": statusName(before),
            "authorization_after": statusName(EKEventStore.authorizationStatus(for: .reminder)),
            "permission_granted": granted,
            "saved": saved,
            "observed_by_identifier": observed,
            "cleanup_confirmed": removed,
            "foreground_unchanged": foregroundUnchanged,
            "failure": authorize && granted ? "none" : failure,
            "task_space_facts": facts,
            "timeline_mappable": timelineMappable,
            "passed": passed
        ]
        try! FileManager.default.createDirectory(at: output.deletingLastPathComponent(), withIntermediateDirectories: true)
        let data = try! JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted, .sortedKeys]) + Data([10])
        try! data.write(to: output, options: .atomic)
        FileHandle.standardOutput.write(data)
        exit(passed ? 0 : 1)
    }
}
