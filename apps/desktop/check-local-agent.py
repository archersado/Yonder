#!/usr/bin/env python3
"""父进程测试 Agent 通过私有 stdio 调用正式 TaskHost/Gateway；不修改产品数据。"""
import json
import pathlib
import select
import sqlite3
import subprocess
import tempfile
import time
import sys


def check(binary):
    with tempfile.TemporaryDirectory(prefix="yonda-local-agent-") as directory:
        agent = subprocess.Popen([str(binary), directory], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        counter = 0

        def request(method, capability="task.read", **fields):
            nonlocal counter
            counter += 1
            request_id = f"agent-{counter}"
            payload = {"jsonrpc": "2.0", "id": request_id, "method": method, "params": {"agent_id": "local-test-agent", "capability": capability, "deadline": int(time.time() * 1000) + 10000, **fields}}
            agent.stdin.write(json.dumps(payload).encode() + b"\n")
            agent.stdin.flush()
            assert select.select([agent.stdout], [], [], 10)[0], "Gateway 响应超时"
            response = json.loads(agent.stdout.readline(65537))
            assert response["id"] == request_id
            return response

        try:
            assert request("task.create", "task.create", idempotency_key="one", description="测试任务一")["error"]["code"] == -32002
            hello = request("gateway.hello", protocol_version={"major": 1, "minor": 1})["result"]
            assert hello["protocol_version"] == {"major": 1, "minor": 1}
            assert any(c["name"] == "task.create" for c in hello["capabilities"])
            first = request("task.create", "task.create", idempotency_key="one", description="测试任务一")["result"]["task"]
            second = request("task.create", "task.create", idempotency_key="two", description="测试任务二")["result"]["task"]
            assert first["task_id"] != second["task_id"]
            assert first["status"] == second["status"] == "created"
            assert request("task.create", "task.create", idempotency_key="one", description="测试任务一")["result"]["task"] == first
            assert request("task.create", "task.create", idempotency_key="one", description="不同说明")["error"]["code"] == -32009
            tasks = request("task.list", limit=100)["result"]["tasks"]
            assert {t["task_id"] for t in tasks} == {first["task_id"], second["task_id"]}
            assert request("task.get", task_id=first["task_id"])["result"]["task"] == first
            assert len(request("task.events", task_id=first["task_id"], after_sequence="0", limit=100)["result"]["events"]) == 1
            agent.stdin.close()
            assert agent.wait(timeout=10) == 0
            with sqlite3.connect(pathlib.Path(directory) / "tasks.db") as db:
                assert db.execute("PRAGMA user_version").fetchone()[0] == 3
                for table in ["tasks", "events", "outbox", "task_creations"]:
                    assert db.execute(f"SELECT count(*) FROM {table}").fetchone()[0] == 2
            for frame in [b"{}", b"x" * 65537]:
                rejected = subprocess.run([str(binary), directory], input=frame, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=10)
                assert rejected.returncode != 0 and not rejected.stdout
            return {"bounded_frames_rejected": True, "local_stdio_agent": True, "actual_sqlite_tasks": 2, "idempotent_replay": True, "conflicting_key_rejected": True, "atomic_rows": True, "product_data_modified": False, "live_desktop_connection": False}
        finally:
            if agent.poll() is None:
                agent.terminate()
                agent.wait(timeout=10)
            agent.stdout.close()
            agent.stderr.close()
            if not agent.stdin.closed:
                agent.stdin.close()


if __name__ == "__main__":
    binary = pathlib.Path(__file__).resolve().parents[2] / "target/debug/examples/local-agent-gateway"
    result = check(binary)
    if len(sys.argv) > 1:
        output = pathlib.Path(sys.argv[1])
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(result, ensure_ascii=False))
