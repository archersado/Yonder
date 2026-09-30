#!/usr/bin/env python3
"""经正式Yonder MCP提交通用多步骤CUA片段，不直连Socket或Driver。"""
import json
import os
import pathlib
import subprocess
import uuid

root = pathlib.Path(__file__).resolve().parents[2]
cli = root / "target/debug/Yonda.app/Contents/MacOS/yonder"
environment = dict(os.environ, YONDER_AGENT_ID="codex-cli")
process = subprocess.Popen(
    [str(cli), "mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, env=environment
)
counter = 0
task_id = None


def rpc(method, params):
    global counter
    counter += 1
    process.stdin.write(json.dumps({"jsonrpc": "2.0", "id": counter, "method": method, "params": params}, ensure_ascii=False) + "\n")
    process.stdin.flush()
    response = json.loads(process.stdout.readline())
    if "error" in response:
        raise RuntimeError(response["error"]["message"])
    return response["result"]


def tool(name, arguments):
    result = rpc("tools/call", {"name": name, "arguments": arguments})
    content = json.loads(result["content"][0]["text"])
    if result.get("isError"):
        raise RuntimeError(content.get("message", "Yonder tool failed"))
    return content


def candidate(identifier, tool_name, action_kind, arguments, before, after):
    return {
        "candidate_id": identifier,
        "tool_name": tool_name,
        "arguments": arguments,
        "action_kind": action_kind,
        "target_ref": "qqmusic-window",
        "preconditions": [{"fact": before, "expected": True}],
        "expected_observe": [{"fact": after, "expected": True}],
    }


try:
    rpc("initialize", {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "yonder-generic-plan-verification", "version": "1"}})
    tools = rpc("tools/list", {})["tools"]
    plan_tool = next(item for item in tools if item["name"] == "task_plan_submit")
    action_kinds = plan_tool["inputSchema"]["properties"]["slots"]["items"]["properties"]["candidates"]["items"]["properties"]["action_kind"]["enum"]
    required_kinds = {"focus-control", "input-text", "activate-control"}
    if not required_kinds.issubset(action_kinds):
        raise RuntimeError("正式MCP未公开协议1.40通用桌面动作")

    created = tool("task_create", {
        "idempotency_key": f"qqmusic-generic-plan-{uuid.uuid4().hex}",
        "name": "QQ音乐搜索播放验证",
        "description": "使用一个慢脑计划片段搜索并尝试播放指定歌曲",
    })["task"]
    task_id, sequence = created["task_id"], created["sequence"]
    plan_id = f"qqmusic-plan-{uuid.uuid4().hex}"
    slots = [
        {"step_id": "launch", "label": "打开QQ音乐", "candidates": [candidate("launch", "launch_app", "launch-application", {"bundle_id": "com.tencent.QQMusicMac"}, "application-ready", "application-ready")]},
        {"step_id": "focus-search", "label": "聚焦歌曲搜索框", "candidates": [candidate("focus-search", "hotkey", "focus-control", {"keys": ["cmd", "f"]}, "application-ready", "target-resolved")]},
        {"step_id": "enter-query", "label": "输入歌曲名称", "candidates": [candidate("enter-query", "type_text", "input-text", {"text": "one last kiss"}, "target-resolved", "target-resolved")]},
        {"step_id": "search", "label": "提交歌曲搜索", "candidates": [candidate("search", "press_key", "activate-control", {"key": "ENTER"}, "target-resolved", "target-resolved")]},
        {"step_id": "play", "label": "播放搜索结果", "candidates": [candidate("play", "press_key", "activate-control", {"key": "ENTER"}, "target-resolved", "target-resolved")]},
    ]
    submitted = tool("task_plan_submit", {
        "task_id": task_id,
        "expected_sequence": sequence,
        "plan_id": plan_id,
        "plan_version": 1,
        "token_budget": 800,
        "slots": slots,
    })
    executed = tool("task_plan_execute", {
        "task_id": task_id,
        "expected_sequence": submitted["sequence"],
        "plan_id": plan_id,
        "plan_version": 1,
    })
    snapshot = tool("task_get", {"task_id": task_id})["task"]
    print(json.dumps({
        "task_id": task_id,
        "status": snapshot["status"],
        "sequence": snapshot["sequence"],
        "submitted_steps": len(slots),
        "plan_disposition": executed["disposition"],
        "handoff_reason": executed.get("handoff_reason"),
        "observation_available": executed.get("observation") is not None,
        "generic_actions_advertised": True,
    }, ensure_ascii=False))
finally:
    process.terminate()
    process.wait(timeout=5)
