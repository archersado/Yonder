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
assert entry in {"pet", "tray", "failure"}
fixture = subprocess.Popen(["/private/tmp/yonda-cu-gateway-fixture"], stdout=subprocess.PIPE, text=True)
state = {}


def read_fixture():
    for line in fixture.stdout:
        try: state.update(json.loads(line))
        except json.JSONDecodeError: pass


threading.Thread(target=read_fixture, daemon=True).start()
end = time.monotonic() + 15
while not (state.get("ready") and state.get("launched") and state.get("window_id")) and time.monotonic() < end: time.sleep(.05)
assert state.get("ready") and state.get("launched") and state.get("window_id"), state
stream = socket.socket(socket.AF_UNIX)
stream.settimeout(45)
stream.connect(str(Path.home() / "Library/Application Support/com.yonder.desktop/agent.sock"))
buffer = stream.makefile("rwb", buffering=0)
counter = 0


def exchange(method, capability, **params):
    global counter
    counter += 1
    request = {"jsonrpc": "2.0", "id": f"region-pause-{counter}", "method": method, "params": {
        "agent_id": "codex-cli", "capability": capability, "deadline": int(time.time() * 1000) + 40000, **params}}
    buffer.write(json.dumps(request, ensure_ascii=False).encode() + b"\n")
    return json.loads(buffer.readline())


def call(method, capability, **params):
    response = exchange(method, capability, **params)
    assert "error" not in response, response
    return response["result"]


try:
    hello = call("gateway.hello", "task.read", protocol_version={"major": 1, "minor": 12})
    assert hello["protocol_version"] == {"major": 1, "minor": 12}
    for trial in range(10):
        created = call("task.create", "task.create", idempotency_key=f"region-pause-{entry}-{time.time_ns()}", description="验证圈选前暂停桌面任务", name=f"圈选暂停 {entry}")["task"]
        step = call("computer.step", "computer.execute", task_id=created["task_id"], expected_sequence=created["sequence"], step_id="unknown" if entry == "failure" else "inspect", label="验证结果待核实" if entry == "failure" else "完成可暂停桌面步骤", tool_name="missing_tool" if entry == "failure" else "get_window_state", arguments={} if entry == "failure" else {"include_screenshot": False, "max_elements": 10})
        if step["status"] != "interrupted": break
        assert step.get("unknown_reason") == "user-input", step
    else: raise AssertionError("连续真实用户输入阻断CUA验证")
    if entry == "failure":
        assert step["status"] == "running" and step["action_succeeded"] is None, step
        native = subprocess.run(["swift", str(root / "apps/desktop/check-region-preview-macos.swift")], env=os.environ | {"YONDA_EXPECT_DESKTOP_STOP_UNCONFIRMED": "1"}, capture_output=True, text=True, timeout=30)
        task = call("task.get", "task.read", task_id=created["task_id"])["task"]
        probe = call("task.create", "task.create", idempotency_key=f"region-pause-probe-{time.time_ns()}", description="验证桌面租约仍被占用", name="圈选暂停租约探针")["task"]
        blocked = exchange("computer.step", "computer.execute", task_id=probe["task_id"], expected_sequence=probe["sequence"], step_id="blocked", label="不得取得桌面租约", tool_name="get_window_state", arguments={"include_screenshot": False, "max_elements": 10})
        db = sqlite3.connect(Path.home() / "Library/Application Support/com.yonder.desktop/tasks.db")
        control = db.execute("SELECT kind,phase,focus_phase FROM task_controls WHERE task_id=?", (created["task_id"],)).fetchone()
        events = db.execute("SELECT count(*),max(sequence) FROM events WHERE task_id=?", (created["task_id"],)).fetchone()
        outbox = db.execute("SELECT count(*),max(sequence) FROM outbox WHERE task_id=?", (created["task_id"],)).fetchone()
        recording_schema = db.execute("SELECT count(*) FROM sqlite_master WHERE type='table' AND name LIKE 'recording%'").fetchone()[0]
        db.close()
        sequence = int(task["sequence"])
        native_result = json.loads(native.stdout) if native.returncode == 0 else {}
        blocked_error = blocked.get("error", {})
        result = {"entry": entry, "native_passed": native.returncode == 0,
                  "task_status": task["status"], "control_is_pause_pending": control == ("pause", "pending", None),
                  "selection_overlay_opened": native_result.get("selection_overlay_opened"),
                  "lease_retained": blocked_error.get("code") == -32012,
                  "blocked_error_code": blocked_error.get("code"), "events": events[0], "outbox": outbox[0], "sequence": sequence,
                  "atomic_history": events == (sequence, sequence) and outbox == (sequence, sequence),
                  "recording_schema_present": recording_schema != 0,
                  "focus_started": control is not None and control[2] is not None, "passed": False}
        result["passed"] = result["native_passed"] and result["task_status"] == "running" and result["control_is_pause_pending"] and result["selection_overlay_opened"] is False and result["lease_retained"] and result["atomic_history"] and not result["recording_schema_present"] and not result["focus_started"]
        (output / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
        print(json.dumps(result, ensure_ascii=False))
        raise SystemExit(0 if result["passed"] else 1)
    assert step["status"] == "running" and step["action_succeeded"] is True, step
    native = subprocess.run(["swift", str(root / "apps/desktop/check-region-preview-macos.swift")], env=os.environ | {"YONDA_EXPECT_DESKTOP_PAUSE": entry}, capture_output=True, text=True, timeout=30)
    task = call("task.get", "task.read", task_id=created["task_id"])["task"]
    db = sqlite3.connect(Path.home() / "Library/Application Support/com.yonder.desktop/tasks.db")
    control = db.execute("SELECT kind,phase,focus_phase FROM task_controls WHERE task_id=?", (created["task_id"],)).fetchone()
    attempts = db.execute("SELECT count(*) FROM task_attempts WHERE task_id=?", (created["task_id"],)).fetchone()[0]
    events = db.execute("SELECT count(*),max(sequence) FROM events WHERE task_id=?", (created["task_id"],)).fetchone()
    outbox = db.execute("SELECT count(*),max(sequence) FROM outbox WHERE task_id=?", (created["task_id"],)).fetchone()
    recording_schema = db.execute("SELECT count(*) FROM sqlite_master WHERE type='table' AND name LIKE 'recording%'").fetchone()[0]
    db.close()
    sequence = int(task["sequence"])
    atomic_history = events == (sequence, sequence) and outbox == (sequence, sequence)
    result = {"entry": entry, "native_passed": native.returncode == 0, "task_status": task["status"],
              "control_is_pause_stopped": control == ("pause", "stopped", None), "attempts": attempts,
              "events": events[0], "outbox": outbox[0], "sequence": sequence,
              "atomic_history": atomic_history,
              "selection_overlay_opened": native.returncode == 0, "recording_schema_present": recording_schema != 0,
              "focus_started": control is not None and control[2] is not None, "passed": False}
    result["passed"] = result["native_passed"] and result["task_status"] == "paused" and result["control_is_pause_stopped"] and attempts == 1 and atomic_history and not result["recording_schema_present"] and not result["focus_started"]
    (output / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(result, ensure_ascii=False))
    raise SystemExit(0 if result["passed"] else 1)
finally:
    buffer.close(); stream.close(); fixture.terminate(); fixture.wait(timeout=5)
