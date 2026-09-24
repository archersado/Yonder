#!/usr/bin/env python3
"""隔离验证Codex薄桥接的WebSocket-over-UDS、steer/start路由，不访问真实会话或模型。"""

import base64
import hashlib
import json
import os
import pathlib
import socket
import struct
import subprocess
import tempfile
import threading
import time


ROOT = pathlib.Path(__file__).resolve().parents[2]
BINARY = ROOT / "target/debug/yonder"
THREAD_ID = "thread-contract-1"
WEBSOCKET_GUID = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"


def receive_exact(connection, length):
    data = b""
    while len(data) < length:
        chunk = connection.recv(length - len(data))
        if not chunk:
            raise EOFError("connection closed")
        data += chunk
    return data


def receive_websocket(connection):
    first, second = receive_exact(connection, 2)
    assert first & 0x0F == 1 and second & 0x80
    length = second & 0x7F
    if length == 126:
        length = struct.unpack("!H", receive_exact(connection, 2))[0]
    elif length == 127:
        length = struct.unpack("!Q", receive_exact(connection, 8))[0]
    mask = receive_exact(connection, 4)
    payload = receive_exact(connection, length)
    return json.loads(bytes(value ^ mask[index % 4] for index, value in enumerate(payload)))


def send_websocket(connection, value):
    payload = json.dumps(value, separators=(",", ":")).encode()
    if len(payload) < 126:
        header = bytes((0x81, len(payload)))
    elif len(payload) <= 0xFFFF:
        header = bytes((0x81, 126)) + struct.pack("!H", len(payload))
    else:
        header = bytes((0x81, 127)) + struct.pack("!Q", len(payload))
    connection.sendall(header + payload)


def app_server(socket_path, methods, ready):
    server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    server.bind(socket_path)
    server.listen(1)
    ready.set()
    connection, _ = server.accept()
    request = b""
    while b"\r\n\r\n" not in request:
        request += connection.recv(4096)
    headers = {}
    for line in request.decode().split("\r\n")[1:]:
        if ":" in line:
            name, value = line.split(":", 1)
            headers[name.lower()] = value.strip()
    accept = base64.b64encode(hashlib.sha1((headers["sec-websocket-key"] + WEBSOCKET_GUID).encode()).digest()).decode()
    connection.sendall(("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: " + accept + "\r\n\r\n").encode())
    reads = 0
    while True:
        try:
            message = receive_websocket(connection)
        except EOFError:
            break
        method = message.get("method")
        if method == "initialized":
            continue
        methods.append(method)
        request_id = message["id"]
        if method == "initialize":
            result = {"userAgent": "fake"}
        elif method == "thread/loaded/list":
            result = {"data": [THREAD_ID]}
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
            send_websocket(connection, {"id": request_id, "error": {"code": -32601, "message": "unsupported"}})
            continue
        send_websocket(connection, {"method": "thread/status/changed", "params": {"threadId": THREAD_ID}})
        send_websocket(connection, {"id": request_id, "result": result})
    connection.close()
    server.close()


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


def gateway(socket_path, results, ready):
    server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    server.bind(socket_path)
    server.listen(1)
    ready.set()
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
        socket_directory = root / "Library/Application Support/com.yonder.desktop"
        socket_directory.mkdir(parents=True)
        yonder_socket = socket_directory / "agent.sock"
        app_server_socket = root / "codex.sock"
        results, methods = [], []
        app_ready, gateway_ready = threading.Event(), threading.Event()
        app_thread = threading.Thread(target=app_server, args=(str(app_server_socket), methods, app_ready), daemon=True)
        gateway_thread = threading.Thread(target=gateway, args=(str(yonder_socket), results, gateway_ready), daemon=True)
        app_thread.start()
        gateway_thread.start()
        app_ready.wait(5)
        gateway_ready.wait(5)
        environment = os.environ.copy()
        environment.update({
            "HOME": str(root),
            "YONDER_AGENT_ID": "codex-contract",
            "YONDER_CODEX_THREAD_ID": THREAD_ID,
            "YONDER_CODEX_APP_SERVER_SOCKET": str(app_server_socket),
        })
        process = subprocess.run([str(BINARY), "agent-bridge"], env=environment, capture_output=True, text=True, timeout=15)
        gateway_thread.join(timeout=5)
        app_thread.join(timeout=5)
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
