#!/usr/bin/env python3
"""验证圈选语音只产生一条 selection 输入；证据不保存转写或截图。"""

import base64
import hashlib
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import threading
import time


root = Path(__file__).resolve().parents[2]
mode, output = sys.argv[1], Path(sys.argv[2])
assert mode in {"image", "text", "cancel", "direct"}
output.mkdir(parents=True, exist_ok=False)
path = Path(os.environ.get("YONDA_AGENT_SOCKET", Path.home() / "Library/Application Support/com.yonder.desktop/agent.sock"))
stream = socket.socket(socket.AF_UNIX)
stream.connect(str(path))
reader = stream.makefile("rb")


def send(value):
    frame = json.dumps(value, separators=(",", ":")).encode()
    assert len(frame) <= 65536
    stream.sendall(frame + b"\n")


def receive():
    raw = reader.readline(65538)
    if not raw.endswith(b"\n") or len(raw) > 65537: raise ConnectionError("agent connection closed")
    return raw, json.loads(raw)


send({"jsonrpc": "2.0", "id": "hello", "method": "gateway.hello", "params": {
    "agent_id": "codex-cli", "capability": "task.read", "deadline": int(time.time() * 1000) + 60000,
    "protocol_version": {"major": 1, "minor": 19}, "session_id": f"region-voice-{mode}",
    "offered_capabilities": ["user_input", "user_input_attachment"],
}})
_, hello = receive()
assert hello["result"]["protocol_version"]["major"] == 1
result = {"mode": mode, "frames": 0, "input_frames": 0, "selection_frames": 0, "voice_frames": 0,
          "attachment_frames": 0, "attachment_valid": False, "content_nonempty": False, "extra_frames": 0, "passed": False}
captured = bytearray()


def serve():
    attachment = expected_hash = None
    expected_bytes = sequence = 0
    while True:
        try: raw, message = receive()
        except (ConnectionError, OSError): return
        result["frames"] += 1
        assert len(raw) - 1 <= 65536
        method, params = message["method"], message["params"]
        result["attachment_frames"] += int(method.startswith("agent.attachment."))
        if method == "agent.attachment.begin":
            attachment, expected_hash, expected_bytes = params["attachment_id"], params["sha256"], params["byte_length"]
        elif method == "agent.attachment.chunk":
            assert params["attachment_id"] == attachment and params["sequence"] == sequence
            captured.extend(base64.b64decode(params["data_base64"], validate=True)); sequence += 1
        elif method == "agent.attachment.finish":
            result["attachment_valid"] = len(captured) == expected_bytes and hashlib.sha256(captured).hexdigest() == expected_hash
            send({"jsonrpc": "2.0", "id": message["id"], "result": {"accepted": result["attachment_valid"]}})
        elif method == "agent.input":
            result["input_frames"] += 1
            result["selection_frames"] += int(params.get("source") == "selection")
            result["voice_frames"] += int(params.get("source") == "voice")
            result["content_nonempty"] = bool(params.get("content", "").strip())
            if mode == "image": result["attachment_valid"] &= params.get("attachment_id") == attachment
            else: result["attachment_valid"] = "attachment_id" not in params
            send({"jsonrpc": "2.0", "id": message["id"], "result": {"accepted": True}})
            stream.settimeout(1.2)
            try: result["extra_frames"] = int(bool(reader.readline()))
            except (OSError, socket.timeout): pass
            return


worker = None if mode == "cancel" else threading.Thread(target=serve, daemon=True)
if worker: worker.start()
environment = os.environ | {"YONDA_VOICE_SUBMIT_MODE": mode}
if os.environ.get("YONDA_VOICE_MANUAL") == "1":
    process = subprocess.Popen(["swift", str(root / "apps/desktop/check-region-preview-macos.swift")], env=environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    first = process.stdout.readline(); print(first, end="", flush=True)
    stdout, stderr = process.communicate(timeout=55)
    native = subprocess.CompletedProcess(process.args, process.returncode, first + stdout, stderr)
else:
    native = subprocess.run(["swift", str(root / "apps/desktop/check-region-preview-macos.swift")], env=environment, capture_output=True, text=True, timeout=55)
if worker:
    if worker.is_alive() and native.returncode != 0: stream.shutdown(socket.SHUT_RDWR)
    worker.join(timeout=5)
else:
    stream.settimeout(1.2)
    try: result["extra_frames"] = int(bool(reader.readline()))
    except (OSError, socket.timeout): pass
result["native_passed"] = native.returncode == 0
try: result["native_reason"] = json.loads(native.stdout.strip().splitlines()[-1]).get("reason")
except (IndexError, json.JSONDecodeError): result["native_reason"] = "none"
result["worker_finished"] = worker is None or not worker.is_alive()
result["passed"] = result["native_passed"] and result["worker_finished"] and result["extra_frames"] == 0 and (
    result["frames"] == 0 if mode == "cancel" else
    result["input_frames"] == 1 and result["selection_frames"] == int(mode != "direct") and result["voice_frames"] == int(mode == "direct") and result["content_nonempty"] and result["attachment_valid"])
(output / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
print(json.dumps(result, ensure_ascii=False))
reader.close(); stream.close()
raise SystemExit(0 if result["passed"] else 1)
