#!/usr/bin/env python3
"""AG-S7 macOS capability probe; does not attach to a real thread or persist identifiers."""

import json
import os
import pathlib
import re
import subprocess


ROOT = pathlib.Path(__file__).resolve().parents[2]
BINARY = ROOT / "target/debug/yonder"


def command(*arguments):
    return subprocess.run(arguments, capture_output=True, text=True, timeout=10)


def help_supports(arguments, *needles):
    result = command(*arguments)
    text = result.stdout + result.stderr
    return result.returncode == 0 and all(needle in text for needle in needles)


def bridge_failure(environment, expected):
    result = subprocess.run(
        [str(BINARY), "agent-bridge"],
        env=environment,
        capture_output=True,
        text=True,
        timeout=10,
    )
    return result.returncode != 0 and expected in result.stderr


def chatgpt_app_server_transport():
    result = command("ps", "-axo", "command")
    commands = [
        line.strip()
        for line in result.stdout.splitlines()
        if line.startswith("/Applications/ChatGPT.app/Contents/Resources/codex ")
        and " app-server" in line
    ]
    return {
        "sample_count": len(commands),
        "private_stdio_only": bool(commands)
        and all("unix://" not in value and "ws://" not in value and "wss://" not in value for value in commands),
        "public_socket_found": any("unix://" in value or "ws://" in value or "wss://" in value for value in commands),
    }


def main():
    if not BINARY.exists():
        raise SystemExit("请先构建yonder-cli")
    version = command("codex", "--version")
    version_match = re.search(r"codex-cli\s+(\d+)\.(\d+)\.(\d+)", version.stdout)
    version_supported = bool(version_match) and tuple(map(int, version_match.groups())) >= (0, 156, 1)
    environment = os.environ.copy()
    environment["YONDER_AGENT_ID"] = "codex-lifecycle-probe"
    environment.pop("YONDER_CODEX_THREAD_ID", None)
    environment.pop("YONDER_CODEX_APP_SERVER_SOCKET", None)
    missing_thread_fails_closed = bridge_failure(environment, "请设置YONDER_CODEX_THREAD_ID")
    environment["YONDER_CODEX_THREAD_ID"] = "thread-lifecycle-probe"
    missing_endpoint_fails_closed = bridge_failure(environment, "请设置YONDER_CODEX_APP_SERVER_SOCKET")
    desktop = chatgpt_app_server_transport()
    report = {
        "passed": version_supported
        and help_supports(("codex", "app-server", "--help"), "unix://")
        and help_supports(("codex", "resume", "--help"), "--remote", "unix://")
        and missing_thread_fails_closed
        and missing_endpoint_fails_closed,
        "codex_cli_version_supported": version_supported,
        "shared_unix_app_server_supported": help_supports(("codex", "app-server", "--help"), "unix://"),
        "remote_resume_supported": help_supports(("codex", "resume", "--help"), "--remote", "unix://"),
        "missing_thread_fails_closed": missing_thread_fails_closed,
        "missing_endpoint_fails_closed": missing_endpoint_fails_closed,
        "chatgpt_desktop": desktop,
    }
    print(json.dumps(report, ensure_ascii=False, sort_keys=True))
    if not report["passed"]:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
