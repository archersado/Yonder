import Foundation

private let maximumMessageBytes = 1024 * 1024
private var sequence = 0

private func reject(_ code: String) -> Never {
    FileHandle.standardError.write(Data((code + "\n").utf8))
    exit(2)
}

private func readExact(_ count: Int, allowCleanEOF: Bool = false) -> Data? {
    var result = Data()
    while result.count < count {
        let chunk = FileHandle.standardInput.readData(ofLength: count - result.count)
        if chunk.isEmpty {
            if allowCleanEOF && result.isEmpty { return nil }
            reject("frame_incomplete")
        }
        result.append(chunk)
    }
    return result
}

private func messageLength(_ header: Data) -> Int {
    let bytes = [UInt8](header)
    return Int(UInt32(bytes[0]))
        | Int(UInt32(bytes[1]) << 8)
        | Int(UInt32(bytes[2]) << 16)
        | Int(UInt32(bytes[3]) << 24)
}

private func writeMessage(_ value: [String: Any]) {
    guard let payload = try? JSONSerialization.data(withJSONObject: value),
          payload.count <= maximumMessageBytes else { reject("response_invalid") }
    var length = UInt32(payload.count).littleEndian
    let header = withUnsafeBytes(of: &length) { Data($0) }
    FileHandle.standardOutput.write(header)
    FileHandle.standardOutput.write(payload)
}

while let header = readExact(4, allowCleanEOF: true) {
    let length = messageLength(header)
    guard length > 0, length <= maximumMessageBytes else { reject("frame_too_large") }
    guard let payload = readExact(length),
          let object = try? JSONSerialization.jsonObject(with: payload) as? [String: Any],
          let type = object["type"] as? String,
          ["recording.started", "recording.stopped", "tab.activated", "tab.updated", "protocol.utf8"].contains(type)
    else { reject("message_invalid") }
    sequence += 1
    writeMessage(["accepted": true, "sequence": sequence])
}
