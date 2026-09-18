import AppKit
import AVFoundation
import Speech

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    private let engine = AVAudioEngine()
    private let recognizer = SFSpeechRecognizer(locale: Locale(identifier: "zh-CN"))
    private var request: SFSpeechAudioBufferRecognitionRequest?
    private var task: SFSpeechRecognitionTask?
    private var timer: Timer?
    private var started = false
    private let status = NSTextField(labelWithString: "点击“开始测试”后才会启用麦克风")
    private let transcript = NSTextView()
    private let start = NSButton(title: "开始测试", target: nil, action: nil)
    private let stop = NSButton(title: "停止", target: nil, action: nil)

    func applicationDidFinishLaunching(_ notification: Notification) {
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 520, height: 320), styleMask: [.titled, .closable], backing: .buffered, defer: false)
        window.title = "Yonder 语音输入测试"
        window.center()

        let stack = NSStackView()
        stack.orientation = .vertical
        stack.spacing = 12
        stack.edgeInsets = NSEdgeInsets(top: 20, left: 20, bottom: 20, right: 20)
        stack.translatesAutoresizingMaskIntoConstraints = false
        status.alignment = .center
        transcript.isEditable = false
        transcript.font = .systemFont(ofSize: 16)
        transcript.string = "转写文字会显示在这里，不会保存。"
        let scroll = NSScrollView()
        scroll.documentView = transcript
        scroll.hasVerticalScroller = true
        scroll.borderType = .bezelBorder
        let buttons = NSStackView(views: [start, stop])
        buttons.spacing = 12
        start.target = self
        start.action = #selector(begin)
        stop.target = self
        stop.action = #selector(end)
        stop.isEnabled = false
        stack.addArrangedSubview(status)
        stack.addArrangedSubview(scroll)
        stack.addArrangedSubview(buttons)
        window.contentView = stack
        NSLayoutConstraint.activate([scroll.widthAnchor.constraint(equalToConstant: 480), scroll.heightAnchor.constraint(equalToConstant: 190)])
        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }

    @objc private func begin() {
        guard !started else { return }
        status.stringValue = "正在请求系统权限…"
        SFSpeechRecognizer.requestAuthorization { [weak self] speech in
            AVCaptureDevice.requestAccess(for: .audio) { mic in
                DispatchQueue.main.async { self?.authorized(speech == .authorized && mic) }
            }
        }
    }

    private func authorized(_ allowed: Bool) {
        guard allowed else {
            status.stringValue = "权限未允许；可关闭窗口后在系统设置中调整"
            return
        }
        do {
            let input = engine.inputNode
            let format = input.outputFormat(forBus: 0)
            let request = SFSpeechAudioBufferRecognitionRequest()
            request.shouldReportPartialResults = true
            if recognizer?.supportsOnDeviceRecognition == true { request.requiresOnDeviceRecognition = true }
            input.installTap(onBus: 0, bufferSize: 1024, format: format) { buffer, _ in request.append(buffer) }
            engine.prepare()
            try engine.start()
            self.request = request
            started = true
            start.isEnabled = false
            stop.isEnabled = true
            transcript.string = ""
            status.stringValue = "正在聆听（最长20秒）"
            task = recognizer?.recognitionTask(with: request) { [weak self] result, error in
                DispatchQueue.main.async {
                    if let result { self?.transcript.string = result.bestTranscription.formattedString }
                    if result?.isFinal == true || error != nil { self?.finish(message: error == nil ? "转写完成" : "转写失败，请重试") }
                }
            }
            timer = Timer.scheduledTimer(withTimeInterval: 20, repeats: false) { [weak self] _ in
                DispatchQueue.main.async { self?.finish(message: "已到20秒上限") }
            }
        } catch {
            finish(message: "麦克风启动失败")
        }
    }

    @objc private func end() { finish(message: "已停止，等待最终文字") }

    private func finish(message: String) {
        guard started else { status.stringValue = message; return }
        engine.stop()
        engine.inputNode.removeTap(onBus: 0)
        request?.endAudio()
        timer?.invalidate()
        timer = nil
        request = nil
        started = false
        start.isEnabled = true
        stop.isEnabled = false
        status.stringValue = message
    }

    func applicationWillTerminate(_ notification: Notification) { finish(message: "已关闭") }
}

@main
@MainActor
struct VoiceSpike {
    static func main() {
        let app = NSApplication.shared
        let delegate = AppDelegate()
        app.delegate = delegate
        app.setActivationPolicy(.regular)
        app.run()
    }
}
