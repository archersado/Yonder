#!/usr/bin/env python3
"""只读验证隔离正式宿主的 1.22/1.23 定位历史投影。"""
import json
import pathlib
import socket
import sys
import time

if len(sys.argv) != 4:
    raise SystemExit("用法：脚本 <隔离 agent.sock> <fixture task_id> <全新证据目录>")
socket_path = pathlib.Path(sys.argv[1]).resolve()
task_id = sys.argv[2]
output = pathlib.Path(sys.argv[3]).resolve()
if socket_path.name != "agent.sock" or socket_path.parent.name != "com.yonder.focus.fixture" or not task_id.startswith("task_") or output.exists():
    raise SystemExit("仅允许全新定位夹具路径、任务和证据目录")

stream = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
stream.settimeout(20)
stream.connect(str(socket_path))
channel = stream.makefile("rwb", buffering=0)
counter = 0

def call(method, **params):
    global counter
    counter += 1
    request = {"jsonrpc": "2.0", "id": f"focus-{counter}", "method": method,
               "params": {"agent_id": "fixture-agent", "capability": "task.read",
                          "deadline": int(time.time() * 1000) + 20000, **params}}
    channel.write(json.dumps(request, ensure_ascii=False).encode() + b"\n")
    response = json.loads(channel.readline())
    if "error" in response:
        raise RuntimeError(response["error"])
    return response["result"]

try:
    versions = {}
    for minor in (22, 23):
        hello = call("gateway.hello", protocol_version={"major": 1, "minor": minor})
        events = call("task.events", task_id=task_id, after_sequence="0", limit=100)["events"]
        versions[str(minor)] = {"negotiated_minor": hello["protocol_version"]["minor"],
                                "events": len(events),
                                "focus_facts": [{"sequence": event["sequence"], **event["focus_event"]}
                                                for event in events if "focus_event" in event]}
    facts = versions["23"]["focus_facts"]
    passed = (versions["22"]["negotiated_minor"] == 22 and versions["23"]["negotiated_minor"] == 23
              and not versions["22"]["focus_facts"]
              and [(fact["sequence"], fact["phase"], fact["control_id"], fact.get("failure")) for fact in facts]
              == [("7", "locating", "control_3", None),
                  ("8", "focused", "control_3", None)])
    report = {"formal_tauri_host": True, "isolated_socket": str(socket_path), "task_id": task_id,
              "versions": versions, "passed": passed}
    output.mkdir(parents=True)
    (output / "result.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(report, ensure_ascii=False))
    raise SystemExit(0 if passed else 1)
finally:
    channel.close()
    stream.close()
