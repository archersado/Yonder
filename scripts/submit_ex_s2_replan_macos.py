#!/usr/bin/env python3
"""慢脑经正式MCP/Gateway读取交回事实并提交新的受限计划片段。"""

import argparse
import json
import os
import pathlib
import subprocess


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--yonder", type=pathlib.Path, required=True)
    parser.add_argument("--task-id", required=True)
    parser.add_argument("--execute-existing", action="store_true")
    parser.add_argument("--execute-after-submit", action="store_true")
    parser.add_argument("--inspect-only", action="store_true")
    parser.add_argument("--cancel", action="store_true")
    parser.add_argument("--plan-version", type=int, default=1)
    args = parser.parse_args()
    process = subprocess.Popen(
        [str(args.yonder), "mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
        text=True, env={**os.environ, "YONDER_AGENT_ID": "codex-cli"},
    )
    request_id = 0

    def rpc(method: str, params: dict) -> dict:
        nonlocal request_id
        request_id += 1
        process.stdin.write(json.dumps({"jsonrpc": "2.0", "id": request_id, "method": method, "params": params}, ensure_ascii=False) + "\n")
        process.stdin.flush()
        response = json.loads(process.stdout.readline())
        if "error" in response:
            raise RuntimeError(response)
        return response["result"]

    def tool(name: str, arguments: dict) -> dict:
        result = rpc("tools/call", {"name": name, "arguments": arguments})
        payload = json.loads(result["content"][0]["text"])
        if result.get("isError"):
            raise RuntimeError(payload)
        return payload

    try:
        rpc("initialize", {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "ex-s2-slow-brain-replan", "version": "1"}})
        process.stdin.write('{"jsonrpc":"2.0","method":"notifications/initialized"}\n')
        process.stdin.flush()
        snapshot = tool("task_get", {"task_id": args.task_id})["task"]
        events = tool("task_events", {"task_id": args.task_id, "after_sequence": "0", "limit": 100})["events"]
        if args.inspect_only:
            print(json.dumps({"task": snapshot, "events": events}, ensure_ascii=False))
            return 0
        if args.cancel:
            cancelled = tool("task_cancel", {"task_id": args.task_id, "expected_sequence": snapshot["sequence"]})["task"]
            print(json.dumps({"task": snapshot, "events_read": len(events), "cancelled": cancelled}, ensure_ascii=False))
            return 0
        if args.execute_existing:
            executed = tool("task_plan_execute", {
                "task_id": args.task_id,
                "expected_sequence": snapshot["sequence"],
                "plan_id": "replan-wake-safe-followup",
                "plan_version": args.plan_version,
            })
            print(json.dumps({"task": snapshot, "events_read": len(events), "executed": executed}, ensure_ascii=False))
            return 0
        if snapshot.get("next_intent") != "需要慢脑重新 Observe 或规划":
            raise RuntimeError("任务不在重新规划边界")
        submitted = tool("task_plan_submit", {
            "task_id": args.task_id,
            "expected_sequence": snapshot["sequence"],
            "plan_id": "replan-wake-safe-followup",
            "plan_version": args.plan_version,
            "token_budget": 40,
            "slots": [
                {
                    "step_id": "replan-launch-textedit",
                    "label": "重新规划：打开安全验证窗口",
                    "candidates": [{
                        "candidate_id": "launch-textedit",
                        "tool_name": "launch_app",
                        "arguments": {"bundle_id": "com.apple.TextEdit"},
                        "action_kind": "launch-application",
                        "target_ref": "textedit-application",
                        "preconditions": [{"fact": "application-ready", "expected": True}],
                        "expected_observe": [{"fact": "application-ready", "expected": True}],
                    }],
                },
                {
                    "step_id": "replan-advance-focus",
                    "label": "重新规划：核验窗口并推进焦点",
                    "candidates": [{
                        "candidate_id": "advance-textedit-focus",
                        "tool_name": "press_key",
                        "arguments": {"key": "tab", "delivery_mode": "background"},
                        "action_kind": "resolve-conversation",
                        "target_ref": "textedit-window",
                        "preconditions": [{"fact": "target-resolved", "expected": True}],
                        "expected_observe": [{"fact": "target-resolved", "expected": True}],
                    }],
                },
            ],
        })
        result = {"task": snapshot, "events_read": len(events), "replan": submitted}
        if args.execute_after_submit:
            result["executed"] = tool("task_plan_execute", {
                "task_id": args.task_id,
                "expected_sequence": submitted["sequence"],
                "plan_id": "replan-wake-safe-followup",
                "plan_version": args.plan_version,
            })
        print(json.dumps(result, ensure_ascii=False))
        return 0
    finally:
        if process.poll() is None:
            process.stdin.close()
            process.stdout.close()
            process.wait(timeout=5)


if __name__ == "__main__":
    raise SystemExit(main())
