#!/bin/sh
set -eu

SCRIPT_DIRECTORY=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
BUILD_DIRECTORY=$(mktemp -d /tmp/yonder-cx-s1-native.XXXXXX)
trap 'rm -rf "$BUILD_DIRECTORY"' EXIT INT TERM

xcrun swiftc "$SCRIPT_DIRECTORY/foreground-context.swift" -o "$BUILD_DIRECTORY/foreground-context"
exec "$BUILD_DIRECTORY/foreground-context" "$@"
