import AVFoundation
import Foundation
import Speech

func authorization(_ status: AVAuthorizationStatus) -> String {
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

let recognizer = SFSpeechRecognizer(locale: Locale(identifier: "zh-CN"))
let result: [String: Any] = [
    "platform": "macOS",
    "capture_framework": "AVFoundation",
    "speech_framework": "Speech",
    "microphone_permission": authorization(AVCaptureDevice.authorizationStatus(for: .audio)),
    "speech_permission": speechAuthorization(SFSpeechRecognizer.authorizationStatus()),
    "zh_cn_recognizer": recognizer != nil,
    "recognizer_available": recognizer?.isAvailable ?? false,
    "on_device_supported": recognizer?.supportsOnDeviceRecognition ?? false,
    "requested_permission": false,
    "opened_microphone": false
]
let data = try JSONSerialization.data(withJSONObject: result, options: [.sortedKeys])
print(String(decoding: data, as: UTF8.self))
