#!/bin/sh
set -eu

mode=fixture
target=
if [ "${1:-}" = "--real-app" ]; then
  mode=real
  target=${2:?缺少应用路径}
  output=${3:-evidence/macos-real-app-20260918/result.json}
else
  output=${1:-evidence/macos-20260918/result.json}
fi
case "$output" in
  /*) ;;
  *) output="$PWD/$output" ;;
esac

build=$(mktemp -d /private/tmp/yonder-native-action.XXXXXX)
trap 'rm -rf "$build"' EXIT
app="$build/YonderNativeFixture.app"
mkdir -p "$(dirname "$output")"
/usr/bin/swiftc macos-probe.swift -o "$build/macos-probe" -framework AppKit
if [ "$mode" = real ]; then
  "$build/macos-probe" --real-app "$target" "$output"
  exit
fi
mkdir -p "$app/Contents/MacOS"
/usr/bin/swiftc -D FIXTURE macos-probe.swift -o "$app/Contents/MacOS/YonderNativeFixture" -framework AppKit
cat > "$app/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>YonderNativeFixture</string>
<key>CFBundleIdentifier</key><string>com.yonder.spike.native-fixture</string>
<key>CFBundleName</key><string>Yonder Native Fixture</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>LSUIElement</key><true/>
</dict></plist>
PLIST
"$build/macos-probe" "$app" "$output"
