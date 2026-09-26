import AVFoundation
import Foundation
import Speech

func captureAuthorization(_ status: AVAuthorizationStatus) -> String {
    switch status {
    case .authorized: return "authorized"
    case .denied: return "denied"
    case .restricted: return "restricted"
    case .notDetermined: return "not_determined"
    @unknown default: return "unknown"
    }
}

func speechAuthorization(_ status: SFSpeechRecognizerAuthorizationStatus) -> String {
    switch status {
    case .authorized: return "authorized"
    case .denied: return "denied"
    case .restricted: return "restricted"
    case .notDetermined: return "not_determined"
    @unknown default: return "unknown"
    }
}

let discovery = AVCaptureDevice.DiscoverySession(
    deviceTypes: [.microphone],
    mediaType: .audio,
    position: .unspecified
)
let inputCount = discovery.devices.count
let result: [String: Any] = [
    "platform": "macOS",
    "input_device_count": inputCount,
    "can_verify_real_device_switch": inputCount >= 2,
    "microphone_permission": captureAuthorization(AVCaptureDevice.authorizationStatus(for: .audio)),
    "speech_permission": speechAuthorization(SFSpeechRecognizer.authorizationStatus()),
    "requested_permission": false,
    "opened_microphone": false,
    "contains_device_name_or_identifier": false
]
let data = try JSONSerialization.data(withJSONObject: result, options: [.sortedKeys])
print(String(decoding: data, as: UTF8.self))
