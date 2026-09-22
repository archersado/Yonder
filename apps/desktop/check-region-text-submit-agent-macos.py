#!/usr/bin/env python3
"""验证圈选入口的无截图文字提交；证据不保存正文或屏幕内容。"""

import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import threading
import time


root = Path(__file__).resolve().parents[2]
outcome = sys.argv[1]
output = Path(sys.argv[2])
output.mkdir(parents=True, exist_ok=False)
assert outcome in {"accepted", "rejected", "unknown"}
path = Path(os.environ.get("YONDA_AGENT_SOCKET", Path.home() / "Library/Application Support/com.yonder.desktop/agent.sock"))
stream = socket.socket(socket.AF_UNIX)
stream.connect(str(path))
reader = stream.makefile("rb")


def send(value):
    frame = json.dumps(value, separators=(",", ":")).encode()
    assert len(frame) <= 65536
    stream.sendall(frame + b"\n")


send({"jsonrpc": "2.0", "id": "hello", "method": "gateway.hello", "params": {
    "agent_id": "codex-cli", "capability": "task.read", "deadline": int(time.time() * 1000) + 60000,
    "protocol_version": {"major": 1, "minor": 19}, "session_id": f"region-text-{outcome}", "offered_capabilities": ["user_input"],
}})
hello = json.loads(reader.readline())
version = hello["result"]["protocol_version"]
assert version["major"] == 1 and 14 <= version["minor"] < 19
result = {"outcome": outcome, "frames": 0, "attachment_frames": 0, "source_is_selection": False,
          "attachment_absent": False, "content_nonempty": False, "content_recorded": False, "passed": False}


def serve():
    frame = json.loads(reader.readline())
    result["frames"] += 1
    result["attachment_frames"] += int(frame["method"].startswith("agent.attachment."))
    params = frame["params"]
    result["source_is_selection"] = params.get("source") == "selection"
    result["attachment_absent"] = "attachment_id" not in params
    result["content_nonempty"] = bool(params.get("content", "").strip())
    response_id = "wrong-id" if outcome == "unknown" else frame["id"]
    send({"jsonrpc": "2.0", "id": response_id, "result": {"accepted": outcome == "accepted"}})


worker = threading.Thread(target=serve, daemon=True)
worker.start()
environment = os.environ | {"YONDA_TEXT_SUBMIT_OUTCOME": outcome}
try:
    native = subprocess.run(["swift", str(root / "apps/desktop/check-region-preview-macos.swift")], env=environment, capture_output=True, text=True, timeout=45)
except subprocess.TimeoutExpired:
    native = subprocess.CompletedProcess([], 124, "", "native-timeout")
if worker.is_alive():
    stream.shutdown(socket.SHUT_RDWR)
worker.join(timeout=5)
for candidate in path.parent.iterdir():
    if candidate.is_file():
        try: result["content_recorded"] |= b"explain without image" in candidate.read_bytes()
        except OSError: pass
result["worker_finished"] = not worker.is_alive()
result["native_passed"] = native.returncode == 0
result["passed"] = all([result["worker_finished"], result["native_passed"], result["frames"] == 1,
                        result["attachment_frames"] == 0, result["source_is_selection"], result["attachment_absent"],
                        result["content_nonempty"], not result["content_recorded"]])
(output / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
print(json.dumps(result, ensure_ascii=False))
reader.close(); stream.close()
raise SystemExit(0 if result["passed"] else 1)
