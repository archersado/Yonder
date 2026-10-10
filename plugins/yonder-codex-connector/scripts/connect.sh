#!/bin/sh
set -eu

if [ -n "${YONDER_CLI_PATH:-}" ] && [ -x "${YONDER_CLI_PATH}" ]; then
  exec "${YONDER_CLI_PATH}" codex-hook
fi

if command -v yonder >/dev/null 2>&1; then
  exec yonder codex-hook
fi

for candidate in \
  "/Applications/Yonda.app/Contents/MacOS/yonder" \
  "/Applications/Yonder.app/Contents/MacOS/yonder" \
  "${HOME}/Applications/Yonda.app/Contents/MacOS/yonder" \
  "${HOME}/Applications/Yonder.app/Contents/MacOS/yonder"
do
  if [ -x "${candidate}" ]; then
    exec "${candidate}" codex-hook
  fi
done

printf '%s\n' 'Yonder CLI 未安装或不可执行' >&2
exit 127
