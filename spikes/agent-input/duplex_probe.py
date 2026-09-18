#!/usr/bin/env python3
"""验证同一Agent连接上的主动输入、确认与失败语义。"""

import json
import socket
import tempfile
import threading
import time
from pathlib import Path

MAX_FRAME_BYTES = 64 * 1024
MAX_CONTENT_BYTES = 16 * 1024


def send_frame(stream, value):
    frame = json.dumps(value, ensure_ascii=False, separators=(",", ":")).encode() + b"\n"
    assert len(frame) <= MAX_FRAME_BYTES
    stream.sendall(frame)


def read_frame(stream):
    data = bytearray()
    while not data.endswith(b"\n"):
        chunk = stream.recv(4096)
        if not chunk:
            raise EOFError("连接在完整帧前关闭")
        data.extend(chunk)
        assert len(data) <= MAX_FRAME_BYTES
    return json.loads(data)


def agent(path, content, behavior, deliveries, errors):
    try:
        with socket.socket(socket.AF_UNIX) as stream:
            stream.connect(path)
            send_frame(stream, {"method": "gateway.hello", "id": "hello-1", "params": {
                "agent_id": "fixture-agent", "session_id": "session-1",
                "offered_capabilities": ["user_input"],
            }})
            assert read_frame(stream)["result"]["accepted"] is True
            seen = set()
            request_count = 0
            while True:
                request = read_frame(stream)
                request_count += 1
                assert request["method"] == "agent.input"
                assert request["params"]["session_id"] == "session-1"
                assert request["params"]["content"] == content
                if behavior == "disconnect":
                    return
                if behavior == "timeout":
                    time.sleep(0.1)
                    return
                if behavior == "reject":
                    send_frame(stream, {"id": request["id"], "result": {"accepted": False}})
                    return
                input_id = request["params"]["input_id"]
                if input_id not in seen:
                    deliveries.append(input_id)
                    seen.add(input_id)
                send_frame(stream, {"id": request["id"], "result": {"accepted": True}})
                if behavior != "duplicate" or request_count == 2:
                    return
    except BaseException as error:
        errors.append(error)


def run_case(behavior):
    content = "帮我整理今天的任务"
    assert len(content.encode()) <= MAX_CONTENT_BYTES
    deliveries, errors = [], []
    with tempfile.TemporaryDirectory(prefix="yonder-agent-input-") as directory:
        path = str(Path(directory) / "agent.sock")
        with socket.socket(socket.AF_UNIX) as listener:
            listener.bind(path)
            listener.listen(1)
            worker = threading.Thread(target=agent, args=(path, content, behavior, deliveries, errors))
            worker.start()
            stream, _ = listener.accept()
            with stream:
                hello = read_frame(stream)
                assert hello["params"]["offered_capabilities"] == ["user_input"]
                send_frame(stream, {"id": hello["id"], "result": {"accepted": True}})
                deadline = int(time.time() * 1000) + 5_000
                request = {"method": "agent.input", "id": "input-1", "params": {
                    "input_id": "input-1", "session_id": hello["params"]["session_id"],
                    "source": "voice", "content": content, "created_at": deadline - 5_000,
                    "deadline": deadline,
                }}
                stream.settimeout(0.05)
                send_frame(stream, request)
                try:
                    ack = read_frame(stream)
                    outcome = "accepted" if ack["result"]["accepted"] else "rejected"
                except (EOFError, socket.timeout):
                    outcome = "unknown"
                if behavior == "duplicate":
                    send_frame(stream, request)
                    assert read_frame(stream)["result"]["accepted"] is True
            worker.join(timeout=1)
            assert not worker.is_alive()
            assert not errors, errors
    return outcome, deliveries


def main():
    accepted, delivered = run_case("accepted")
    assert accepted == "accepted" and delivered == ["input-1"]
    rejected, delivered = run_case("reject")
    assert rejected == "rejected" and delivered == []
    for behavior in ("disconnect", "timeout"):
        outcome, delivered = run_case(behavior)
        assert outcome == "unknown" and delivered == []
    duplicate, delivered = run_case("duplicate")
    assert duplicate == "accepted" and delivered == ["input-1"]
    print(json.dumps({"transport": "uds", "session": "bound", "accepted": True,
                      "rejected": True, "unknown": ["disconnect", "timeout"],
                      "duplicate_delivery_count": len(delivered)}))


if __name__ == "__main__":
    main()
