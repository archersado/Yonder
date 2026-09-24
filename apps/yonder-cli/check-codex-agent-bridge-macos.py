#!/usr/bin/env python3
"""隔离验证Codex薄桥接的steer/start路由，不访问真实会话或模型。"""

import json
import os
import pathlib
import socket
import subprocess
import tempfile
import threading
import time


ROOT = pathlib.Path(__file__).resolve().parents[2]
BINARY = ROOT / "target/debug/yonder"
THREAD_ID = "thread-contract-1"


FAKE_CODEX = r'''#!/usr/bin/env python3
import json, os, sys
reads = 0
with open(os.environ["FAKE_CODEX_METHODS"], "w", encoding="utf-8") as log:
    for line in sys.stdin:
        message = json.loads(line)
        method = message.get("method")
        if method == "initialized":
            continue
        log.write(method + "\n")
        log.flush()
        request_id = message["id"]
        if method == "initialize":
            result = {"userAgent": "fake"}
        elif method == "thread/loaded/list":
            result = {"data": ["thread-contract-1"]}
        elif method == "thread/read":
            reads += 1
            if reads == 1:
                result = {"thread": {"status": {"type": "active"}, "turns": [{"id": "turn-active", "status": "inProgress"}]}}
            else:
                result = {"thread": {"status": {"type": "idle"}, "turns": [{"id": "turn-interrupted", "status": "interrupted"}]}}
        elif method == "turn/steer":
            assert message["params"]["expectedTurnId"] == "turn-active"
            result = {"turnId": "turn-active"}
        elif method == "turn/start":
            result = {"turn": {"id": "turn-new", "status": "inProgress", "items": [], "error": None}}
        else:
            print(json.dumps({"id": request_id, "error": {"code": -32601, "message": "unsupported"}}), flush=True)
            continue
        print(json.dumps({"method": "thread/status/changed", "params": {"threadId": "thread-contract-1"}}), flush=True)
        print(json.dumps({"id": request_id, "result": result}), flush=True)
'''


def frame(connection):
    data = b""
    while not data.endswith(b"\n"):
        chunk = connection.recv(65536)
        if not chunk:
            raise EOFError("connection closed")
        data += chunk
    return json.loads(data[:-1])


def send(connection, value):
    connection.sendall(json.dumps(value, separators=(",", ":")).encode() + b"\n")


def gateway(socket_path, results):
    server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    server.bind(socket_path)
    server.listen(1)
    connection, _ = server.accept()
    hello = frame(connection)
    assert hello["method"] == "gateway.hello"
    assert hello["params"]["session_id"] == THREAD_ID
    assert hello["params"]["offered_capabilities"] == ["user_input"]
    send(connection, {"jsonrpc": "2.0", "id": "hello", "result": {"kind": "hello", "protocol_version": {"major": 1, "minor": 19}, "platform": "macos", "capabilities": []}})
    for index in (1, 2):
        now = int(time.time() * 1000)
        request_id = f"input-{index}"
        send(connection, {"jsonrpc": "2.0", "id": request_id, "method": "agent.input", "params": {"input_id": request_id, "session_id": THREAD_ID, "source": "voice", "content": f"contract-{index}", "created_at": now, "deadline": now + 10000}})
        response = frame(connection)
        results.append(response["id"] == request_id and response["result"]["accepted"] is True)
    connection.close()
    server.close()


def main():
    if not BINARY.exists():
        raise SystemExit("请先构建yonder-cli")
    with tempfile.TemporaryDirectory(prefix="ya5-", dir="/tmp") as temporary:
        root = pathlib.Path(temporary)
        fake_bin = root / "bin"
        fake_bin.mkdir()
        codex = fake_bin / "codex"
        codex.write_text(FAKE_CODEX, encoding="utf-8")
        codex.chmod(0o700)
        socket_directory = root / "Library/Application Support/com.yonder.desktop"
        socket_directory.mkdir(parents=True)
        yonder_socket = socket_directory / "agent.sock"
        methods_path = root / "methods.txt"
        app_server_socket = root / "codex.sock"
        app_server_socket.touch()
        results = []
        gateway_thread = threading.Thread(target=gateway, args=(str(yonder_socket), results), daemon=True)
        gateway_thread.start()
        environment = os.environ.copy()
        environment.update({
            "HOME": str(root),
            "PATH": str(fake_bin) + os.pathsep + environment["PATH"],
            "FAKE_CODEX_METHODS": str(methods_path),
            "YONDER_AGENT_ID": "codex-contract",
            "YONDER_CODEX_THREAD_ID": THREAD_ID,
            "YONDER_CODEX_APP_SERVER_SOCKET": str(app_server_socket),
        })
        process = subprocess.run([str(BINARY), "agent-bridge"], env=environment, capture_output=True, text=True, timeout=15)
        gateway_thread.join(timeout=5)
        methods = methods_path.read_text(encoding="utf-8").splitlines()
        expected = ["initialize", "thread/loaded/list", "thread/read", "turn/steer", "thread/read", "turn/start"]
        passed = results == [True, True] and methods == expected
        report = {"passed": passed, "accepted": results, "methods": methods, "bridge_closed_after_gateway_eof": process.returncode != 0}
        if not passed:
            report["bridge_error"] = process.stderr.strip()
        print(json.dumps(report, ensure_ascii=False))
        if not passed:
            raise SystemExit(1)


if __name__ == "__main__":
    main()
