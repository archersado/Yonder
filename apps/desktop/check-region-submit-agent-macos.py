#!/usr/bin/env python3
"""用受控本地Agent验证圈选附件提交；证据不保存截图、正文或摘要。"""

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
outcome = sys.argv[1]
output = Path(sys.argv[2])
output.mkdir(parents=True, exist_ok=False)
assert outcome in {"accepted", "rejected", "unknown", "unsupported"}
path = Path.home() / "Library/Application Support/com.yonder.desktop/agent.sock"
stream = socket.socket(socket.AF_UNIX)
stream.connect(str(path))
reader = stream.makefile("rb")


def send(value):
    frame = json.dumps(value, separators=(",", ":")).encode()
    assert len(frame) <= 65536
    stream.sendall(frame + b"\n")


def receive():
    frame = reader.readline(65538)
    assert frame.endswith(b"\n") and len(frame) <= 65537
    return frame, json.loads(frame)


capabilities = ["user_input"] if outcome == "unsupported" else ["user_input", "user_input_attachment"]
send({"jsonrpc": "2.0", "id": "hello", "method": "gateway.hello", "params": {
    "agent_id": "codex-cli", "capability": "task.read", "deadline": int(time.time() * 1000) + 60000,
    "protocol_version": {"major": 1, "minor": 19}, "session_id": f"region-{outcome}", "offered_capabilities": capabilities,
}})
_, hello = receive()
version = hello["result"]["protocol_version"]
assert version == ({"major": 1, "minor": 18} if outcome == "unsupported" else {"major": 1, "minor": 19})

result = {"outcome": outcome, "frames": 0, "chunks": 0, "largest_frame_bytes": 0, "attachment_bytes": 0,
          "hash_matches": False, "input_references_attachment": False, "input_content_nonempty": False,
          "attachment_content_recorded": False, "passed": False}
captured = bytearray()


def serve():
    attachment = None
    expected_hash = None
    expected_bytes = None
    sequence = 0
    while True:
        raw, message = receive()
        result["frames"] += 1
        result["largest_frame_bytes"] = max(result["largest_frame_bytes"], len(raw) - 1)
        method = message["method"]
        params = message["params"]
        if method == "agent.attachment.begin":
            attachment = params["attachment_id"]
            expected_hash = params["sha256"]
            expected_bytes = params["byte_length"]
            assert params["session_id"] == f"region-{outcome}" and params["mime"] == "image/png"
        elif method == "agent.attachment.chunk":
            assert params["attachment_id"] == attachment and params["sequence"] == sequence
            captured.extend(base64.b64decode(params["data_base64"], validate=True)); sequence += 1; result["chunks"] += 1
        elif method == "agent.attachment.finish":
            result["attachment_bytes"] = len(captured)
            result["hash_matches"] = len(captured) == expected_bytes and hashlib.sha256(captured).hexdigest() == expected_hash
            send({"jsonrpc": "2.0", "id": message["id"], "result": {"accepted": result["hash_matches"]}})
        elif method == "agent.input":
            result["input_references_attachment"] = params.get("attachment_id") == attachment
            result["input_content_nonempty"] = bool(params.get("content", "").strip())
            if outcome == "unknown": send({"jsonrpc": "2.0", "id": "wrong-id", "result": {"accepted": True}})
            else: send({"jsonrpc": "2.0", "id": message["id"], "result": {"accepted": outcome == "accepted"}})
            return


worker = None if outcome == "unsupported" else threading.Thread(target=serve)
if worker: worker.start()
environment = os.environ | {"YONDA_SUBMIT_OUTCOME": outcome}
native = subprocess.run(["swift", str(root / "apps/desktop/check-region-preview-macos.swift")], env=environment, capture_output=True, text=True, timeout=45)
if worker: worker.join(timeout=5)
if outcome == "unsupported":
    stream.settimeout(1)
    try: unexpected = stream.recv(1)
    except socket.timeout: unexpected = b""
    result["no_attachment_frames"] = not unexpected
else:
    result["worker_finished"] = not worker.is_alive()
for candidate in path.parent.iterdir():
    if candidate.is_file():
        try:
            contents = candidate.read_bytes()
            result["attachment_content_recorded"] |= b"explain selection" in contents or bool(captured) and bytes(captured) in contents
        except OSError:
            pass
result["native_passed"] = native.returncode == 0
result["passed"] = not result["attachment_content_recorded"] and result["native_passed"] and (result.get("no_attachment_frames", False) if outcome == "unsupported" else result["worker_finished"] and result["hash_matches"] and result["input_references_attachment"] and result["input_content_nonempty"] and result["largest_frame_bytes"] <= 65536 and 0 < result["attachment_bytes"] <= 4 * 1024 * 1024)
(output / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
print(json.dumps(result, ensure_ascii=False))
reader.close(); stream.close()
raise SystemExit(0 if result["passed"] else 1)
