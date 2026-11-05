#!/usr/bin/env python3
"""在独立目录准备 CU-S5 固定上游源码；拒绝未声明改动。"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parent
REVISION = "d3dc190afbf34cd2377252398ddd5c30efbd093a"
DEFAULT_SOURCE = ROOT.parent / "agent-desktop-driver" / "target" / "patched-agent-desktop"
DESTINATION = ROOT / "target" / "patched-agent-desktop"
PATCHES = (
    ROOT.parent / "agent-desktop-driver" / "patches" / "0001-reject-inconclusive-pointer-hit-test.patch",
)
ALLOWED_CHANGES = {
    "crates/core/src/actionability/receives_events.rs",
    "crates/core/src/adapter/hit_test_tests.rs",
    "crates/core/src/commands/point_resolve_tests.rs",
}
SAMPLES = (
    ROOT.parent.parent
    / "openspec"
    / "changes"
    / "cu-s5-agent-desktop-v2-spike"
    / "sample-definitions-v2.json"
)
STAMP = ".yonder-v2-source.json"


def run(*args: str, cwd: Path | None = None) -> str:
    return subprocess.run(
        args, cwd=cwd, check=True, capture_output=True, text=True
    ).stdout.strip()


def file_hash(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tree_hash(root: Path) -> str:
    digest = hashlib.sha256()
    names = run("git", "ls-files", "-z", cwd=root).split("\0")
    for name in sorted(filter(None, names)):
        path = root / name
        encoded = name.encode()
        digest.update(len(encoded).to_bytes(4, "big"))
        digest.update(encoded)
        digest.update(
            b"symlink:" + os.readlink(path).encode()
            if path.is_symlink()
            else path.read_bytes()
        )
    return digest.hexdigest()


def patch_set_hash() -> str:
    digest = hashlib.sha256()
    for patch in PATCHES:
        data = patch.read_bytes()
        digest.update(patch.name.encode())
        digest.update(b"\0")
        digest.update(data)
        digest.update(b"\0")
    return digest.hexdigest()


def expected_stamp(root: Path) -> dict[str, object]:
    return {
        "schema_version": "yonder-agent-desktop-v2-source/v1",
        "source_revision": REVISION,
        "patches": [
            {"name": patch.name, "sha256": file_hash(patch)} for patch in PATCHES
        ],
        "patch_set_sha256": patch_set_hash(),
        "source_tree_sha256": tree_hash(root),
        "sample_definitions_sha256": file_hash(SAMPLES),
        "native_dispatch_admitted": False,
    }


def verify_existing() -> dict[str, object] | None:
    if not DESTINATION.exists():
        return None
    stamp_path = DESTINATION / STAMP
    if not stamp_path.is_file():
        raise RuntimeError("v2 源码目录缺少来源 stamp；拒绝复用")
    stamp = json.loads(stamp_path.read_text(encoding="utf-8"))
    if run("git", "rev-parse", "HEAD", cwd=DESTINATION) != REVISION:
        raise RuntimeError("v2 上游 revision 漂移")
    changed = set(run("git", "diff", "--name-only", cwd=DESTINATION).splitlines())
    if changed != ALLOWED_CHANGES:
        raise RuntimeError(f"v2 已准备源码的改动文件集合不符：{sorted(changed)}")
    untracked = set(
        run("git", "ls-files", "--others", "--exclude-standard", cwd=DESTINATION).splitlines()
    )
    if untracked != {STAMP}:
        raise RuntimeError(f"v2 已准备源码出现额外未跟踪文件：{sorted(untracked)}")
    if stamp != expected_stamp(DESTINATION):
        raise RuntimeError("v2 源码、补丁或样本哈希不匹配；拒绝覆盖")
    return stamp


def prepare(source: str) -> dict[str, object]:
    existing = verify_existing()
    if existing is not None:
        return existing
    DESTINATION.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="agent-desktop-v2-", dir=DESTINATION.parent) as temp:
        checkout = Path(temp) / "source"
        run("git", "clone", "--quiet", "--no-checkout", "--", source, str(checkout))
        run("git", "checkout", "--quiet", "--detach", REVISION, cwd=checkout)
        if run("git", "rev-parse", "HEAD", cwd=checkout) != REVISION:
            raise RuntimeError("固定上游 revision 不存在")
        for patch in PATCHES:
            run("git", "apply", "--check", str(patch), cwd=checkout)
            run("git", "apply", str(patch), cwd=checkout)
        changed = set(run("git", "diff", "--name-only", cwd=checkout).splitlines())
        if changed != ALLOWED_CHANGES:
            raise RuntimeError(f"补丁改动文件集合不符：{sorted(changed)}")
        stamp = expected_stamp(checkout)
        (checkout / STAMP).write_text(
            json.dumps(stamp, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
        )
        os.rename(checkout, DESTINATION)
    return stamp


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", default=str(DEFAULT_SOURCE), help="本地或远端 Git 来源")
    args = parser.parse_args()
    print(json.dumps(prepare(args.source), ensure_ascii=False, indent=2))
