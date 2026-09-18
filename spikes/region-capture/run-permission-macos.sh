#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
bundle="$root/.permission-probe.app"
contents="$bundle/Contents"
executable="$contents/MacOS/region-permission-probe"
result="$contents/result.json"
cleanup() { rm -rf "$bundle"; }
trap cleanup EXIT HUP INT TERM
cleanup
mkdir -p "$contents/MacOS"
cat >"$contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleExecutable</key><string>region-permission-probe</string>
  <key>CFBundleIdentifier</key><string>com.yonder.spike.region-capture-permission</string>
  <key>CFBundleName</key><string>Yonder Region Permission Probe</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>NSScreenCaptureUsageDescription</key><string>Validate fail-closed screen capture permission handling.</string>
</dict></plist>
PLIST
xcrun swiftc "$root/macos-permission-probe.swift" -o "$executable" -framework CoreGraphics -framework Foundation
open -gjW "$bundle" --args "$result"
cat "$result"
python3 -c 'import json,sys; sys.exit(0 if json.load(open(sys.argv[1]))["fail_closed"] else 3)' "$result"
