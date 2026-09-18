#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
out="$root/.macos-probe"
xcrun swiftc -parse-as-library "$root/macos-probe.swift" -o "$out" -framework AppKit -framework CoreGraphics -framework ScreenCaptureKit
trap 'rm -f "$out"' EXIT HUP INT TERM
"$out"
