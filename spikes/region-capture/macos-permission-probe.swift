import CoreGraphics
import Foundation

let authorized = CGPreflightScreenCaptureAccess()
let result: [String: Any] = [
    "bundle_id": Bundle.main.bundleIdentifier ?? "unavailable",
    "preflight_authorized": authorized,
    "capture_attempted": false,
    "permission_requested": false,
    "classification": authorized ? "already-authorized" : "permission-required",
    "fail_closed": !authorized,
]
let data = try JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted, .sortedKeys])
if CommandLine.arguments.count == 2 {
    try data.write(to: URL(fileURLWithPath: CommandLine.arguments[1]), options: .atomic)
} else {
    FileHandle.standardOutput.write(data)
    FileHandle.standardOutput.write(Data("\n".utf8))
}
