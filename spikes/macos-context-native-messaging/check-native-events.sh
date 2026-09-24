#!/bin/sh
set -eu

SCRIPT_DIRECTORY=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
BUILD_DIRECTORY=$(mktemp -d /tmp/yonder-cx-s1-events.XXXXXX)
cleanup() {
  rm -rf "$BUILD_DIRECTORY"
}
trap cleanup EXIT INT TERM

xcrun swiftc "$SCRIPT_DIRECTORY/foreground-context.swift" -o "$BUILD_DIRECTORY/foreground-context"

"$BUILD_DIRECTORY/foreground-context" \
  --observe 8 \
  --fixture-app /System/Applications/TextEdit.app \
  --fixture-app /System/Applications/Calculator.app \
  --fixture-app /System/Library/CoreServices/Finder.app \
  > "$BUILD_DIRECTORY/result.json"

python3 - "$BUILD_DIRECTORY/result.json" <<'PY'
import json, pathlib, sys
value = json.loads(pathlib.Path(sys.argv[1]).read_text())
value["passed"] = (
    value.get("status") == "stopped"
    and value.get("activation_events", 0) >= 1
    and value.get("observer_attempts", 0) >= 2
    and value.get("observer_registrations", 0) >= 1
    and value.get("observer_released") is True
    and value.get("workspace_notification_removed") is True
)
print(json.dumps(value, ensure_ascii=False, sort_keys=True))
if not value["passed"]:
    raise SystemExit(1)
PY
