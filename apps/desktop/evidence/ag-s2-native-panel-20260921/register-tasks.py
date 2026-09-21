#!/usr/bin/env python3
"""通过生产UDS/MCP登记原生面板验证任务；保存结构化结果。"""
import json, os, pathlib, subprocess, time

root = pathlib.Path(__file__).resolve().parents[4]
home = pathlib.Path(os.environ.get("YONDER_HOME", pathlib.Path.home()))
binary = root / "target/debug/yonder"
output = pathlib.Path(__file__).with_name("agent-result.json")
agent_id = "local-test-agent"

def call(process, number, name, arguments):
    process.stdin.write(json.dumps({"jsonrpc":"2.0","id":number,"method":"tools/call","params":{"name":name,"arguments":arguments}}, ensure_ascii=False) + "\n")
    process.stdin.flush()
    response = json.loads(process.stdout.readline())
    result = response["result"]
    if result["isError"]:
        raise RuntimeError(json.loads(result["content"][0]["text"]))
    return json.loads(result["content"][0]["text"])

with subprocess.Popen([str(binary), "mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True,
                      env={"HOME": str(home), "PATH": "/usr/bin:/bin", "YONDER_AGENT_ID": agent_id}) as process:
    call(process, 0, "task_list", {"include_finished": True})
    tasks = []
    for number, key, name, description in [
        (1, f"ag-s2-panel-{time.time_ns()}-one", "整理桌面测试", "生产UDS登记，用于原生面板悬停验证"),
        (2, f"ag-s2-panel-{time.time_ns()}-two", "检查文档测试", "生产UDS登记，用于原生详情验证"),
    ]:
        task = call(process, number, "task_create", {"idempotency_key": key, "name": name, "description": description})["task"]
        assert task["owner_agent_id"] == agent_id and task["status"] == "created"
        tasks.append(task)
    assert len({task["task_id"] for task in tasks}) == 2
    result = {"passed": True, "transport": "production-uds-mcp", "owner_agent_id": agent_id, "registered_tasks": 2,
              "created_only": all(task["status"] == "created" for task in tasks), "task_ids": [task["task_id"] for task in tasks]}
    output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(result, ensure_ascii=False, indent=2))
