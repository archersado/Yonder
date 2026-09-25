#!/usr/bin/env python3
"""只读验证隔离 Tauri 宿主的本地 Gateway 历史 Observe 协议。"""
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
if socket_path.name != "agent.sock" or socket_path.parent.name != "com.yonder.observation.fixture":
    raise SystemExit("只允许隔离宿主的 agent.sock")
if not task_id.startswith("task_") or output.exists():
    raise SystemExit("任务 ID 无效或证据目录已存在")

stream = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
stream.settimeout(20)
stream.connect(str(socket_path))
channel = stream.makefile("rwb", buffering=0)
counter = 0

def call(method, **params):
    global counter
    counter += 1
    request = {
        "jsonrpc": "2.0", "id": f"observe-{counter}", "method": method,
        "params": {"agent_id": "fixture-agent", "capability": "task.read",
                   "deadline": int(time.time() * 1000) + 20000, **params},
    }
    channel.write(json.dumps(request, ensure_ascii=False).encode() + b"\n")
    response = json.loads(channel.readline())
    if "error" in response:
        raise RuntimeError(response["error"])
    return response["result"]

try:
    versions = {}
    for minor in (20, 21):
        hello = call("gateway.hello", protocol_version={"major": 1, "minor": minor})
        events = call("task.events", task_id=task_id, after_sequence="0", limit=100)["events"]
        versions[str(minor)] = {
            "negotiated_minor": hello["protocol_version"]["minor"],
            "events": len(events),
            "observations": [event["observation"] for event in events if "observation" in event],
        }
    historical = versions["21"]["observations"]
    passed = (
        versions["20"]["negotiated_minor"] == 20
        and versions["21"]["negotiated_minor"] == 21
        and not versions["20"]["observations"]
        and [(item["step_id"], item["result"], item["summary"]) for item in historical]
        == [("step-one", "matched", "目标已打开"), ("step-two", "unknown", "核实超时，结果未知")]
    )
    report = {"formal_tauri_host": True, "isolated_socket": str(socket_path),
              "task_id": task_id, "versions": versions, "passed": passed}
    output.mkdir(parents=True)
    (output / "result.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(report, ensure_ascii=False))
    raise SystemExit(0 if passed else 1)
finally:
    channel.close()
    stream.close()
