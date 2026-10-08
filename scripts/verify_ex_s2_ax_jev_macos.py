#!/usr/bin/env python3
"""经正式 Yonder MCP/Gateway 验证新鲜 AX 候选的自然 Jev 选择。"""

import json
import os
import pathlib
import plistlib
import queue
import signal
import subprocess
import sys
import threading
import time


ROOT = pathlib.Path(__file__).resolve().parents[1]
YONDER = ROOT / "target/debug/Yonda.app/Contents/MacOS/yonder"
FIXTURE_SOURCE = ROOT / "apps/desktop/tests/plan-fragment-fixture-macos.swift"
FIXTURE_APP = pathlib.Path("/private/tmp/YonderEX2AXFixture.app")
FIXTURE_BINARY = FIXTURE_APP / "Contents/MacOS/YonderEX2AXFixture"
FOCUS = ROOT / "target/debug/examples/work_focus_check"
result = {"platform": "macos", "entry": "bundled-yonder-mcp", "passed": False}
fixture_state: dict = {}
fixture = None
mcp = None
task = None
tool_fn = None


def deadline(_signal, _frame):
    raise TimeoutError("verification-deadline-exceeded")


def read_fixture(stream):
    for line in stream:
        try:
            fixture_state.update(json.loads(line))
        except json.JSONDecodeError:
            pass


def focus_fixture():
    failure = "not-ready"
    for _ in range(3):
        process = subprocess.Popen(
            [str(FOCUS), str(fixture_state["pid"]), str(fixture_state["window_id"])],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True,
        )
        ready = json.loads(process.stdout.readline())
        focused = {}
        if ready.get("ready"):
            process.stdin.write("focus\n")
            process.stdin.flush()
            focused = json.loads(process.stdout.readline())
        process.stdin.write("release\nquit\n")
        process.stdin.flush()
        process.wait(timeout=5)
        if focused.get("outcome") == "focused":
            return
        failure = focused.get("outcome", "not-ready")
        time.sleep(0.2)
    if fixture_state.get("target_key") is True:
        return
    raise RuntimeError(f"fixture-focus-{failure}")


try:
    signal.signal(signal.SIGALRM, deadline)
    signal.alarm(90)
    for path in (YONDER, FOCUS):
        if not path.is_file():
            raise RuntimeError(f"missing-artifact-{path.name}")
    FIXTURE_BINARY.parent.mkdir(parents=True, exist_ok=True)
    with (FIXTURE_APP / "Contents/Info.plist").open("wb") as stream:
        plistlib.dump({
            "CFBundleExecutable": FIXTURE_BINARY.name,
            "CFBundleIdentifier": "com.yonder.fixture.exs2",
            "CFBundleName": "Yonder EX-S2 AX Fixture",
            "CFBundlePackageType": "APPL",
            "CFBundleVersion": "1",
            "LSMinimumSystemVersion": "13.0",
        }, stream)
    subprocess.run(["swiftc", str(FIXTURE_SOURCE), "-o", str(FIXTURE_BINARY)], check=True)
    subprocess.run(["codesign", "--force", "--sign", "-", str(FIXTURE_APP)], check=True)
    fixture = subprocess.Popen([str(FIXTURE_BINARY)], stdout=subprocess.PIPE, text=True)
    threading.Thread(target=read_fixture, args=(fixture.stdout,), daemon=True).start()
    until = time.monotonic() + 10
    while time.monotonic() < until and not all(
        fixture_state.get(key) for key in ("ready", "launched", "pid", "window_id")
    ):
        time.sleep(0.05)
    if not fixture_state.get("window_id"):
        raise RuntimeError("fixture-not-ready")
    time.sleep(0.3)
    focus_fixture()

    mcp = subprocess.Popen(
        [str(YONDER), "mcp"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True,
        env={**os.environ, "YONDER_AGENT_ID": "codex-cli"},
    )
    responses: queue.Queue = queue.Queue()

    def read_mcp():
        for line in mcp.stdout:
            try:
                responses.put(json.loads(line))
            except json.JSONDecodeError:
                pass

    threading.Thread(target=read_mcp, daemon=True).start()
    request_id = 0

    def rpc(method, params):
        rpc.counter += 1
        request = {"jsonrpc": "2.0", "id": rpc.counter, "method": method, "params": params}
        mcp.stdin.write(json.dumps(request, ensure_ascii=False) + "\n")
        mcp.stdin.flush()
        response = responses.get(timeout=45)
        if response.get("id") != rpc.counter or "error" in response:
            raise RuntimeError(f"mcp-{method}-failed")
        return response["result"]
    rpc.counter = request_id

    def tool(name, arguments):
        response = rpc("tools/call", {"name": name, "arguments": arguments})
        payload = json.loads(response["content"][0]["text"])
        if response.get("isError"):
            raise RuntimeError(f"tool-{name}-{payload.get('code', 'failed')}")
        return payload
    tool_fn = tool

    rpc("initialize", {
        "protocolVersion": "2025-06-18",
        "capabilities": {},
        "clientInfo": {"name": "ex-s2-ax-jev-verification", "version": "1"},
    })
    mcp.stdin.write('{"jsonrpc":"2.0","method":"notifications/initialized"}\n')
    mcp.stdin.flush()
    task = tool("task_create", {
        "idempotency_key": f"ex-s2-ax-jev-{time.time_ns()}",
        "name": "AX 控件快脑自然选择验证",
        "description": "Codex慢脑提交语义步骤，Jev从Yonder新鲜Observe的控件候选中选择",
    })["task"]
    submitted = tool("task_plan_submit", {
        "task_id": task["task_id"],
        "expected_sequence": task["sequence"],
        "plan_id": "ax-natural-choice-plan",
        "plan_version": 1,
        "token_budget": 100,
        "slots": [
            {
                "step_id": "observe-fixture-controls",
                "label": "打开隔离验证应用并观察控件",
                "candidates": [{
                    "candidate_id": "launch-fixture-app",
                    "tool_name": "launch_app",
                    "arguments": {"bundle_id": "com.yonder.fixture.exs2"},
                    "action_kind": "launch-application",
                    "target_ref": "isolated-fixture-window",
                    "preconditions": [{"fact": "target-resolved", "expected": True}],
                    "expected_observe": [{"fact": "target-resolved", "expected": True}],
                }],
            },
            {
                "step_id": "choose-primary-control",
                "label": "点击执行验证主动作按钮",
                "candidates": [{
                    "candidate_id": "activate-observed-control",
                    "tool_name": "click",
                    "arguments": {},
                    "action_kind": "activate-control",
                    "target_ref": "isolated-fixture-window",
                    "preconditions": [{"fact": "target-resolved", "expected": True}],
                    "expected_observe": [{"fact": "target-resolved", "expected": True}],
                }],
            },
        ],
    })
    started = time.monotonic()
    executed = tool("task_plan_execute", {
        "task_id": task["task_id"],
        "expected_sequence": submitted["sequence"],
        "plan_id": "ax-natural-choice-plan",
        "plan_version": 1,
    })
    elapsed_ms = round((time.monotonic() - started) * 1000)
    time.sleep(0.2)
    events = tool("task_events", {
        "task_id": task["task_id"], "after_sequence": "0", "limit": 100,
    })["events"]
    primary_selected = fixture_state.get("primary_selected") is True
    fragment_finished = executed.get("disposition") in (
        "fragment-complete", "awaiting-goal-verification"
    )
    if fragment_finished and primary_selected:
        completed = tool("task_complete", {
            "task_id": task["task_id"], "expected_sequence": executed["sequence"],
        })["task"]
        result["passed"] = completed.get("status") == "completed"
    result.update({
        "task_id": task["task_id"],
        "disposition": executed.get("disposition"),
        "elapsed_ms": elapsed_ms,
        "primary_selected": primary_selected,
        "event_count": len(events),
        "observed_attempts": sum(
            event.get("attempt_result", {}).get("phase") == "observed" for event in events
        ),
    })
except Exception as error:
    result["failure_class"] = str(error)
finally:
    signal.alarm(0)
    if task is not None and not result["passed"] and tool_fn is not None:
        try:
            snapshot = tool_fn("task_get", {"task_id": task["task_id"]})["task"]
            if snapshot.get("status") in ("created", "running", "interrupted"):
                tool_fn("task_cancel", {
                    "task_id": task["task_id"],
                    "expected_sequence": snapshot["sequence"],
                })
        except Exception:
            pass
    for process in (mcp, fixture):
        if process is not None and process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()

print(json.dumps(result, ensure_ascii=False))
sys.exit(0 if result["passed"] else 1)
