#!/usr/bin/env python3
"""只读核对固定 v1 源码中的 v2 门禁缺口；绝不派发桌面输入。"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess


HERE = Path(__file__).resolve().parent
V1 = HERE.parent / "agent-desktop-driver"
SOURCE = V1 / "target" / "patched-agent-desktop"
REVISION = "d3dc190afbf34cd2377252398ddd5c30efbd093a"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args: str, cwd: Path) -> str:
    return subprocess.run(
        ["git", *args], cwd=cwd, check=True, capture_output=True, text=True
    ).stdout.strip()


def source_hash(root: Path) -> str:
    digest = hashlib.sha256()
    for name in sorted(filter(None, git("ls-files", "-z", cwd=root).split("\0"))):
        path = root / name
        encoded = name.encode()
        digest.update(len(encoded).to_bytes(4, "big"))
        digest.update(encoded)
        digest.update(b"symlink:" + os.readlink(path).encode() if path.is_symlink() else path.read_bytes())
    return digest.hexdigest()


def read(relative: str) -> str:
    return (SOURCE / relative).read_text(encoding="utf-8")


def run() -> dict[str, object]:
    manifest = json.loads((V1 / "build-manifest.json").read_text(encoding="utf-8"))
    stamp = json.loads((SOURCE / ".yonder-patched-source.json").read_text(encoding="utf-8"))
    provenance = (
        git("rev-parse", "HEAD", cwd=SOURCE) == REVISION
        and manifest["source_revision"] == REVISION
        and stamp["source_revision"] == REVISION
        and manifest["patch_sha256"] == stamp["patch_sha256"]
        and sha256(V1 / "patches" / "0001-reject-inconclusive-pointer-hit-test.patch")
        == stamp["patch_sha256"]
        and source_hash(SOURCE) == manifest["patched_source_tree_sha256"]
        == stamp["source_tree_sha256"]
    )
    if not provenance:
        raise RuntimeError("v1 固定源码来源校验失败；停止 v2 只读审计")

    helpers = read("crates/core/src/commands/helpers.rs")
    wait = read("crates/core/src/ref_action/ref_action_wait.rs")
    dispatch = read("crates/core/src/ref_action/ref_action_single.rs")
    windows_mouse = read("crates/windows/src/input/mouse/mouse_send.rs")
    windows_key = read("crates/windows/src/input/keyboard/keyboard_send.rs")
    mac_mouse = read("crates/macos/src/input/mouse/mod.rs")
    mac_key = read("crates/macos/src/input/keyboard/keyboard_event.rs")
    checks = {
        "ref_store_reloaded_for_action": "load_ref_entry(&args.ref_id" in helpers,
        "action_re_resolves_saved_ref": "resolve_saved_ref(adapter, entry" in helpers,
        "action_auto_wait_can_retry": "resolves_again(&error, deadline)" in wait,
        "dispatch_drops_landing_handle": ".map(|(result, _)| result)" in dispatch,
        "windows_mouse_untagged": "dwExtraInfo: 0" in windows_mouse,
        "windows_keyboard_untagged": "dwExtraInfo: 0" in windows_key,
        "mac_mouse_posts_event": ".post(CGEventTapLocation::HID)" in mac_mouse,
        "mac_keyboard_posts_event": ".post(" in mac_key,
        "mac_mouse_has_attempt_tag": "EventSourceUserData" in mac_mouse,
        "mac_keyboard_has_attempt_tag": "EventSourceUserData" in mac_key,
    }
    return {
        "schema_version": "yonder-agent-desktop-v2-source-gate/v1",
        "source_revision": REVISION,
        "source_tree_sha256": stamp["source_tree_sha256"],
        "patch_sha256": stamp["patch_sha256"],
        "inspection_scope": "known_seams_only",
        "checks": checks,
        "identity_gate_passed": False,
        "input_gate_passed": False,
        "native_dispatch_admitted": False,
        "reason": "原生对象交付连续性与持续有效的输入归属均未证明",
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="门禁未通过时返回 2")
    args = parser.parse_args()
    result = run()
    print(json.dumps(result, ensure_ascii=False, indent=2, sort_keys=True))
    if args.check and not result["native_dispatch_admitted"]:
        raise SystemExit(2)
