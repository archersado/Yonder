#!/usr/bin/env python3
"""验证会话内临时附件的有界分块与失败清理；不读写真实截图。"""

import base64
import hashlib
import json
from pathlib import Path


MAX_FRAME_BYTES = 64 * 1024
MAX_ATTACHMENT_BYTES = 4 * 1024 * 1024
CHUNK_BYTES = 47 * 1024


class Rejected(Exception):
    pass


class Receiver:
    def __init__(self):
        self.pending = {}
        self.largest_frame = 0

    def _frame(self, value):
        size = len(json.dumps(value, separators=(",", ":")).encode())
        self.largest_frame = max(self.largest_frame, size)
        if size > MAX_FRAME_BYTES:
            raise Rejected("frame-too-large")

    def begin(self, session, attachment, size, digest, deadline):
        self._frame({"kind": "begin", "session": session, "attachment": attachment,
                     "mime": "image/png", "bytes": size, "sha256": digest, "deadline": deadline})
        if not 0 < size <= MAX_ATTACHMENT_BYTES:
            raise Rejected("attachment-too-large")
        self.pending[(session, attachment)] = {
            "size": size, "digest": digest, "deadline": deadline,
            "next": 0, "data": bytearray(), "ready": False,
        }

    def chunk(self, session, attachment, sequence, raw):
        key = (session, attachment)
        state = self.pending.get(key)
        try:
            encoded = base64.b64encode(raw).decode("ascii")
            self._frame({"kind": "chunk", "session": session, "attachment": attachment,
                         "sequence": sequence, "data": encoded})
            if state is None or sequence != state["next"]:
                raise Rejected("chunk-out-of-order")
            if len(state["data"]) + len(raw) > state["size"]:
                raise Rejected("attachment-overflow")
            state["data"].extend(raw)
            state["next"] += 1
        except Rejected:
            self.pending.pop(key, None)
            raise

    def finish(self, session, attachment):
        key = (session, attachment)
        state = self.pending.get(key)
        try:
            self._frame({"kind": "finish", "session": session, "attachment": attachment})
            if state is None or len(state["data"]) != state["size"]:
                raise Rejected("attachment-incomplete")
            if hashlib.sha256(state["data"]).hexdigest() != state["digest"]:
                raise Rejected("attachment-hash-mismatch")
            state["ready"] = True
        except Rejected:
            self.pending.pop(key, None)
            raise

    def consume(self, session, attachment):
        key = (session, attachment)
        state = self.pending.get(key)
        self._frame({"kind": "input", "session": session, "attachment": attachment})
        if state is None or not state["ready"]:
            raise Rejected("attachment-unavailable")
        self.pending.pop(key)

    def expire(self, now):
        self.pending = {key: value for key, value in self.pending.items() if value["deadline"] > now}

    def disconnect(self, session):
        self.pending = {key: value for key, value in self.pending.items() if key[0] != session}

    def buffered_bytes(self):
        return sum(len(value["data"]) for value in self.pending.values())


def transfer(receiver, session, attachment, data, deadline=1000, digest=None):
    receiver.begin(session, attachment, len(data), digest or hashlib.sha256(data).hexdigest(), deadline)
    chunks = 0
    for offset in range(0, len(data), CHUNK_BYTES):
        receiver.chunk(session, attachment, chunks, data[offset:offset + CHUNK_BYTES])
        chunks += 1
    receiver.finish(session, attachment)
    receiver.consume(session, attachment)
    return chunks


def rejected(reason, action):
    try:
        action()
    except Rejected as error:
        return str(error) == reason
    return False


def main():
    receiver = Receiver()
    one_chunks = transfer(receiver, "session-a", "one", b"x")
    cross = b"YONDER-NON-SENSITIVE-FIXTURE" * 4000
    cross_chunks = transfer(receiver, "session-a", "cross", cross)
    boundary = bytes(range(256)) * (MAX_ATTACHMENT_BYTES // 256)
    boundary_chunks = transfer(receiver, "session-a", "boundary", boundary)

    oversize = rejected("attachment-too-large", lambda: receiver.begin(
        "session-a", "oversize", MAX_ATTACHMENT_BYTES + 1, "0" * 64, 1000))

    receiver.begin("session-a", "unordered", 2, hashlib.sha256(b"ab").hexdigest(), 1000)
    unordered = rejected("chunk-out-of-order", lambda: receiver.chunk("session-a", "unordered", 1, b"a"))

    bad = b"fixture"
    receiver.begin("session-a", "bad-hash", len(bad), "0" * 64, 1000)
    receiver.chunk("session-a", "bad-hash", 0, bad)
    hash_mismatch = rejected("attachment-hash-mismatch", lambda: receiver.finish("session-a", "bad-hash"))

    isolated = b"isolated"
    receiver.begin("session-a", "isolated", len(isolated), hashlib.sha256(isolated).hexdigest(), 1000)
    receiver.chunk("session-a", "isolated", 0, isolated)
    receiver.finish("session-a", "isolated")
    cross_session = rejected("attachment-unavailable", lambda: receiver.consume("session-b", "isolated"))
    cross_session_preserved = receiver.buffered_bytes() == len(isolated)
    receiver.disconnect("session-a")
    owner_disconnect_cleared = receiver.buffered_bytes() == 0

    receiver.begin("session-a", "timeout", 1, hashlib.sha256(b"x").hexdigest(), 10)
    receiver.chunk("session-a", "timeout", 0, b"x")
    receiver.expire(10)
    timeout_cleared = receiver.buffered_bytes() == 0

    receiver.begin("session-a", "disconnect", 1, hashlib.sha256(b"x").hexdigest(), 1000)
    receiver.chunk("session-a", "disconnect", 0, b"x")
    receiver.disconnect("session-a")
    disconnect_cleared = receiver.buffered_bytes() == 0

    result = {
        "passed": all((oversize, unordered, hash_mismatch, cross_session, cross_session_preserved,
                       owner_disconnect_cleared,
                       timeout_cleared, disconnect_cleared, receiver.buffered_bytes() == 0,
                       receiver.largest_frame <= MAX_FRAME_BYTES)),
        "limits": {"frame_bytes": MAX_FRAME_BYTES, "attachment_bytes": MAX_ATTACHMENT_BYTES,
                   "chunk_bytes": CHUNK_BYTES},
        "success_samples": {
            "one_byte_chunks": one_chunks,
            "cross_chunk_bytes": len(cross), "cross_chunk_count": cross_chunks,
            "boundary_bytes": len(boundary), "boundary_chunk_count": boundary_chunks,
        },
        "rejections": {"oversize": oversize, "out_of_order": unordered,
                       "hash_mismatch": hash_mismatch, "cross_session": cross_session,
                       "cross_session_preserved_owner_attachment": cross_session_preserved},
        "cleanup": {"timeout": timeout_cleared, "disconnect": disconnect_cleared,
                    "owner_disconnect_after_cross_session": owner_disconnect_cleared,
                    "final_buffered_bytes": receiver.buffered_bytes()},
        "largest_frame_bytes": receiver.largest_frame,
        "contains_attachment_content": False,
    }
    output = Path(__file__).with_name("evidence") / "result.json"
    output.parent.mkdir(exist_ok=True)
    output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, ensure_ascii=False, separators=(",", ":")))
    raise SystemExit(0 if result["passed"] else 1)


if __name__ == "__main__":
    main()
