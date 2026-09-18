import AppKit
import Darwin

let fixtureID = "com.yonder.spike.native-fixture"
let marker = URL(fileURLWithPath: NSTemporaryDirectory()).appendingPathComponent("yonder-native-action-\(getuid()).ready")

#if FIXTURE
let fixtureApp = NSApplication.shared
fixtureApp.setActivationPolicy(.accessory)
try? Data("{\"ready\":true}\n".utf8).write(to: marker, options: .atomic)
fixtureApp.run()
exit(0)
#else
let realApp = CommandLine.arguments.count == 4 && CommandLine.arguments[1] == "--real-app"
guard realApp || CommandLine.arguments.count == 3 else { fputs("用法：macos-probe [--real-app] APP OUTPUT\n", stderr); exit(2) }

let appURL = URL(fileURLWithPath: CommandLine.arguments[realApp ? 2 : 1])
let outputURL = URL(fileURLWithPath: CommandLine.arguments[realApp ? 3 : 2])
let workspace = NSWorkspace.shared
let targetID = Bundle(url: appURL)?.bundleIdentifier

func workerPids() -> Set<Int32> {
    let process = Process()
    let pipe = Pipe()
    process.executableURL = URL(fileURLWithPath: "/bin/ps")
    process.arguments = ["-axo", "pid=,command="]
    process.standardOutput = pipe
    try! process.run()
    let text = String(decoding: pipe.fileHandleForReading.readDataToEndOfFile(), as: UTF8.self)
    process.waitUntilExit()
    return Set(text.split(separator: "\n").compactMap { line in
        guard line.contains("cua_worker") || line.contains("trycua") || line.contains("CuaDriver") else { return nil }
        return Int32(line.split(whereSeparator: \.isWhitespace).first ?? "")
    })
}

try? FileManager.default.removeItem(at: marker)
let initiallyRunning = targetID.map(NSRunningApplication.runningApplications(withBundleIdentifier:)) ?? []
if !realApp { for app in initiallyRunning { app.terminate() } }

let beforeFrontmost = workspace.frontmostApplication?.processIdentifier
let beforeWorkers = workerPids()
let configuration = NSWorkspace.OpenConfiguration()
configuration.activates = false
configuration.addsToRecentItems = false

var launched: NSRunningApplication?
var launchFailed = false
var launchDone = false
workspace.openApplication(at: appURL, configuration: configuration) { app, error in
    launched = app
    launchFailed = error != nil
    launchDone = true
}

let deadline = Date().addingTimeInterval(5)
while (!launchDone || (!realApp && !FileManager.default.fileExists(atPath: marker.path))) && Date() < deadline {
    RunLoop.main.run(until: Date().addingTimeInterval(0.02))
}

let ready = realApp
    ? launchDone && launched != nil && launched?.bundleIdentifier == targetID
    : FileManager.default.fileExists(atPath: marker.path)
let afterFrontmost = workspace.frontmostApplication?.processIdentifier
let afterWorkers = workerPids()
let foregroundUnchanged = beforeFrontmost != nil && beforeFrontmost == afterFrontmost
let noNewWorkers = afterWorkers.subtracting(beforeWorkers).isEmpty

let unsupportedBeforeFrontmost = workspace.frontmostApplication?.processIdentifier
let unsupportedBeforeWorkers = workerPids()
let unsupportedClassification = "unsupported"
let unsupportedAfterWorkers = workerPids()
let unsupportedNoFallback = unsupportedClassification == "unsupported"
    && unsupportedBeforeFrontmost == workspace.frontmostApplication?.processIdentifier
    && unsupportedAfterWorkers.subtracting(unsupportedBeforeWorkers).isEmpty

let facts: [[String: Any]] = [
    ["sequence": 1, "kind": "step-declared", "execution_class": "background-native"],
    ["sequence": 2, "kind": "attempt-started", "execution_class": "background-native"],
    ["sequence": 3, "kind": "observe", "result": ready ? "observed" : "unknown"],
    ["sequence": 4, "kind": "attempt-result", "result": ready ? "observed" : "unknown"]
]
let timelineMappable = facts.enumerated().allSatisfy { $0.element["sequence"] as? Int == $0.offset + 1 }
var cleanupForced = false
let cleanupRequired = !realApp || initiallyRunning.isEmpty
if cleanupRequired, let launched {
    launched.terminate()
    let gracefulDeadline = Date().addingTimeInterval(1)
    while !launched.isTerminated && Date() < gracefulDeadline { RunLoop.main.run(until: Date().addingTimeInterval(0.02)) }
    if !launched.isTerminated {
        cleanupForced = true
        launched.forceTerminate()
        let forcedDeadline = Date().addingTimeInterval(2)
        while !launched.isTerminated && Date() < forcedDeadline { RunLoop.main.run(until: Date().addingTimeInterval(0.02)) }
    }
}
let cleanupTerminated = !cleanupRequired || launched?.isTerminated == true
let passed = launchDone && !launchFailed && launched != nil && ready && foregroundUnchanged && noNewWorkers && unsupportedNoFallback && timelineMappable && cleanupTerminated
let result: [String: Any] = [
    "platform": "macos",
    "native_api": "NSWorkspace.OpenConfiguration",
    "target_kind": realApp ? "real-application" : "fixture",
    "target_preexisting": !initiallyRunning.isEmpty,
    "activates": false,
    "launch_completed": launchDone,
    "launch_failed": launchFailed,
    "observe_ready": ready,
    "foreground_unchanged": foregroundUnchanged,
    "new_trycua_workers": afterWorkers.subtracting(beforeWorkers).count,
    "unsupported_classification": unsupportedClassification,
    "unsupported_no_fallback": unsupportedNoFallback,
    "cleanup_forced": cleanupForced,
    "cleanup_terminated": cleanupTerminated,
    "cleanup_skipped_existing": !cleanupRequired,
    "task_space_facts": facts,
    "timeline_mappable": timelineMappable,
    "passed": passed
]

try? FileManager.default.removeItem(at: marker)
try! FileManager.default.createDirectory(at: outputURL.deletingLastPathComponent(), withIntermediateDirectories: true)
let data = try! JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted, .sortedKeys])
try! (data + Data([10])).write(to: outputURL, options: .atomic)
FileHandle.standardOutput.write(data + Data([10]))
exit(passed ? 0 : 1)
#endif
