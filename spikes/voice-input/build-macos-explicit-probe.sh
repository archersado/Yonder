#!/bin/sh
set -eu

root=/private/tmp/yonda-voice-spike
app="$root/Yonder Voice Spike.app"
mkdir -p "$app/Contents/MacOS"

swiftc -parse-as-library macos-explicit-probe.swift -o "$app/Contents/MacOS/yonda-voice-spike" -framework AppKit -framework AVFoundation -framework Speech
cat > "$app/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleExecutable</key><string>yonda-voice-spike</string>
  <key>CFBundleIdentifier</key><string>com.yonder.voice-spike</string>
  <key>CFBundleName</key><string>Yonder Voice Spike</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>NSMicrophoneUsageDescription</key><string>仅在你点击开始测试后采集本次语音并转成文字。</string>
  <key>NSSpeechRecognitionUsageDescription</key><string>将你主动录制的测试语音转换为文字。</string>
</dict></plist>
PLIST
codesign --force --sign - "$app" >/dev/null
printf '%s\n' "$app"
