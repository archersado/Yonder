import AVFoundation
import Foundation

final class ProbeDelegate: NSObject, AVSpeechSynthesizerDelegate {
    private let identifiers: [ObjectIdentifier: Int]
    var starts: [Int] = []
    var finishes: [Int] = []
    var cancellations: [Int] = []
    var firstStartAt: Date?
    var cancelledAt: Date?

    init(_ utterances: [AVSpeechUtterance]) {
        identifiers = Dictionary(uniqueKeysWithValues: utterances.enumerated().map {
            (ObjectIdentifier($0.element), $0.offset)
        })
    }

    func speechSynthesizer(_ synthesizer: AVSpeechSynthesizer, didStart utterance: AVSpeechUtterance) {
        guard let index = identifiers[ObjectIdentifier(utterance)] else { return }
        starts.append(index)
        if firstStartAt == nil { firstStartAt = Date() }
    }

    func speechSynthesizer(_ synthesizer: AVSpeechSynthesizer, didFinish utterance: AVSpeechUtterance) {
        guard let index = identifiers[ObjectIdentifier(utterance)] else { return }
        finishes.append(index)
    }

    func speechSynthesizer(_ synthesizer: AVSpeechSynthesizer, didCancel utterance: AVSpeechUtterance) {
        guard let index = identifiers[ObjectIdentifier(utterance)] else { return }
        cancellations.append(index)
        cancelledAt = Date()
    }
}

func waitUntil(timeout: TimeInterval, condition: () -> Bool) -> Bool {
    let deadline = Date().addingTimeInterval(timeout)
    while Date() < deadline {
        if condition() { return true }
        RunLoop.current.run(until: Date().addingTimeInterval(0.01))
    }
    return condition()
}

func makeUtterance(index: Int, voice: AVSpeechSynthesisVoice) -> AVSpeechUtterance {
    let samples = ["编号一。", "编号二。", "编号三。"]
    let utterance = AVSpeechUtterance(string: samples[index])
    utterance.voice = voice
    utterance.volume = 0
    utterance.rate = AVSpeechUtteranceMaximumSpeechRate
    return utterance
}

let chineseVoices = AVSpeechSynthesisVoice.speechVoices().filter {
    $0.language.lowercased().hasPrefix("zh")
}

var report: [String: Any] = [
    "kind": "macos-native-voice-reply-capability",
    "platform": "macos",
    "installed_chinese_voice_count": chineseVoices.count,
    "local_voice_available": !chineseVoices.isEmpty,
    "offline_verified": false,
    "output_device_switch_verified": false,
    "contains_text_or_voice_name": false,
]

guard let voice = chineseVoices.first else {
    report["result"] = "local_voice_unavailable"
    let data = try JSONSerialization.data(withJSONObject: report, options: [.sortedKeys])
    print(String(decoding: data, as: UTF8.self))
    exit(0)
}

let queueUtterances = (0..<3).map { makeUtterance(index: $0, voice: voice) }
let queueDelegate = ProbeDelegate(queueUtterances)
var queueSynthesizer: AVSpeechSynthesizer? = AVSpeechSynthesizer()
queueSynthesizer?.delegate = queueDelegate
let queueStartedAt = Date()
for utterance in queueUtterances { queueSynthesizer?.speak(utterance) }
let queueCompleted = waitUntil(timeout: 8) { queueDelegate.finishes.count == 3 }
let queueElapsedMs = Int(Date().timeIntervalSince(queueStartedAt) * 1_000)
let strictOrder = queueDelegate.starts == [0, 1, 2] && queueDelegate.finishes == [0, 1, 2]
queueSynthesizer?.delegate = nil
queueSynthesizer = nil

let longSample = Array(repeating: "编号。", count: 80).joined()
let stopUtterance = AVSpeechUtterance(string: longSample)
stopUtterance.voice = voice
stopUtterance.volume = 0
stopUtterance.rate = AVSpeechUtteranceMinimumSpeechRate
let stopDelegate = ProbeDelegate([stopUtterance])
var stopSynthesizer: AVSpeechSynthesizer? = AVSpeechSynthesizer()
stopSynthesizer?.delegate = stopDelegate
stopSynthesizer?.speak(stopUtterance)
let didStart = waitUntil(timeout: 3) { stopDelegate.firstStartAt != nil }
let stopRequestedAt = Date()
let stopAccepted = stopSynthesizer?.stopSpeaking(at: .immediate) ?? false
var stoppedSpeakingAt: Date?
let stoppedSpeaking = waitUntil(timeout: 3) {
    let stopped = !(stopSynthesizer?.isSpeaking ?? false)
    if stopped && stoppedSpeakingAt == nil { stoppedSpeakingAt = Date() }
    return stopped
}
let terminalCallback = waitUntil(timeout: 3) {
    stopDelegate.cancellations.count + stopDelegate.finishes.count == 1
}
RunLoop.current.run(until: Date().addingTimeInterval(0.35))
let stopElapsedMs = stoppedSpeakingAt.map {
    Int($0.timeIntervalSince(stopRequestedAt) * 1_000)
} ?? -1
let terminalCallbackCount = stopDelegate.cancellations.count + stopDelegate.finishes.count
let nativeCancelCallback = stopDelegate.cancellations.count == 1
let nativeFinishAfterStop = stopDelegate.finishes.count == 1
stopSynthesizer?.delegate = nil
stopSynthesizer = nil

report["queue_completed"] = queueCompleted
report["strict_callback_order"] = strictOrder
report["queue_elapsed_ms"] = queueElapsedMs
report["stop_started"] = didStart
report["stop_request_accepted"] = stopAccepted
report["stopped_speaking"] = stoppedSpeaking
report["terminal_callback_once"] = terminalCallback && terminalCallbackCount == 1
report["native_cancel_callback"] = nativeCancelCallback
report["native_finish_after_stop"] = nativeFinishAfterStop
report["requires_generation_guard"] = nativeFinishAfterStop
report["stop_elapsed_ms"] = stopElapsedMs
report["released_after_probe"] = queueSynthesizer == nil && stopSynthesizer == nil
report["result"] = queueCompleted && strictOrder && didStart && stopAccepted && stoppedSpeaking
    && terminalCallback && terminalCallbackCount == 1
    ? "candidate_pass"
    : "candidate_fail"

let data = try JSONSerialization.data(withJSONObject: report, options: [.sortedKeys])
print(String(decoding: data, as: UTF8.self))
