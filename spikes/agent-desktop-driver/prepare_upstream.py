#!/usr/bin/env python3
"""Prepare the pinned, patched agent-desktop source for the CU-S4 Spike build."""

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
SOURCE_URL = "https://github.com/vercel-labs/agent-desktop.git"
PATCH = ROOT / "patches" / "0001-reject-inconclusive-pointer-hit-test.patch"
DESTINATION = ROOT / "target" / "patched-agent-desktop"
STAMP = ".yonder-patched-source.json"
ALLOWED_CHANGES = {
    "crates/core/src/actionability/receives_events.rs",
    "crates/core/src/adapter/hit_test_tests.rs",
    "crates/core/src/commands/point_resolve_tests.rs",
}


def run(*args: str, cwd: Path | None = None) -> str:
    result = subprocess.run(args, cwd=cwd, check=True, capture_output=True, text=True)
    return result.stdout.strip()


def source_hash(root: Path) -> str:
    digest = hashlib.sha256()
    tracked = run("git", "ls-files", "-z", cwd=root).split("\0")
    for name in sorted(filter(None, tracked)):
        path = root / name
        relative = name.encode()
        digest.update(len(relative).to_bytes(4, "big"))
        digest.update(relative)
        if path.is_symlink():
            digest.update(b"symlink:")
            digest.update(os.readlink(path).encode())
        else:
            digest.update(path.read_bytes())
    return digest.hexdigest()


def expected_stamp(root: Path) -> dict[str, str]:
    return {
        "source_revision": REVISION,
        "patch_sha256": hashlib.sha256(PATCH.read_bytes()).hexdigest(),
        "source_tree_sha256": source_hash(root),
    }


def verify_existing() -> bool:
    if not DESTINATION.exists():
        return False
    stamp_path = DESTINATION / STAMP
    if not stamp_path.is_file():
        raise RuntimeError(f"prepared source has no provenance stamp: {DESTINATION}")
    stamp = json.loads(stamp_path.read_text(encoding="utf-8"))
    if stamp != expected_stamp(DESTINATION):
        raise RuntimeError(f"prepared source or patch changed; remove stale source: {DESTINATION}")
    if run("git", "rev-parse", "HEAD", cwd=DESTINATION) != REVISION:
        raise RuntimeError("prepared source revision changed")
    print(json.dumps(stamp, sort_keys=True))
    return True


def prepare(source: str) -> None:
    if verify_existing():
        return
    DESTINATION.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="agent-desktop-", dir=DESTINATION.parent) as temporary:
        staging = Path(temporary) / "source"
        run("git", "clone", "--quiet", "--no-checkout", "--", source, str(staging))
        run("git", "checkout", "--quiet", "--detach", REVISION, cwd=staging)
        if run("git", "rev-parse", "HEAD", cwd=staging) != REVISION:
            raise RuntimeError("upstream checkout did not resolve to the pinned revision")
        run("git", "apply", "--check", str(PATCH), cwd=staging)
        run("git", "apply", str(PATCH), cwd=staging)
        changed = set(run("git", "diff", "--name-only", cwd=staging).splitlines())
        if changed != ALLOWED_CHANGES:
            raise RuntimeError(f"unexpected patched files: {sorted(changed)}")
        stamp = expected_stamp(staging)
        (staging / STAMP).write_text(json.dumps(stamp, sort_keys=True) + "\n", encoding="utf-8")
        os.rename(staging, DESTINATION)
    print(json.dumps(stamp, sort_keys=True))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", default=SOURCE_URL, help="Git URL or local checkout")
    arguments = parser.parse_args()
    prepare(arguments.source)
