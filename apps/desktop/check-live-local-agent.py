#!/usr/bin/env python3
"""本地测试Agent启动正式Debug小龙，真实登记任务；EOF后保留小龙供用户检查。"""
import json
import pathlib
import select
import subprocess
import time
import sys

root = pathlib.Path(__file__).resolve().parents[2]
output = pathlib.Path(sys.argv[1])
output.mkdir(parents=True, exist_ok=False)
binary = root / "apps/desktop/target/preview/Yonda Task Space.app/Contents/MacOS/yonder-desktop"
agent = subprocess.Popen([str(binary), "--local-agent-stdio"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, start_new_session=True)
counter = 0
named = "--named" in sys.argv[2:]


def request(method, capability="task.read", **fields):
    global counter
    counter += 1
    request_id = f"local-agent-{counter}"
    message = {"jsonrpc": "2.0", "id": request_id, "method": method, "params": {"agent_id": "local-test-agent", "capability": capability, "deadline": int(time.time() * 1000) + 20000, **fields}}
    agent.stdin.write(json.dumps(message).encode() + b"\n")
    agent.stdin.flush()
    assert select.select([agent.stdout], [], [], 20)[0], "正式Gateway响应超时"
    response = json.loads(agent.stdout.readline(65537))
    assert response["id"] == request_id
    return response


try:
    assert request("task.list", limit=100)["error"]["code"] == -32002
    minor = 3 if named else 1
    assert request("gateway.hello", protocol_version={"major": 1, "minor": minor})["result"]["protocol_version"]["minor"] == minor
    tasks = []
    for key, description in [("desktop-local-test-one", "本地Agent桌面接入测试一"), ("desktop-local-test-two", "本地Agent桌面接入测试二")]:
        fields = {"name": "整理桌面测试" if key.endswith("one") else "检查文档测试"} if named else {}
        if named: key += "-named-v3"
        first = request("task.create", "task.create", idempotency_key=key, description=description, **fields)["result"]["task"]
        assert request("task.create", "task.create", idempotency_key=key, description=description, **fields)["result"]["task"] == first
        assert first["status"] in (["created", "cancelled"] if named else ["created"])
        if named: assert first["name"] == fields["name"]
        tasks.append(first)
    assert len({task["task_id"] for task in tasks}) == 2
    actual = request("task.list", limit=100)["result"]["tasks"]
    assert {task["task_id"] for task in tasks} <= {task["task_id"] for task in actual}
    result = {"pid": agent.pid, "live_desktop_connection": True, "registered_tasks": 2, "idempotent_replay": True, "created_only": all(task["status"] == "created" for task in tasks), "agent_names": named, "passed": True}
    (output / "agent-result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result))
    native = subprocess.run(["swift", str(root / "apps/desktop/check-local-agent-menu-macos.swift"), str(output / "native"), *(["--named"] if named else [])], timeout=45)
    assert native.returncode == 0, f"原生菜单验证失败：{native.returncode}"
    if named:
        for task in tasks:
            cancelled = request("task.cancel", "task.cancel", task_id=task["task_id"], expected_sequence=task["sequence"])["result"]["task"]
            assert cancelled["status"] == "cancelled" and cancelled["name"] == task["name"]
        result["cancelled_after_native_test"] = True
        (output / "agent-result.json").write_text(json.dumps(result, indent=2) + "\n")
finally:
    agent.stdin.close()
    agent.stdout.close()
    # EOF只结束连接，小龙仍运行；不手工删除或改写正式任务。
