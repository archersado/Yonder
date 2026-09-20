#!/usr/bin/env python3
"""通过正式UDS与真实CUA步骤验证圈选前暂停；不保存屏幕内容。"""

import json
import os
from pathlib import Path
import socket
import sqlite3
import subprocess
import sys
import threading
import time


root = Path(__file__).resolve().parents[2]
entry = sys.argv[1]
output = Path(sys.argv[2])
output.mkdir(parents=True, exist_ok=False)
assert entry in {"pet", "tray"}
fixture = subprocess.Popen(["/private/tmp/yonda-cu-gateway-fixture"], stdout=subprocess.PIPE, text=True)
state = {}


def read_fixture():
    for line in fixture.stdout:
        try: state.update(json.loads(line))
        except json.JSONDecodeError: pass


threading.Thread(target=read_fixture, daemon=True).start()
end = time.monotonic() + 15
while not (state.get("ready") and state.get("launched") and state.get("app_active") and state.get("target_key")) and time.monotonic() < end: time.sleep(.05)
assert state.get("ready") and state.get("app_active") and state.get("target_key")
focused = subprocess.run(["/private/tmp/yonda-focus-target", str(state["pid"]), str(state["window_id"])], capture_output=True, text=True, timeout=10)
assert focused.returncode == 0 and json.loads(focused.stdout).get("focused")

stream = socket.socket(socket.AF_UNIX)
stream.settimeout(45)
stream.connect(str(Path.home() / "Library/Application Support/com.yonder.desktop/agent.sock"))
buffer = stream.makefile("rwb", buffering=0)
counter = 0


def call(method, capability, **params):
    global counter
    counter += 1
    request = {"jsonrpc": "2.0", "id": f"region-pause-{counter}", "method": method, "params": {
        "agent_id": "codex-cli", "capability": capability, "deadline": int(time.time() * 1000) + 40000, **params}}
    buffer.write(json.dumps(request, ensure_ascii=False).encode() + b"\n")
    response = json.loads(buffer.readline())
    assert "error" not in response, response
    return response["result"]


try:
    hello = call("gateway.hello", "task.read", protocol_version={"major": 1, "minor": 12})
    assert hello["protocol_version"] == {"major": 1, "minor": 12}
    created = call("task.create", "task.create", idempotency_key=f"region-pause-{entry}-{time.time_ns()}", description="验证圈选前暂停桌面任务", name=f"圈选暂停 {entry}")["task"]
    step = call("computer.step", "computer.execute", task_id=created["task_id"], expected_sequence=created["sequence"], step_id="inspect", label="完成可暂停桌面步骤", tool_name="get_window_state", arguments={"include_screenshot": False, "max_elements": 10})
    assert step["status"] == "running" and step["action_succeeded"] is True, step
    native = subprocess.run(["swift", str(root / "apps/desktop/check-region-preview-macos.swift")], env=os.environ | {"YONDA_EXPECT_DESKTOP_PAUSE": entry}, capture_output=True, text=True, timeout=30)
    task = call("task.get", "task.read", task_id=created["task_id"])["task"]
    db = sqlite3.connect(Path.home() / "Library/Application Support/com.yonder.desktop/tasks.db")
    control = db.execute("SELECT kind,phase,focus_phase FROM task_controls WHERE task_id=?", (created["task_id"],)).fetchone()
    attempts = db.execute("SELECT count(*) FROM task_attempts WHERE task_id=?", (created["task_id"],)).fetchone()[0]
    events = db.execute("SELECT count(*),max(sequence) FROM events WHERE task_id=?", (created["task_id"],)).fetchone()
    outbox = db.execute("SELECT count(*),max(sequence) FROM outbox WHERE task_id=?", (created["task_id"],)).fetchone()
    db.close()
    sequence = int(task["sequence"])
    atomic_history = events == (sequence, sequence) and outbox == (sequence, sequence)
    result = {"entry": entry, "native_passed": native.returncode == 0, "task_status": task["status"],
              "control_is_pause_stopped": control == ("pause", "stopped", None), "attempts": attempts,
              "events": events[0], "outbox": outbox[0], "sequence": sequence,
              "atomic_history": atomic_history,
              "selection_overlay_opened": native.returncode == 0, "recording_started": False,
              "focus_started": control is not None and control[2] is not None, "passed": False}
    result["passed"] = result["native_passed"] and result["task_status"] == "paused" and result["control_is_pause_stopped"] and attempts == 1 and atomic_history and not result["focus_started"]
    (output / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(result, ensure_ascii=False))
    raise SystemExit(0 if result["passed"] else 1)
finally:
    buffer.close(); stream.close(); fixture.terminate(); fixture.wait(timeout=5)
