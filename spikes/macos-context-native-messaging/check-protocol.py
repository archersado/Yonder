#!/usr/bin/env python3
"""验证macOS Native Host协议；只输出结构化断言，不输出测试正文。"""

import json
import pathlib
import struct
import subprocess
import tempfile


ROOT = pathlib.Path(__file__).resolve().parent


def encode(value):
    payload = json.dumps(value, ensure_ascii=False, separators=(",", ":")).encode()
    return struct.pack("=I", len(payload)) + payload


def read_frame(stream):
    header = stream.read(4)
    assert len(header) == 4
    length = struct.unpack("=I", header)[0]
    payload = stream.read(length)
    assert len(payload) == length
    return json.loads(payload)


def main():
    with tempfile.TemporaryDirectory(prefix="ycx1-", dir="/tmp") as directory:
        host = pathlib.Path(directory) / "native-host"
        subprocess.run(["xcrun", "swiftc", str(ROOT / "native-host.swift"), "-o", str(host)], check=True)
        process = subprocess.Popen([str(host)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        first = encode({"type": "protocol.utf8", "sample": "验证"})
        process.stdin.write(first[:3])
        process.stdin.flush()
        process.stdin.write(first[3:] + encode({"type": "tab.activated"}))
        process.stdin.flush()
        responses = [read_frame(process.stdout), read_frame(process.stdout)]
        process.stdin.close()
        return_code = process.wait(timeout=5)
        stderr = process.stderr.read()

        oversized = subprocess.Popen([str(host)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        _, oversized_stderr = oversized.communicate(struct.pack("=I", 1024 * 1024 + 1), timeout=5)

        invalid = subprocess.Popen([str(host)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        _, invalid_stderr = invalid.communicate(struct.pack("=I", 1) + b"{", timeout=5)

        result = {
            "passed": responses == [{"accepted": True, "sequence": 1}, {"accepted": True, "sequence": 2}]
                and return_code == 0 and not stderr
                and oversized.returncode == 2 and oversized_stderr == b"frame_too_large\n"
                and invalid.returncode == 2 and invalid_stderr == b"message_invalid\n",
            "utf8_fragmented": responses[0].get("accepted") is True,
            "consecutive_frames": len(responses) == 2,
            "stdout_protocol_only": not stderr,
            "oversized_rejected": oversized.returncode == 2,
            "invalid_json_rejected": invalid.returncode == 2,
        }
        print(json.dumps(result, ensure_ascii=False, sort_keys=True))
        if not result["passed"]:
            raise SystemExit(1)


if __name__ == "__main__":
    main()
