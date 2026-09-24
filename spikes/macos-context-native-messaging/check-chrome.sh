#!/bin/sh
set -eu

SCRIPT_DIRECTORY=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
PROFILE_DIRECTORY=$(mktemp -d /tmp/yonder-cx-s1-chrome.XXXXXX)
INSTALLED=false
cleanup() {
  if test "$INSTALLED" = true; then "$SCRIPT_DIRECTORY/register-chrome-host.sh" uninstall >/dev/null; fi
  rm -rf "$PROFILE_DIRECTORY"
}
trap cleanup EXIT INT TERM

if test ! -x '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'; then
  echo '{"passed":false,"status":"browser_unavailable"}'
  exit 1
fi

"$SCRIPT_DIRECTORY/register-chrome-host.sh" install >/dev/null
INSTALLED=true
node "$SCRIPT_DIRECTORY/check-chrome.mjs" "$PROFILE_DIRECTORY" "$SCRIPT_DIRECTORY/extension"
