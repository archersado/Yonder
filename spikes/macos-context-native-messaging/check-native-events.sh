#!/bin/sh
set -eu

SCRIPT_DIRECTORY=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
BUILD_DIRECTORY=$(mktemp -d /tmp/yonder-cx-s1-events.XXXXXX)
cleanup() {
  rm -rf "$BUILD_DIRECTORY"
}
trap cleanup EXIT INT TERM

xcrun swiftc "$SCRIPT_DIRECTORY/foreground-context.swift" -o "$BUILD_DIRECTORY/foreground-context"
for fixture in One Two; do
  APP_DIRECTORY="$BUILD_DIRECTORY/Fixture${fixture}.app"
  mkdir -p "$APP_DIRECTORY/Contents/MacOS"
  cp "$SCRIPT_DIRECTORY/fixtures/Fixture${fixture}-Info.plist" "$APP_DIRECTORY/Contents/Info.plist"
  xcrun swiftc "$SCRIPT_DIRECTORY/fixture-app.swift" -o "$APP_DIRECTORY/Contents/MacOS/Fixture${fixture}"
  codesign --force --sign - "$APP_DIRECTORY"
done

"$BUILD_DIRECTORY/foreground-context" --observe 17 > "$BUILD_DIRECTORY/result.json" &
PROBE_PROCESS=$!
sleep 1
open -n "$BUILD_DIRECTORY/FixtureOne.app"
sleep 7
open -n "$BUILD_DIRECTORY/FixtureTwo.app"
wait "$PROBE_PROCESS"

python3 - "$BUILD_DIRECTORY/result.json" <<'PY'
import json, pathlib, sys
value = json.loads(pathlib.Path(sys.argv[1]).read_text())
value["passed"] = (
    value.get("status") == "stopped"
    and value.get("activation_events", 0) >= 2
    and value.get("window_events", 0) >= 2
    and value.get("observer_attempts", 0) >= 3
    and value.get("observer_registrations", 0) >= 3
    and value.get("observer_released") is True
    and value.get("workspace_notification_removed") is True
)
print(json.dumps(value, ensure_ascii=False, sort_keys=True))
if not value["passed"]:
    raise SystemExit(1)
PY
