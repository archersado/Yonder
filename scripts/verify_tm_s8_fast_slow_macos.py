#!/usr/bin/env python3
"""经正式 MCP/Gateway 验证 Codex 慢脑、Jev 快脑与连续 CUA 执行。"""

import argparse
import json
import os
import pathlib
import subprocess
import time


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--yonder", type=pathlib.Path, required=True)
    parser.add_argument("--checkpoint", action="store_true")
    args = parser.parse_args()
    process = subprocess.Popen(
        [str(args.yonder), "mcp"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True,
        env={**os.environ, "YONDER_AGENT_ID": "codex-cli"},
    )

    def rpc(number: int, method: str, params: dict) -> dict:
        request = {
            "jsonrpc": "2.0",
            "id": number,
            "method": method,
            "params": params,
        }
        process.stdin.write(json.dumps(request, ensure_ascii=False) + "\n")
        process.stdin.flush()
        response = json.loads(process.stdout.readline())
        if "error" in response:
            raise RuntimeError(response)
        return response["result"]

    def tool(
        number: int, name: str, arguments: dict, *, allow_error: bool = False
    ) -> dict:
        result = rpc(
            number,
            "tools/call",
            {"name": name, "arguments": arguments},
        )
        payload = json.loads(result["content"][0]["text"])
        if result.get("isError"):
            if allow_error:
                return {"error": payload}
            raise RuntimeError(payload)
        return payload

    def prepare_textedit(number: int) -> dict:
        created = tool(
            number,
            "task_create",
            {
                "idempotency_key": f"tm-s8-cua-target-{time.time_ns()}",
                "name": "Yonder CUA 目标准备",
                "description": "仅通过Yonder CUA启动TextEdit安全验证目标",
            },
        )["task"]
        launched = tool(
            number + 1,
            "computer_step",
            {
                "task_id": created["task_id"],
                "expected_sequence": created["sequence"],
                "step_id": "launch-textedit",
                "label": "由Yonder启动TextEdit验证窗口",
                "tool_name": "launch_app",
                "arguments": {"bundle_id": "com.apple.TextEdit"},
            },
        )
        completed = tool(
            number + 2,
            "task_complete",
            {
                "task_id": created["task_id"],
                "expected_sequence": launched["sequence"],
            },
        )["task"]
        return {
            "task_id": created["task_id"],
            "action_succeeded": launched["action_succeeded"],
            "final_status": completed["status"],
        }

    try:
        rpc(
            0,
            "initialize",
            {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {
                    "name": "codex-fast-slow-verification",
                    "version": "1",
                },
            },
        )
        process.stdin.write(
            '{"jsonrpc":"2.0","method":"notifications/initialized"}\n'
        )
        process.stdin.flush()
        setup = prepare_textedit(10)
        created = tool(
            20,
            "task_create",
            {
                "idempotency_key": f"tm-s8-fast-slow-{time.time_ns()}",
                "name": "快慢脑正式链路验证（纯Yonder CUA）",
                "description": "Codex经Gateway提交多候选计划片段，由Jev选择并由Yonder连续执行CUA",
            },
        )["task"]
        slots = [
            {
                "step_id": "enter-fixture-marker",
                "label": "在隔离窗口输入验证标记",
                "candidates": [
                    {
                        "candidate_id": "type-fixture-marker",
                        "tool_name": "type_text",
                        "arguments": {
                            "text": "YONDER_FAST_SLOW_SELECTED",
                            "delivery_mode": "background",
                        },
                        "action_kind": "draft-message",
                        "target_ref": "textedit-composer",
                        "preconditions": [
                            {"fact": "composer-ready", "expected": True}
                        ],
                        "expected_observe": [
                            {"fact": "composer-ready", "expected": True}
                        ],
                    },
                    {
                        "candidate_id": "type-unrelated-marker",
                        "tool_name": "type_text",
                        "arguments": {
                            "text": "UNRELATED_SAFE_MARKER",
                            "delivery_mode": "background",
                        },
                        "action_kind": "draft-message",
                        "target_ref": "unrelated-composer",
                        "preconditions": [
                            {"fact": "composer-ready", "expected": False}
                        ],
                        "expected_observe": [
                            {"fact": "composer-ready", "expected": False}
                        ],
                    },
                ],
            },
            {
                "step_id": "advance-fixture-focus",
                "label": "在同一窗口推进安全焦点",
                "candidates": [
                    {
                        "candidate_id": "press-tab",
                        "tool_name": "press_key",
                        "arguments": {
                            "key": "tab",
                            "delivery_mode": "background",
                        },
                        "action_kind": "resolve-conversation",
                        "target_ref": "textedit-window",
                        "preconditions": [
                            {"fact": "target-resolved", "expected": True}
                        ],
                        "expected_observe": [
                            {"fact": "target-resolved", "expected": True}
                        ],
                    }
                ],
            },
        ]
        submitted = tool(
            21,
            "task_plan_submit",
            {
                "task_id": created["task_id"],
                "expected_sequence": created["sequence"],
                "plan_id": "codex-jev-cua-plan",
                "plan_version": 1,
                "token_budget": 100,
                "slots": slots,
            },
        )
        print(
            json.dumps(
                {
                    "checkpoint": "plan-submitted",
                    "task_id": created["task_id"],
                    "sequence": submitted["sequence"],
                    "disposition": submitted["disposition"],
                },
                ensure_ascii=False,
            ),
            flush=True,
        )
        if args.checkpoint:
            input()
        started = time.monotonic()
        executed = tool(
            22,
            "task_plan_execute",
            {
                "task_id": created["task_id"],
                "expected_sequence": submitted["sequence"],
                "plan_id": "codex-jev-cua-plan",
                "plan_version": 1,
            },
        )
        elapsed_ms = round((time.monotonic() - started) * 1000)
        snapshot = tool(23, "task_get", {"task_id": created["task_id"]})["task"]
        events = tool(
            24,
            "task_events",
            {
                "task_id": created["task_id"],
                "after_sequence": "0",
                "limit": 100,
            },
        )["events"]
        completed = None
        if executed["disposition"] == "fragment-complete":
            completed = tool(
                25,
                "task_complete",
                {
                    "task_id": created["task_id"],
                    "expected_sequence": executed["sequence"],
                },
            )["task"]
        result = {
            "yonder_cua_setup": setup,
            "task_id": created["task_id"],
            "plan_submit": submitted,
            "plan_execute": executed,
            "execute_elapsed_ms": elapsed_ms,
            "status_before_complete": snapshot["status"],
            "events_before_complete": len(events),
            "observed_attempts": sum(
                event.get("attempt_result", {}).get("phase") == "observed"
                for event in events
            ),
            "final_status": completed["status"] if completed else snapshot["status"],
            "final_sequence": completed["sequence"]
            if completed
            else snapshot["sequence"],
        }
        if executed["disposition"] == "handback":
            last_sequence = None
            stable_reads = 0
            for offset in range(8):
                snapshot = tool(
                    30 + offset,
                    "task_get",
                    {"task_id": created["task_id"]},
                )["task"]
                if snapshot["sequence"] == last_sequence:
                    stable_reads += 1
                else:
                    stable_reads = 0
                    last_sequence = snapshot["sequence"]
                if stable_reads >= 1 and snapshot.get("next_intent"):
                    break
                time.sleep(0.2)
            result["task_after_handback"] = snapshot
            result["yonder_cua_replan_setup"] = prepare_textedit(40)
            replan_slots = [
                {
                    "step_id": "replan-type-marker",
                    "label": "交回后由Yonder输入验证标记",
                    "candidates": [slots[0]["candidates"][0]],
                },
                {
                    "step_id": "replan-advance-focus",
                    "label": "交回后由Yonder连续推进焦点",
                    "candidates": slots[1]["candidates"],
                },
            ]
            replan = tool(
                50,
                "task_plan_submit",
                {
                    "task_id": created["task_id"],
                    "expected_sequence": snapshot["sequence"],
                    "plan_id": "codex-replan-after-handback",
                    "plan_version": 1,
                    "token_budget": 100,
                    "slots": replan_slots,
                },
            )
            result["replan_submit"] = replan
            result["replan_execute"] = tool(
                51,
                "task_plan_execute",
                {
                    "task_id": created["task_id"],
                    "expected_sequence": replan["sequence"],
                    "plan_id": "codex-replan-after-handback",
                    "plan_version": 1,
                },
                allow_error=True,
            )
        print(json.dumps(result, ensure_ascii=False), flush=True)
        return 0 if completed and completed["status"] == "completed" else 1
    finally:
        if process.poll() is None:
            process.stdin.close()
            process.stdout.close()
            process.wait(timeout=5)


if __name__ == "__main__":
    raise SystemExit(main())
