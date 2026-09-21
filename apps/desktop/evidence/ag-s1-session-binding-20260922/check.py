#!/usr/bin/env python3
"""AG-S1本地UDS首帧身份绑定与CLI显式身份验证。"""

import json
import os
import pathlib
import stat
import subprocess
import tempfile
import time
import socket

ROOT = pathlib.Path(__file__).resolve().parents[4]
DESKTOP = ROOT / "target/debug/yonder-desktop"
CLI = ROOT / "target/debug/yonder"
PROTOCOL = {"major": 1, "minor": 18}


def request(name, request_id, params):
    return {"jsonrpc": "2.0", "id": request_id, "method": name, "params": params}


def base_params(agent_id, capability):
    return {
        "agent_id": agent_id,
        "capability": capability,
        "deadline": int(time.time() * 1000) + 60_000,
    }


def read_frame(stream):
    line = stream.readline()
    assert line.endswith(b"\n"), line
    return json.loads(line)


def write_json(stream, value):
    stream.write(json.dumps(value, ensure_ascii=False).encode() + b"\n")
    stream.flush()


def hello(stream, request_id, agent_id):
    params = base_params(agent_id, "task.read")
    params["protocol_version"] = PROTOCOL
    write_json(stream, request("gateway.hello", request_id, params))
    response = read_frame(stream)
    assert response["id"] == request_id and "result" in response, response
    return response["result"]


def call(stream, name, request_id, params, expect_error=False):
    write_json(stream, request(name, request_id, params))
    response = read_frame(stream)
    assert response["id"] == request_id, response
    if expect_error:
        assert "error" in response, response
        return response["error"]
    assert "result" in response, response
    return response["result"]


def create(stream, request_id, agent_id, key):
    params = base_params(agent_id, "task.create")
    params.update({"idempotency_key": key, "description": "AG-S1会话绑定验证", "name": "会话绑定"})
    return call(stream, "task.create", request_id, params)["task"]


def get(stream, request_id, agent_id, task_id, expect_error=False):
    params = base_params(agent_id, "task.read")
    params["task_id"] = task_id
    return call(stream, "task.get", request_id, params, expect_error=expect_error)


def cli_call(process, request_id, method, params):
    process.stdin.write(json.dumps({"jsonrpc": "2.0", "id": request_id, "method": method, "params": params}) + "\n")
    process.stdin.flush()
    response = json.loads(process.stdout.readline())
    assert response["id"] == request_id and "result" in response, response
    return response["result"]


def run():
    home = pathlib.Path(tempfile.mkdtemp(prefix="/tmp/yonder-ag-s1-verify-"))
    data_dir = home / "Library/Application Support/com.yonder.desktop"
    socket_path = data_dir / "agent.sock"
    with open(home / "desktop.stderr", "wb") as stderr:
        desktop = subprocess.Popen(
            [str(DESKTOP), "--local-agent-stdio"], cwd=ROOT, env={**os.environ, "HOME": str(home)}, stderr=stderr
        )
        try:
            for _ in range(100):
                if socket_path.exists():
                    break
                time.sleep(0.1)
            assert socket_path.exists(), "UDS未启动"
            assert stat.S_IMODE(data_dir.stat().st_mode) == 0o700
            assert stat.S_IMODE(socket_path.stat().st_mode) == 0o600

            first = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            first.settimeout(5)
            first.connect(str(socket_path))
            with first.makefile("rwb", buffering=0) as stream:
                hello(stream, "hello-a", "agent-a")
                created_a = create(stream, "create-a", "agent-a", f"ag-s1-a-{time.time_ns()}")
                assert created_a["owner_agent_id"] == "agent-a"
                assert get(stream, "get-a", "agent-a", created_a["task_id"])["task"] == created_a
                error = get(stream, "get-cross", "agent-b", created_a["task_id"], expect_error=True)
                assert error["code"] == -32003, error
                second_hello = call(stream, "gateway.hello", "hello-b", {
                    **base_params("agent-b", "task.read"), "protocol_version": PROTOCOL
                }, expect_error=True)
                assert second_hello["code"] == -32003, second_hello
            first.close()

            second = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            second.settimeout(5)
            second.connect(str(socket_path))
            with second.makefile("rwb", buffering=0) as stream:
                hello(stream, "hello-b", "agent-b")
                created_b = create(stream, "create-b", "agent-b", f"ag-s1-b-{time.time_ns()}")
                assert created_b["owner_agent_id"] == "agent-b"
                cross_error = get(stream, "get-cross", "agent-a", created_b["task_id"], expect_error=True)
                assert cross_error["code"] == -32003, cross_error
            second.close()

            missing = subprocess.run([str(CLI), "mcp"], cwd=ROOT, input="", text=True, capture_output=True)
            assert missing.returncode != 0 and "YONDER_AGENT_ID" in missing.stderr, (missing.returncode, missing.stderr)
            invalid = subprocess.run(
                [str(CLI), "mcp"], cwd=ROOT, input="", text=True, capture_output=True,
                env={**os.environ, "HOME": str(home), "YONDER_AGENT_ID": "bad id"}
            )
            assert invalid.returncode != 0 and "YONDER_AGENT_ID无效" in invalid.stderr, (invalid.returncode, invalid.stderr)
            with subprocess.Popen(
                [str(CLI), "mcp"], cwd=ROOT, text=True, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                env={**os.environ, "HOME": str(home), "YONDER_AGENT_ID": "agent-b"}
            ) as mcp:
                initialized = cli_call(mcp, 1, "initialize", {
                    "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "verification", "version": "1"}
                })
                assert initialized["protocolVersion"] == "2025-06-18"

            result = {
                "platform": "macOS",
                "socket_path": "HOME/Library/Application Support/com.yonder.desktop/agent.sock",
                "directory_mode": "0700",
                "socket_mode": "0600",
                "first_hello_binding": True,
                "same_connection_identity_switch_rejected": True,
                "cross_connection_isolation": True,
                "cli_missing_agent_id_rejected": True,
                "cli_invalid_agent_id_rejected": True,
                "cli_valid_agent_id_initialize": True,
                "agent_a_task_id": created_a["task_id"],
                "agent_b_task_id": created_b["task_id"],
                "passed": True,
            }
            (pathlib.Path(__file__).parent / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
            print(json.dumps(result, ensure_ascii=False))
        finally:
            desktop.terminate()
            desktop.wait(timeout=10)


if __name__ == "__main__":
    run()
