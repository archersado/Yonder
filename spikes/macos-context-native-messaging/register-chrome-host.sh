#!/bin/sh
set -eu

SCRIPT_DIRECTORY=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
HOST_DIRECTORY=/tmp/yonder-cx-s1-macos-host
HOST_PATH="$HOST_DIRECTORY/native-host"
MANIFEST_DIRECTORY="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts"
MANIFEST_PATH="$MANIFEST_DIRECTORY/com.yonder.context_spike.json"

case "${1:-}" in
  install)
    if test -e "$MANIFEST_PATH"; then
      echo "manifest_exists" >&2
      exit 2
    fi
    mkdir -p "$HOST_DIRECTORY" "$MANIFEST_DIRECTORY"
    chmod 700 "$HOST_DIRECTORY"
    xcrun swiftc "$SCRIPT_DIRECTORY/native-host.swift" -o "$HOST_PATH"
    chmod 700 "$HOST_PATH"
    sed "s|__HOST_PATH__|$HOST_PATH|g" "$SCRIPT_DIRECTORY/native-host-manifest.json" > "$MANIFEST_PATH"
    chmod 600 "$MANIFEST_PATH"
    echo "installed"
    ;;
  uninstall)
    if test -f "$MANIFEST_PATH"; then rm "$MANIFEST_PATH"; fi
    if test -f "$HOST_PATH"; then rm "$HOST_PATH"; fi
    if test -d "$HOST_DIRECTORY"; then rmdir "$HOST_DIRECTORY" 2>/dev/null || true; fi
    echo "uninstalled"
    ;;
  *)
    echo "usage: register-chrome-host.sh install|uninstall" >&2
    exit 2
    ;;
esac
