#!/usr/bin/env python3
"""只读核对固定 v1 源码中的 v2 门禁缺口；绝不派发桌面输入。"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import prepare_upstream as prep


HERE = Path(__file__).resolve().parent
SOURCE = prep.DESTINATION
REVISION = "d3dc190afbf34cd2377252398ddd5c30efbd093a"


def read(relative: str) -> str:
    return (SOURCE / relative).read_text(encoding="utf-8")


def seams(root: str, needle: str) -> list[str]:
    base = SOURCE / root
    return sorted(
        path.relative_to(SOURCE).as_posix()
        for path in base.rglob("*.rs")
        if not path.name.endswith("tests.rs") and needle in path.read_text(encoding="utf-8")
    )


def run() -> dict[str, object]:
    manifest = json.loads((HERE / "source-manifest.json").read_text(encoding="utf-8"))
    stamp = prep.verify_existing()
    provenance = (
        stamp is not None
        and manifest["source_revision"] == REVISION
        and stamp["source_revision"] == REVISION
        and manifest["patch_set_sha256"] == stamp["patch_set_sha256"]
        and manifest["patches"] == stamp["patches"]
        and manifest["patched_source_tree_sha256"] == stamp["source_tree_sha256"]
        and manifest["sample_definitions_sha256"] == stamp["sample_definitions_sha256"]
    )
    if not provenance:
        raise RuntimeError("v2 固定源码来源校验失败；停止只读审计")

    helpers = read("crates/core/src/commands/helpers.rs")
    wait = read("crates/core/src/ref_action/ref_action_wait.rs")
    dispatch = read("crates/core/src/ref_action/ref_action_single.rs")
    windows_mouse = read("crates/windows/src/input/mouse/mouse_send.rs")
    windows_key = read("crates/windows/src/input/keyboard/keyboard_send.rs")
    windows_shell = read("crates/windows/src/system/shell/shell_surface_raise.rs")
    mac_mouse = read("crates/macos/src/input/mouse/mod.rs")
    mac_key = read("crates/macos/src/input/keyboard/keyboard_event.rs")
    win_sendinput = seams("crates/windows/src", "SendInput(")
    mac_hid_post = seams("crates/macos/src", ".post(CGEventTapLocation::HID)")
    mac_pid_post = seams("crates/macos/src", ".post_to_pid(")
    expected_win = [
        "crates/windows/src/input/keyboard/keyboard_send.rs",
        "crates/windows/src/input/mouse/mouse_send.rs",
        "crates/windows/src/system/shell/shell_surface_raise.rs",
    ]
    expected_mac_hid = [
        "crates/macos/src/input/keyboard/keyboard_event.rs",
        "crates/macos/src/input/mouse/mod.rs",
        "crates/macos/src/input/mouse/mouse_drag.rs",
        "crates/macos/src/input/mouse/mouse_move.rs",
        "crates/macos/src/input/mouse/mouse_scroll.rs",
    ]
    checks = {
        "ref_store_reloaded_for_action": "load_ref_entry(&args.ref_id" in helpers,
        "action_re_resolves_saved_ref": "resolve_saved_ref(adapter, entry" in helpers,
        "action_auto_wait_can_retry": "resolves_again(&error, deadline)" in wait,
        "dispatch_drops_landing_handle": ".map(|(result, _)| result)" in dispatch,
        "windows_mouse_untagged": "dwExtraInfo: 0" in windows_mouse,
        "windows_keyboard_untagged": "dwExtraInfo: 0" in windows_key,
        "windows_shell_untagged": "dwExtraInfo: 0" in windows_shell,
        "mac_mouse_posts_event": ".post(CGEventTapLocation::HID)" in mac_mouse,
        "mac_keyboard_posts_event": ".post(" in mac_key,
        "mac_mouse_has_attempt_tag": "EventSourceUserData" in mac_mouse,
        "mac_keyboard_has_attempt_tag": "EventSourceUserData" in mac_key,
        "synthesis_seams_match_reviewed_inventory": (
            win_sendinput == expected_win
            and mac_hid_post == expected_mac_hid
            and mac_pid_post == ["crates/macos/src/input/keyboard/keyboard_event.rs"]
        ),
    }
    return {
        "schema_version": "yonder-agent-desktop-v2-source-gate/v1",
        "source_revision": REVISION,
        "source_tree_sha256": stamp["source_tree_sha256"],
        "patch_set_sha256": stamp["patch_set_sha256"],
        "inspection_scope": "known_seams_only",
        "synthesis_seams": {
            "windows_sendinput": win_sendinput,
            "macos_hid_post": mac_hid_post,
            "macos_pid_post": mac_pid_post,
        },
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
