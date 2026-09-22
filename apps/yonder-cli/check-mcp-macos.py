#!/usr/bin/env python3
"""真实MCP stdio→UDS→正式TaskHost验证；只保存结构化结果。"""
import json
import os
import pathlib
import subprocess
import time

root = pathlib.Path(__file__).resolve().parents[2]
binary = root / "apps/desktop/target/preview/Yonda Task Space.app/Contents/MacOS/yonder"
output = root / "apps/desktop/evidence/control-request-mcp-20260916"
output.mkdir(parents=True, exist_ok=True)
agent_id = "mcp-check"

def call(process, message):
    process.stdin.write(json.dumps(message, ensure_ascii=False) + "\n")
    process.stdin.flush()
    response = json.loads(process.stdout.readline())
    assert response["id"] == message["id"] and "result" in response
    return response["result"]

def tool(process, number, name, arguments):
    result = call(process, {"jsonrpc":"2.0","id":number,"method":"tools/call","params":{"name":name,"arguments":arguments}})
    assert not result["isError"], result
    return json.loads(result["content"][0]["text"])

with subprocess.Popen([str(binary), "mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True,
                      env={**os.environ, "YONDER_AGENT_ID": agent_id}) as process:
    initialized = call(process, {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"yonder-check","version":"1"}}})
    process.stdin.write('{"jsonrpc":"2.0","method":"notifications/initialized"}\n')
    process.stdin.flush()
    listed = call(process, {"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}})
    names = {item["name"] for item in listed["tools"]}
    expected = {"task_create","task_list","task_get","task_cancel","task_control","task_events","task_step_declare","task_step_get"}
    assert expected <= names, names
    key = f"codex-mcp-{time.time_ns()}"
    created = tool(process, 3,"task_create",{"idempotency_key":key,"name":"Codex CLI 接入验证","description":"验证MCP经本地Gateway登记任务"})["task"]
    current = tool(process, 4,"task_get",{"task_id":created["task_id"]})["task"]
    assert current == created and created["owner_agent_id"] == agent_id and created["status"] == "created"
    cancelled = tool(process, 5,"task_cancel",{"task_id":created["task_id"],"expected_sequence":created["sequence"]})["task"]
    assert cancelled["status"] == "cancelled"
    result = {"mcp_protocol":initialized["protocolVersion"],"tools":len(names),"task_control_exposed":True,"owner":created["owner_agent_id"],"created":True,"read_back":True,"cancelled_retained":True,"passed":True}
    (output / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(result, ensure_ascii=False))
