#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
out="$root/.selection-probe"
xcrun swiftc -parse-as-library "$root/macos-selection-probe.swift" -o "$out" -framework AppKit -framework CoreGraphics -framework Foundation
trap 'rm -f "$out"' EXIT HUP INT TERM
"$out"
