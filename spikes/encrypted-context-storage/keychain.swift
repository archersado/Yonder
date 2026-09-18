// E0-S6 原生探针：只操作本次创建的 UUID 条目，不输出密钥。
import Foundation
import Security

struct ProbeFailure: Error { let step: String; let status: OSStatus }
func require(_ status: OSStatus, _ step: String, _ expected: OSStatus = errSecSuccess) throws {
    if status != expected { throw ProbeFailure(step: step, status: status) }
}

let service = "com.yonder.e0-s6.probe." + UUID().uuidString
let query: [String: Any] = [
    kSecClass as String: kSecClassGenericPassword,
    kSecAttrService as String: service,
    kSecAttrAccount as String: "temporary-master-key",
    kSecAttrSynchronizable as String: false,
]
var secret = Data(count: 32)
var written = false
var deleted = false
var report: [String: Any] = ["platform": "macOS", "probe": "E0-S6-Keychain", "passed": false]
do {
    let random = secret.withUnsafeMutableBytes { SecRandomCopyBytes(kSecRandomDefault, $0.count, $0.baseAddress!) }
    try require(random, "random")
    var item = query
    item[kSecValueData as String] = secret
    item[kSecAttrAccessible as String] = kSecAttrAccessibleWhenUnlockedThisDeviceOnly
    try require(SecItemAdd(item as CFDictionary, nil), "write")
    written = true
    report["written"] = true
    try require(SecItemAdd(item as CFDictionary, nil), "duplicate", errSecDuplicateItem)
    report["duplicate_rejected"] = true
    item.removeValue(forKey: kSecValueData as String)
    var read = query
    read[kSecReturnData as String] = true
    read[kSecMatchLimit as String] = kSecMatchLimitOne
    var result: CFTypeRef?
    try require(SecItemCopyMatching(read as CFDictionary, &result), "read")
    guard var copy = result as? Data else { throw ProbeFailure(step: "read_type", status: errSecDecode) }
    result = nil
    let equal = copy == secret && copy.count == 32
    copy.resetBytes(in: 0..<copy.count)
    guard equal else { throw ProbeFailure(step: "roundtrip", status: errSecDecode) }
    report["roundtrip"] = true
    try require(SecItemDelete(query as CFDictionary), "delete")
    deleted = true
    report["deleted"] = true
    try require(SecItemCopyMatching(read as CFDictionary, &result), "absent_after_delete", errSecItemNotFound)
    report["absent_after_delete"] = true
    report["passed"] = true
} catch let error as ProbeFailure {
    report["failed_step"] = error.step
    report["os_status"] = Int(error.status)
} catch {
    report["failed_step"] = "unexpected"
}
if written && !deleted {
    let cleanup = SecItemDelete(query as CFDictionary)
    report["cleanup_status"] = Int(cleanup)
    if cleanup != errSecSuccess && cleanup != errSecItemNotFound { report["cleanup_service"] = service }
}
secret.resetBytes(in: 0..<secret.count)
// Swift/CF 可能复制缓冲区；本探针不宣称实现产品级内存清零保证。
let output = try JSONSerialization.data(withJSONObject: report, options: [.sortedKeys])
print(String(data: output, encoding: .utf8)!)
exit(report["passed"] as? Bool == true ? 0 : 1)
