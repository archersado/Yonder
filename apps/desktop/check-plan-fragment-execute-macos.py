#!/usr/bin/env python3
"""EX-S2 的受控 macOS 原生单槽位验证；输出不含模型、窗口或输入正文。"""
import json
import os
import pathlib
import queue
import signal
import sqlite3
import subprocess
import sys
import tempfile
import threading
import time

root = pathlib.Path(__file__).resolve().parents[2]
binary = root / "target/debug/Yonda.app/Contents/MacOS/yonder-desktop"
fixture_bin = pathlib.Path("/private/tmp/yonder-ex2-plan-fixture")
fixture_src = root / "apps/desktop/tests/plan-fragment-fixture-macos.swift"
result = {"platform": "macos", "recording_started": False}
fixture = None
agent = None
fixture_state = {}
test_home = None

def verification_deadline(_signal, _frame):
    raise TimeoutError("verification-deadline-exceeded")

signal.signal(signal.SIGALRM, verification_deadline)
signal.alarm(25)

def read_line_with_timeout(stream, timeout=2):
    """子进程意外保留 stdout 时也必须让验证器进入 finally 清理。"""
    lines = queue.Queue(maxsize=1)
    threading.Thread(target=lambda: lines.put(stream.readline()), daemon=True).start()
    try:
        return lines.get(timeout=timeout)
    except queue.Empty:
        return ""

def isolated_agent_environment():
    """创建全新任务库，只导入 Jev 非秘密配置；凭据始终由 Keychain 提供。"""
    global test_home
    test_home = tempfile.TemporaryDirectory(prefix="yonder-ex2-home-")
    home = pathlib.Path(test_home.name)
    environment = os.environ.copy()
    environment["HOME"] = str(home)
    database = home / "Library/Application Support/com.yonder.desktop/tasks.db"
    # 首次运行由正式组合根创建 schema，之后再关闭；不能手写任务库结构。
    initializer = subprocess.Popen(
        [str(binary), "--local-agent-stdio"],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
        env=environment,
    )
    until = time.monotonic() + 10
    while time.monotonic() < until and not database.exists():
        time.sleep(0.05)
    initializer.terminate()
    try:
        initializer.wait(timeout=5)
    except subprocess.TimeoutExpired:
        initializer.kill()
        initializer.wait()
    if not database.exists():
        raise RuntimeError("isolated-store-not-created")
    source = pathlib.Path.home() / "Library/Application Support/com.yonder.desktop/tasks.db"
    try:
        with sqlite3.connect(f"file:{source}?mode=ro", uri=True) as source_db:
            row = source_db.execute("SELECT config_json FROM jev_config WHERE id=1").fetchone()
        if row is None:
            raise RuntimeError("jev-config-unavailable")
        with sqlite3.connect(database) as isolated_db:
            isolated_db.execute(
                "INSERT INTO jev_config(id, config_json) VALUES (1, ?1) "
                "ON CONFLICT(id) DO UPDATE SET config_json=excluded.config_json",
                row,
            )
    except sqlite3.Error as error:
        raise RuntimeError("isolated-jev-config-unavailable") from error
    return environment

def refocus_fixture():
    """宿主启动可能成为前台窗口；在派发前恢复唯一 fixture 为受控目标。"""
    focus_adapter = root / "target/debug/examples/work_focus_check"
    last_failure = "capture-failed"
    for _ in range(3):
        focus_process = subprocess.Popen(
            [str(focus_adapter), str(fixture_state["pid"]), str(fixture_state["window_id"])],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1,
        )
        ready_line = read_line_with_timeout(focus_process.stdout)
        ready = json.loads(ready_line) if ready_line else {"ready": False}
        outcome = {}
        if ready.get("ready"):
            focus_process.stdin.write("focus\n")
            focus_process.stdin.flush()
            outcome_line = read_line_with_timeout(focus_process.stdout)
            outcome = json.loads(outcome_line) if outcome_line else {}
        focus_process.stdin.write("release\nquit\n")
        focus_process.stdin.flush()
        focus_process.wait(timeout=5)
        if outcome.get("outcome") == "focused":
            return
        last_failure = outcome.get("outcome", "capture-failed")
        time.sleep(0.2)
    raise RuntimeError(f"fixture-refocus-{last_failure}")

try:
    subprocess.run(["swiftc", str(fixture_src), "-o", str(fixture_bin)], check=True)
    fixture = subprocess.Popen([str(fixture_bin)], stdout=subprocess.PIPE, text=True)
    def read_fixture():
        for line in fixture.stdout:
            try:
                fixture_state.update(json.loads(line))
            except json.JSONDecodeError:
                pass
    threading.Thread(target=read_fixture, daemon=True).start()
    until = time.monotonic() + 15
    while time.monotonic() < until:
        if (fixture_state.get("ready") and fixture_state.get("launched")
                and fixture_state.get("pid") and fixture_state.get("window_id")):
            break
        time.sleep(0.05)
    if not (fixture_state.get("ready") and fixture_state.get("launched")
            and fixture_state.get("pid") and fixture_state.get("window_id")):
        raise RuntimeError("fixture-not-ready")
    # `windowNumber` 已分配并不表示 WindowServer/AX 树已经同时可查询；等待
    # 一个短暂稳定窗口只影响无副作用的夹具预检。
    time.sleep(0.3)
    # AppKit 在首次激活期间可能重排标题栏。该循环只重新捕获 fixture 的
    # 焦点引用，尚未创建任务或派发 CUA，不能掩盖实际执行时的几何变化拒绝。
    focus_adapter = root / "target/debug/examples/work_focus_check"
    focus_failure = "unknown"
    for _ in range(3):
        focus_process = subprocess.Popen(
            [str(focus_adapter), str(fixture_state["pid"]), str(fixture_state["window_id"])],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1,
        )
        focus_ready_line = read_line_with_timeout(focus_process.stdout)
        focus_ready = json.loads(focus_ready_line) if focus_ready_line else {"ready": False}
        focus_result = {}
        if focus_ready.get("ready"):
            focus_process.stdin.write("focus\n")
            focus_process.stdin.flush()
            focus_result_line = read_line_with_timeout(focus_process.stdout)
            focus_result = json.loads(focus_result_line) if focus_result_line else {}
        focus_process.stdin.write("release\nquit\n")
        focus_process.stdin.flush()
        focus_process.wait(timeout=5)
        if focus_result.get("outcome") == "focused":
            break
        focus_failure = focus_result.get("outcome", "capture-failed")
        time.sleep(0.2)
    else:
        raise RuntimeError(f"fixture-focus-{focus_failure}")

    agent = subprocess.Popen(
        [str(binary), "--local-agent-stdio"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        env=isolated_agent_environment(),
    )
    # 先启动正式宿主并完成其数据目录初始化，再恢复 fixture；保证 CUA 捕获的
    # frontmost target 就是这一个隔离窗口，而非 Yonder 自身的辅助窗口。
    time.sleep(0.3)
    refocus_fixture()
    agent_responses = queue.Queue()
    def read_agent():
        for line in agent.stdout:
            try:
                agent_responses.put(json.loads(line))
            except json.JSONDecodeError:
                agent_responses.put(None)
    threading.Thread(target=read_agent, daemon=True).start()
    counter = [0]
    def call(method, capability, **params):
        counter[0] += 1
        request = {
            "jsonrpc": "2.0", "id": f"ex2-{counter[0]}", "method": method,
            "params": {"agent_id": "local-test-agent", "capability": capability,
                       "deadline": int(time.time() * 1000) + 45_000, **params},
        }
        agent.stdin.write(json.dumps(request).encode() + b"\n")
        agent.stdin.flush()
        try:
            response = agent_responses.get(timeout=45)
        except queue.Empty:
            raise RuntimeError("gateway-timeout")
        if response is None or response.get("id") != request["id"]:
            raise RuntimeError(f"gateway-{method}")
        if "error" in response:
            raise RuntimeError(f"gateway-{method}-{response['error'].get('code', 'unknown')}")
        return response["result"]

    hello = call("gateway.hello", "task.read", protocol_version={"major": 1, "minor": 31})
    capabilities = {entry.get("name"): entry.get("availability") for entry in hello.get("capabilities", [])}
    result["plan_execute_capability"] = capabilities.get("task.plan.execute") == "available"
    if not result["plan_execute_capability"]:
        raise RuntimeError("plan-execute-unavailable")
    task = call(
        "task.create", "task.create", idempotency_key=f"ex2-native-{time.time_ns()}",
        description="EX-S2隔离原生计划片段验证", name="EX-S2 原生验证",
    )["task"]
    submitted = call(
        "task.plan.submit", "task.plan.submit", task_id=task["task_id"],
        expected_sequence=task["sequence"], plan_id="native-plan", plan_version=1,
        token_budget=100, slots=[{"step_id": "native-input", "label": "隔离窗口输入验证",
        "candidates": [{"candidate_id": "type_text", "tool_name": "type_text",
                        "arguments": {"text": "YONDER_SDK_INPUT_A", "delivery_mode": "background"}}]}],
    )
    executed = call(
        "task.plan.execute", "task.plan.execute", task_id=task["task_id"],
        expected_sequence=submitted["sequence"], plan_id="native-plan", plan_version=1,
    )
    result["jev_selected_submitted_candidate"] = executed.get("disposition") == "advanced"
    if not result["jev_selected_submitted_candidate"]:
        raise RuntimeError("plan-handed-back")
    until = time.monotonic() + 8
    native_match = False
    while time.monotonic() < until:
        if fixture_state.get("input_matches"):
            native_match = True
            break
        time.sleep(0.05)
    result["native_target_matches"] = native_match
    if not native_match:
        raise RuntimeError("native-target-mismatch")
    completed = call("task.complete", "task.complete", task_id=task["task_id"], expected_sequence=executed["sequence"])["task"]
    result["task_completed"] = completed.get("status") == "completed"
    # recording_started 必须为 false，不能把这项安全断言混入正向通过条件。
    result["passed"] = all(result.get(key) is True for key in (
        "plan_execute_capability",
        "jev_selected_submitted_candidate",
        "native_target_matches",
        "task_completed",
    ))
except Exception as error:
    result["failure_class"] = str(error)
    result["passed"] = False
finally:
    if agent is not None:
        agent.terminate()
        try:
            agent.wait(timeout=5)
        except subprocess.TimeoutExpired:
            agent.kill()
            agent.wait()
    if fixture is not None:
        fixture.terminate()
        try:
            fixture.wait(timeout=5)
        except subprocess.TimeoutExpired:
            fixture.kill()
            fixture.wait()
    if test_home is not None:
        test_home.cleanup()
    signal.alarm(0)

print(json.dumps(result, ensure_ascii=False))
sys.exit(0 if result.get("passed") else 1)
