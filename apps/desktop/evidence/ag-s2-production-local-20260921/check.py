#!/usr/bin/env python3
"""真实macOS生产UDS下的AG-S2任务登记验证；保存结构化结果。"""
import json, os, pathlib, subprocess, sys, time

root = pathlib.Path(__file__).resolve().parents[4]
home = pathlib.Path(os.environ["YONDER_HOME"])
binary = root / "target/debug/yonder"
result_path = pathlib.Path(__file__).with_name("result.json")
agent_id = "ag-s2-owner"
other_agent_id = "ag-s2-other"

def start(agent):
    return subprocess.Popen([str(binary), "mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True,
                            env={**os.environ, "HOME": str(home), "YONDER_AGENT_ID": agent})

def call(process, number, name, arguments):
    process.stdin.write(json.dumps({"jsonrpc":"2.0","id":number,"method":"tools/call","params":{"name":name,"arguments":arguments}}, ensure_ascii=False) + "\n")
    process.stdin.flush()
    response = json.loads(process.stdout.readline())
    assert response["id"] == number
    result = response["result"]
    if result["isError"]:
        return {"error": json.loads(result["content"][0]["text"])}
    value = json.loads(result["content"][0]["text"])
    print(json.dumps({"probe": number, "value": value}, ensure_ascii=False))
    return value

def mcp(agent):
    process = start(agent)
    process.stdin.write(json.dumps({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"ag-s2-check","version":"1"}}}) + "\n")
    process.stdin.flush()
    response = json.loads(process.stdout.readline())
    assert response["result"]["protocolVersion"] == "2025-06-18"
    process.stdin.write('{"jsonrpc":"2.0","method":"notifications/initialized"}\n')
    process.stdin.flush()
    return process

owner = mcp(agent_id)
other = mcp(other_agent_id)
try:
    key = f"ag-s2-production-{time.time_ns()}"
    created_response = call(owner, 1, "task_create", {"idempotency_key": key, "name": "生产登记幂等验证", "description": "验证UDS首帧身份下的任务登记"})
    print(json.dumps({"created_response": created_response}, ensure_ascii=False), flush=True)
    created = created_response["task"]
    assert created["owner_agent_id"] == agent_id and created["status"] == "created"
    repeated = call(owner, 2, "task_create", {"idempotency_key": key, "name": "生产登记幂等验证", "description": "验证UDS首帧身份下的任务登记"})["task"]
    assert repeated == created
    conflict = call(owner, 3, "task_create", {"idempotency_key": key, "name": "生产登记幂等验证", "description": "不同内容必须冲突"})
    assert conflict.get("error", {}).get("code") == -32009
    events = call(owner, 4, "task_events", {"task_id": created["task_id"], "after_sequence": 0})
    assert len(events["events"]) == 1, events
    other_read = call(other, 5, "task_get", {"task_id": created["task_id"]})
    assert other_read.get("error", {}).get("code") == -32004, other_read
    cancelled = call(owner, 6, "task_cancel", {"task_id": created["task_id"], "expected_sequence": created["sequence"]})["task"]
    assert cancelled["status"] == "cancelled"
    other_cancel = call(other, 7, "task_cancel", {"task_id": created["task_id"], "expected_sequence": cancelled["sequence"]})
    assert other_cancel.get("error", {}).get("code") == -32004, other_cancel
    result = {
        "passed": True,
        "protocol_version": "2025-06-18",
        "owner_agent_id": created["owner_agent_id"],
        "task_id": created["task_id"],
        "same_content_same_task_id": repeated["task_id"] == created["task_id"],
        "conflict_code": conflict["error"]["code"],
        "events_after_repeat": len(events["events"]),
        "other_agent_get_code": other_read["error"]["code"],
        "cancelled_status": cancelled["status"],
        "other_agent_cancel_code": other_cancel["error"]["code"],
    }
    result_path.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(result, ensure_ascii=False, indent=2))
finally:
    for process in (owner, other):
        process.stdin.close()
        process.stdout.close()
        process.wait(timeout=5)
