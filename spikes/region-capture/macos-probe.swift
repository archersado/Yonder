import AppKit
import CoreGraphics
import Foundation
import ScreenCaptureKit

final class PatternView: NSView {
    override func draw(_ dirtyRect: NSRect) {
        guard let context = NSGraphicsContext.current?.cgContext else { return }
        let w = bounds.width / 2, h = bounds.height / 2
        for (rect, color) in [
            (CGRect(x: 0, y: h, width: w, height: h), CGColor(red: 1, green: 0, blue: 0, alpha: 1)),
            (CGRect(x: w, y: h, width: w, height: h), CGColor(red: 0, green: 1, blue: 0, alpha: 1)),
            (CGRect(x: 0, y: 0, width: w, height: h), CGColor(red: 0, green: 0, blue: 1, alpha: 1)),
            (CGRect(x: w, y: 0, width: w, height: h), CGColor(red: 1, green: 1, blue: 0, alpha: 1)),
        ] {
            context.setFillColor(color)
            context.fill(rect)
        }
    }
}

@main struct RegionCaptureProbe {
    @MainActor static func main() async {
        NSApplication.shared.setActivationPolicy(.accessory)
        guard let screen = NSScreen.main else { exitWith(["passed": false, "error": "display-unavailable"]) }
        let size = CGSize(width: 240, height: 160)
        let origin = CGPoint(x: screen.visibleFrame.midX - size.width / 2, y: screen.visibleFrame.midY - size.height / 2)
        let panel = NSPanel(contentRect: CGRect(origin: origin, size: size), styleMask: [.borderless, .nonactivatingPanel], backing: .buffered, defer: false)
        panel.level = .floating
        panel.isOpaque = true
        panel.hasShadow = false
        panel.backgroundColor = .black
        panel.contentView = PatternView(frame: CGRect(origin: .zero, size: size))
        panel.orderFrontRegardless()
        panel.displayIfNeeded()
        defer { panel.orderOut(nil); panel.close() }

        try? await Task.sleep(for: .milliseconds(250))
        guard let info = (CGWindowListCopyWindowInfo([.optionIncludingWindow], CGWindowID(panel.windowNumber)) as? [[String: Any]])?.first,
              let dictionary = info[kCGWindowBounds as String] as? NSDictionary,
              let captureRect = CGRect(dictionaryRepresentation: dictionary) else {
            exitWith(["passed": false, "error": "window-bounds-unavailable"])
        }

        do {
            let image = try await SCScreenshotManager.captureImage(in: captureRect)
            let samples = rgbaSamples(image)
            let scaleX = Double(image.width) / captureRect.width
            let scaleY = Double(image.height) / captureRect.height
            let colorsPass = samples.count == 4
                && samples[0][0] > 200 && samples[0][1] < 80 && samples[0][2] < 80
                && samples[1][1] > 200 && samples[1][0] < 80 && samples[1][2] < 80
                && samples[2][2] > 200 && samples[2][0] < 80 && samples[2][1] < 80
                && samples[3][0] > 200 && samples[3][1] > 200 && samples[3][2] < 80
            let scalePass = abs(scaleX - scaleY) < 0.01 && scaleX >= 1
            exitWith([
                "passed": colorsPass && scalePass,
                "capture": ["points_width": Int(captureRect.width), "points_height": Int(captureRect.height), "pixels_width": image.width, "pixels_height": image.height],
                "scale": scaleX,
                "display_count": NSScreen.screens.count,
                "negative_origin_available": NSScreen.screens.contains { $0.frame.minX < 0 || $0.frame.minY < 0 },
                "color_checks": colorsPass,
                "scale_check": scalePass,
                "pixels_released_after_check": true,
                "screenshot_persisted": false,
            ])
        } catch {
            exitWith(["passed": false, "error": "permission-required-or-capture-failed", "error_code": (error as NSError).code, "screenshot_persisted": false])
        }
    }

    static func rgbaSamples(_ image: CGImage) -> [[Int]] {
        let width = image.width, height = image.height, bytesPerRow = width * 4
        var bytes = [UInt8](repeating: 0, count: bytesPerRow * height)
        let context = CGContext(data: &bytes, width: width, height: height, bitsPerComponent: 8, bytesPerRow: bytesPerRow,
                                space: CGColorSpace(name: CGColorSpace.sRGB)!, bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
        context.draw(image, in: CGRect(x: 0, y: 0, width: width, height: height))
        return [(width / 4, height / 4), (width * 3 / 4, height / 4), (width / 4, height * 3 / 4), (width * 3 / 4, height * 3 / 4)].map { x, y in
            let offset = y * bytesPerRow + x * 4
            return [Int(bytes[offset]), Int(bytes[offset + 1]), Int(bytes[offset + 2])]
        }
    }

    static func exitWith(_ result: [String: Any]) -> Never {
        let data = try! JSONSerialization.data(withJSONObject: result, options: [.prettyPrinted, .sortedKeys])
        FileHandle.standardOutput.write(data)
        FileHandle.standardOutput.write(Data("\n".utf8))
        exit((result["passed"] as? Bool) == true ? 0 : 2)
    }
}
