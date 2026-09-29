#!/usr/bin/env python3
"""在正式Yonder Gateway中制造无副作用交回，并观察慢脑是否提交新计划。"""

import argparse
import json
import os
import pathlib
import subprocess
import time


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--yonder", type=pathlib.Path, required=True)
    parser.add_argument("--wait-seconds", type=float, default=45)
    args = parser.parse_args()
    process = subprocess.Popen(
        [str(args.yonder), "mcp"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True,
        env={**os.environ, "YONDER_AGENT_ID": "codex-cli"},
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
        rpc("initialize", {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "ex-s2-replan-wake", "version": "1"}})
        process.stdin.write('{"jsonrpc":"2.0","method":"notifications/initialized"}\n')
        process.stdin.flush()
        created = tool("task_create", {
            "idempotency_key": f"ex-s2-replan-wake-{time.time_ns()}",
            "name": "EX-S2 慢脑重新规划链路验证",
            "description": "用不存在的应用目标验证CUA失败交回、慢脑唤醒与新计划展示",
        })["task"]
        submitted = tool("task_plan_submit", {
            "task_id": created["task_id"],
            "expected_sequence": created["sequence"],
            "plan_id": "replan-wake-initial",
            "plan_version": 1,
            "token_budget": 20,
            "slots": [{
                "step_id": "launch-missing-application",
                "label": "验证失效目标并交回慢脑",
                "candidates": [{
                    "candidate_id": "launch-missing-application",
                    "tool_name": "launch_app",
                    "arguments": {"bundle_id": "com.yonder.verification.DoesNotExist"},
                    "action_kind": "launch-application",
                    "target_ref": "missing-application",
                    "preconditions": [{"fact": "application-ready", "expected": True}],
                    "expected_observe": [{"fact": "application-ready", "expected": True}],
                }],
            }],
        })
        executed = tool("task_plan_execute", {
            "task_id": created["task_id"],
            "expected_sequence": submitted["sequence"],
            "plan_id": "replan-wake-initial",
            "plan_version": 1,
        })
        print(json.dumps({"checkpoint": "handback", "task_id": created["task_id"], "executed": executed}, ensure_ascii=False), flush=True)
        deadline = time.monotonic() + args.wait_seconds
        latest = tool("task_get", {"task_id": created["task_id"]})["task"]
        while time.monotonic() < deadline:
            current = tool("task_get", {"task_id": created["task_id"]})["task"]
            latest = current
            if int(current["sequence"]) > int(executed["sequence"]):
                break
            time.sleep(0.5)
        events = tool("task_events", {"task_id": created["task_id"], "after_sequence": executed["sequence"], "limit": 20})["events"]
        replanned = int(latest["sequence"]) > int(executed["sequence"])
        print(json.dumps({"passed": executed["disposition"] == "handback" and replanned, "task_id": created["task_id"], "handback_sequence": executed["sequence"], "latest": latest, "events_after_handback": events}, ensure_ascii=False), flush=True)
        return 0 if executed["disposition"] == "handback" and replanned else 1
    finally:
        if process.poll() is None:
            process.stdin.close()
            process.stdout.close()
            process.wait(timeout=5)


if __name__ == "__main__":
    raise SystemExit(main())
