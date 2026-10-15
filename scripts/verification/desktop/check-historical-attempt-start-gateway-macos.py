#!/usr/bin/env python3
"""只读验证隔离正式宿主的 1.24/1.25 尝试开始历史投影。"""
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
if socket_path.name != "agent.sock" or socket_path.parent.name != "com.yonder.attempt-start.fixture" or not task_id.startswith("task_") or output.exists():
    raise SystemExit("仅允许全新尝试开始夹具路径、任务和证据目录")

stream = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
stream.settimeout(20)
stream.connect(str(socket_path))
channel = stream.makefile("rwb", buffering=0)
counter = 0

def call(method, **params):
    global counter
    counter += 1
    request = {"jsonrpc": "2.0", "id": f"attempt-start-{counter}", "method": method,
               "params": {"agent_id": "fixture-agent", "capability": "task.read",
                          "deadline": int(time.time() * 1000) + 20000, **params}}
    channel.write(json.dumps(request, ensure_ascii=False).encode() + b"\n")
    response = json.loads(channel.readline())
    if "error" in response:
        raise RuntimeError(response["error"])
    return response["result"]

try:
    versions = {}
    for minor in (24, 25):
        hello = call("gateway.hello", protocol_version={"major": 1, "minor": minor})
        events = call("task.events", task_id=task_id, after_sequence="0", limit=100)["events"]
        versions[str(minor)] = {
            "negotiated_minor": hello["protocol_version"]["minor"],
            "start_facts": [{"sequence": event["sequence"], **event["attempt_started"]}
                            for event in events if "attempt_started" in event],
            "result_sequences": [event["sequence"] for event in events if "attempt_result" in event],
        }
    starts = versions["25"]["start_facts"]
    passed = (
        versions["24"]["negotiated_minor"] == 24
        and versions["25"]["negotiated_minor"] == 25
        and not versions["24"]["start_facts"]
        and starts == [{"sequence": "3", "step_id": "step-one", "attempt_id": "attempt-one",
                        "worker_instance_id": "worker-one", "host_session_id": "host-one"}]
        and versions["25"]["result_sequences"] == ["4"]
    )
    report = {"formal_tauri_host": True, "isolated_socket": str(socket_path), "task_id": task_id,
              "versions": versions, "start_and_result_are_distinct": True,
              "payload_exposed": False, "passed": passed}
    output.mkdir(parents=True)
    (output / "result.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(report, ensure_ascii=False))
    raise SystemExit(0 if passed else 1)
finally:
    channel.close()
    stream.close()
