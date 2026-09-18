#!/bin/sh
set -eu

output=${1:-evidence/eventkit-reminder-macos-20260918/result.json}
case "$output" in
  /*) ;;
  *) output="$PWD/$output" ;;
esac
build=$(mktemp -d /private/tmp/yonder-eventkit.XXXXXX)
trap 'rm -rf "$build"' EXIT
authorization="$(dirname "$output")/authorization.json"
app="$build/YonderEventKitSpike.app"
mkdir -p "$app/Contents/MacOS" "$(dirname "$output")"
/usr/bin/swiftc -parse-as-library eventkit-reminder-macos.swift -o "$app/Contents/MacOS/YonderEventKitSpike" -framework AppKit -framework EventKit
cat > "$app/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>YonderEventKitSpike</string>
<key>CFBundleIdentifier</key><string>com.yonder.spike.eventkit</string>
<key>CFBundleName</key><string>Yonder EventKit Spike</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>LSUIElement</key><true/>
<key>NSRemindersFullAccessUsageDescription</key><string>验证Yonder后台原生动作的创建、观察与清理。</string>
<key>NSRemindersUsageDescription</key><string>验证Yonder后台原生动作的创建、观察与清理。</string>
</dict></plist>
PLIST
/usr/bin/codesign --force --sign - "$app" >/dev/null
/usr/bin/open -g -W -n "$app" --args --authorize "$authorization"
cat "$authorization"
/usr/bin/python3 - "$authorization" <<'PY'
import json,sys
raise SystemExit(0 if json.load(open(sys.argv[1]))["passed"] else 1)
PY
/usr/bin/open -g -W -n "$app" --args --background "$output"
cat "$output"
/usr/bin/python3 - "$output" <<'PY'
import json,sys
raise SystemExit(0 if json.load(open(sys.argv[1]))["passed"] else 1)
PY
