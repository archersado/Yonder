#!/usr/bin/env python3
"""TM-S7macOS真实Browser Gateway共享启动验证。"""

import json
import os
import pathlib
import shutil
import socket
import subprocess
import tempfile
import time

ROOT = pathlib.Path(__file__).resolve().parents[4]
PROTOCOL = {"major": 1, "minor": 17}
AGENT_ID = "codex-cli"


def base_params(capability):
    return {
        "agent_id": AGENT_ID,
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


def hello(stream):
    params = base_params("task.read")
    params["protocol_version"] = PROTOCOL
    params["agent_id"] = AGENT_ID
    write_json(stream, {"jsonrpc": "2.0", "id": "hello", "method": "gateway.hello", "params": params})
    response = read_frame(stream)
    assert response["id"] == "hello" and "result" in response, response
    return response["result"]


def call(stream, request_id, method, capability, **params):
    request = {"jsonrpc": "2.0", "id": request_id, "method": method, "params": {**base_params(capability), **params}}
    write_json(stream, request)
    response = read_frame(stream)
    assert response["id"] == request_id and "result" in response, response
    return response["result"]


def run():
    output_dir = pathlib.Path(__file__).parent
    real_home = pathlib.Path(os.environ["HOME"])
    ego_binary = real_home / ".local/bin/ego-browser"
    assert ego_binary.exists(), "真实ego-browser不存在"

    home = pathlib.Path(tempfile.mkdtemp(prefix="/tmp/yonder-tm-s7-native-"))
    data_dir = home / "Library/Application Support/com.yonder.desktop"
    socket_path = data_dir / "agent.sock"
    ego_link_dir = home / ".local/bin"
    ego_link_dir.mkdir(parents=True)
    (ego_link_dir / "ego-browser").symlink_to(ego_binary.resolve())

    with open(home / "desktop.stderr", "wb") as stderr:
        desktop = subprocess.Popen(
            [str(ROOT / "target/debug/yonder-desktop")], cwd=ROOT,
            env={**os.environ, "HOME": str(home)}, stderr=stderr
        )
        stream = None
        try:
            for _ in range(100):
                if socket_path.exists():
                    break
                time.sleep(0.1)
            assert socket_path.exists(), "生产UDS未启动"
            stream = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            stream.settimeout(45)
            stream.connect(str(socket_path))
            with stream.makefile("rwb", buffering=0) as gateway:
                handshake = hello(gateway)
                capabilities = {item["name"]: item for item in handshake["capabilities"]}
                assert handshake["protocol_version"] == PROTOCOL, handshake
                assert capabilities["browser.execute"]["availability"] == "available", capabilities

                created = call(gateway, "create", "task.create", "task.create",
                               idempotency_key=f"tm-s7-native-{time.time_ns()}",
                               description="验证TM-S7统一启动事实", name="统一启动原生验证")["task"]
                assert created["status"] == "created", created

                declared = call(gateway, "declare", "task.step.declare", "task.step.declare",
                                task_id=created["task_id"], expected_sequence=created["sequence"],
                                step_id="create-space", label="创建Browser Task Space")["task"]
                opened = call(gateway, "open", "browser.execute", "browser.execute",
                              task_id=created["task_id"], expected_sequence=declared["sequence"], operation="create")
                assert opened["task"]["status"] == "running", opened
                external_ref = opened["reference"]["external_task_ref"]
                assert external_ref.startswith("ego:") and not opened["reference"]["finished"], opened

                advanced = call(gateway, "advance", "task.step.advance", "task.step.advance",
                                task_id=created["task_id"], expected_sequence=opened["task"]["sequence"])["task"]
                closing = call(gateway, "declare-finish", "task.step.declare", "task.step.declare",
                               task_id=created["task_id"], expected_sequence=advanced["sequence"],
                               step_id="finish-space", label="完成Browser Task Space")["task"]
                finished = call(gateway, "finish", "browser.execute", "browser.execute",
                                task_id=created["task_id"], expected_sequence=closing["sequence"], operation="finish")
                assert finished["task"]["status"] == "completed", finished
                assert finished["reference"]["external_task_ref"] == external_ref, finished
                assert finished["reference"]["finished"], finished

                events = call(gateway, "events", "task.events", "task.read",
                              task_id=created["task_id"], after_sequence="0", limit=100)["events"]
                running_events = [event for event in events if event.get("status") == "running"]
                assert running_events, events
                result = {
                    "platform": "macOS",
                    "runtime": "ego-lite",
                    "protocol": handshake["protocol_version"],
                    "task_id": created["task_id"],
                    "created_status": created["status"],
                    "after_first_dispatch_status": opened["task"]["status"],
                    "external_task_ref": external_ref,
                    "final_status": finished["task"]["status"],
                    "running_events": len(running_events),
                    "total_events": len(events),
                    "passed": True,
                }
                (output_dir / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
                print(json.dumps(result, ensure_ascii=False))
        finally:
            if stream is not None:
                stream.close()
            desktop.terminate()
            desktop.wait(timeout=10)


if __name__ == "__main__":
    run()
