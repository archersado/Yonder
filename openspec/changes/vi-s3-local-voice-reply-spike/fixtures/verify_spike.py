#!/usr/bin/env python3
import json
import platform
import shutil
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent


def command_path(name: str) -> str:
    found = shutil.which(name)
    if found:
        return found
    candidate = Path.home() / ".cargo" / "bin" / name
    if candidate.is_file():
        return str(candidate)
    raise RuntimeError(f"missing_tool:{name}")


def run_json(command: list[str]) -> dict:
    completed = subprocess.run(command, check=True, capture_output=True, text=True)
    return json.loads(completed.stdout)


def main() -> None:
    if platform.system() != "Darwin":
        raise RuntimeError("unsupported_platform")

    with tempfile.TemporaryDirectory(prefix="yonder-vi-s3-") as temporary:
        output = Path(temporary)
        ordered_binary = output / "ordered_reply_probe"
        native_binary = output / "macos_voice_reply_probe"

        subprocess.run(
            [
                command_path("rustc"),
                str(ROOT / "ordered_reply_probe.rs"),
                "-o",
                str(ordered_binary),
            ],
            check=True,
        )
        ordered = run_json([str(ordered_binary)])
        required_ordered = (
            "ordered_release",
            "skipped_sequence_progresses",
            "stale_generation_rejected",
            "bounded_capacity",
        )
        assert all(ordered.get(field) is True for field in required_ordered)

        subprocess.run(
            [
                "xcrun",
                "swiftc",
                str(ROOT / "macos_voice_reply_probe.swift"),
                "-framework",
                "AVFoundation",
                "-o",
                str(native_binary),
            ],
            check=True,
        )
        native_runs = [run_json([str(native_binary)]) for _ in range(3)]
        for result in native_runs:
            assert result.get("result") == "candidate_pass"
            assert result.get("strict_callback_order") is True
            assert result.get("stopped_speaking") is True
            assert result.get("terminal_callback_once") is True
            assert result.get("released_after_probe") is True
            assert result.get("contains_text_or_voice_name") is False

        summary = {
            "kind": "vi-s3-macos-spike-verification",
            "runs": len(native_runs),
            "ordered_contract": "pass",
            "native_candidate": "pass",
            "installed_chinese_voice_count": native_runs[0][
                "installed_chinese_voice_count"
            ],
            "queue_elapsed_ms_range": [
                min(run["queue_elapsed_ms"] for run in native_runs),
                max(run["queue_elapsed_ms"] for run in native_runs),
            ],
            "stop_elapsed_ms_range": [
                min(run["stop_elapsed_ms"] for run in native_runs),
                max(run["stop_elapsed_ms"] for run in native_runs),
            ],
            "requires_generation_guard": any(
                run["requires_generation_guard"] for run in native_runs
            ),
            "offline_verified": False,
            "output_device_switch_verified": False,
            "contains_text_or_voice_name": False,
        }
        print(json.dumps(summary, ensure_ascii=False, sort_keys=True))


if __name__ == "__main__":
    main()
