"""在隔离临时根目录验证 macOS .app 新装、升级、失败回退与卸载。"""

import argparse
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys
import tempfile

from release import audit_macos_bundle, tree_sha256


class LifecycleError(ValueError):
    pass


def copy_application(source, destination):
    destination.parent.mkdir(parents=True, exist_ok=True)
    try:
        subprocess.run(
            ["/usr/bin/ditto", str(source), str(destination)],
            check=True,
            capture_output=True,
            text=True,
            timeout=120,
        )
    except (OSError, subprocess.CalledProcessError, subprocess.TimeoutExpired) as error:
        raise LifecycleError("应用复制失败") from error


def verify_signature(bundle):
    try:
        subprocess.run(
            ["/usr/bin/codesign", "--verify", "--deep", "--strict", str(bundle)],
            check=True,
            capture_output=True,
            text=True,
            timeout=30,
        )
    except (OSError, subprocess.CalledProcessError, subprocess.TimeoutExpired) as error:
        raise LifecycleError("应用签名校验失败") from error


def install_application(source, destination, copier=copy_application, verifier=verify_signature):
    if destination.exists():
        raise LifecycleError("安装位置已存在应用")
    staging = destination.with_name(f".{destination.name}.installing")
    if staging.exists():
        raise LifecycleError("安装暂存位置已存在")
    copier(source, staging)
    try:
        verifier(staging)
        os.replace(staging, destination)
        verifier(destination)
    except Exception:
        if destination.exists():
            os.replace(destination, destination.with_name(f".{destination.name}.failed"))
        raise
    return tree_sha256(destination)


def replace_application(
    installed,
    candidate,
    copier=copy_application,
    verifier=verify_signature,
    failure_stage=None,
):
    if not installed.is_dir():
        raise LifecycleError("没有可升级的旧应用")
    staging = installed.with_name(f".{installed.name}.upgrade")
    backup = installed.with_name(f".{installed.name}.previous")
    failed = installed.with_name(f".{installed.name}.failed")
    if staging.exists() or backup.exists() or failed.exists():
        raise LifecycleError("升级暂存位置不为空")
    copier(candidate, staging)
    verifier(staging)
    previous_hash = tree_sha256(installed)
    os.replace(installed, backup)
    try:
        if failure_stage == "before-placement":
            raise LifecycleError("注入候选落位前失败")
        os.replace(staging, installed)
        if failure_stage == "after-placement":
            raise LifecycleError("注入候选落位后失败")
        if failure_stage is not None:
            raise LifecycleError("未知失败注入阶段")
        verifier(installed)
    except Exception:
        if installed.exists():
            os.replace(installed, failed)
        os.replace(backup, installed)
        if tree_sha256(installed) != previous_hash:
            raise LifecycleError("旧应用恢复后不一致")
        raise
    return backup, previous_hash, tree_sha256(installed)


def uninstall_application(installed, trash):
    if not installed.is_dir() or trash.exists():
        raise LifecycleError("卸载位置无效")
    trash.parent.mkdir(parents=True, exist_ok=True)
    os.replace(installed, trash)
    if installed.exists() or not trash.is_dir():
        raise LifecycleError("应用未移动到可恢复位置")


def create_user_data(root):
    data = root / "Library/Application Support/com.yonder.desktop"
    attachments = data / "attachments"
    attachments.mkdir(parents=True)
    database = data / "tasks.db"
    with sqlite3.connect(database) as connection:
        connection.execute("CREATE TABLE fixture(id TEXT PRIMARY KEY, state TEXT NOT NULL)")
        connection.execute("INSERT INTO fixture VALUES ('task-fixture', 'completed')")
    (data / "tasks.db-wal").write_bytes(b"synthetic-wal-fixture")
    (data / "tasks.db-shm").write_bytes(b"synthetic-shm-fixture")
    (attachments / "artifact.bin").write_bytes(b"synthetic-attachment-fixture")
    return data


def exercise(previous, candidate, expected_commit, channel):
    if sys.platform != "darwin":
        raise LifecycleError("macOS安装生命周期只能在macOS验证")
    verify_signature(previous)
    try:
        release_contract = json.loads(
            (candidate / "Contents/Resources/release-contract.json").read_text(encoding="utf-8")
        )
        expected_version = release_contract["version"]
    except (OSError, json.JSONDecodeError, KeyError, TypeError) as error:
        raise LifecycleError("候选发布契约不可用") from error
    candidate_audit = audit_macos_bundle(
        candidate,
        expected_channel=channel,
        expected_commit=expected_commit,
        expected_version=expected_version,
        verify_signature=True,
    )
    previous_hash = tree_sha256(previous)
    candidate_hash = tree_sha256(candidate)
    if previous_hash == candidate_hash:
        raise LifecycleError("升级样本的新旧应用必须不同")

    with tempfile.TemporaryDirectory(prefix="yonder-release-lifecycle-") as directory:
        fixture = Path(directory)

        fresh_app = fixture / "fresh/Applications/Yonda.app"
        fresh_hash = install_application(candidate, fresh_app)
        if fresh_hash != candidate_hash:
            raise LifecycleError("新装应用与候选不一致")

        for failure_stage in ["before-placement", "after-placement"]:
            rollback_root = fixture / failure_stage
            rollback_app = rollback_root / "Applications/Yonda.app"
            install_application(previous, rollback_app)
            rollback_data = create_user_data(rollback_root)
            rollback_data_hash = tree_sha256(rollback_data)
            try:
                replace_application(
                    rollback_app, candidate, failure_stage=failure_stage
                )
            except LifecycleError as error:
                if str(error) != f"注入候选落位{'前' if failure_stage == 'before-placement' else '后'}失败":
                    raise
            else:
                raise LifecycleError("失败注入未阻止升级")
            if tree_sha256(rollback_app) != previous_hash:
                raise LifecycleError("失败回退没有恢复旧应用")
            if tree_sha256(rollback_data) != rollback_data_hash:
                raise LifecycleError("失败回退改变了用户数据")

        upgrade_app = fixture / "upgrade/Applications/Yonda.app"
        install_application(previous, upgrade_app)
        upgrade_data = create_user_data(fixture / "upgrade")
        user_data_hash = tree_sha256(upgrade_data)
        backup, installed_previous_hash, installed_candidate_hash = replace_application(
            upgrade_app, candidate
        )
        if installed_previous_hash != previous_hash or tree_sha256(backup) != previous_hash:
            raise LifecycleError("升级回退副本与旧应用不一致")
        if installed_candidate_hash != candidate_hash:
            raise LifecycleError("升级后应用与候选不一致")
        if tree_sha256(upgrade_data) != user_data_hash:
            raise LifecycleError("升级改变了用户数据")
        uninstall_application(upgrade_app, fixture / "upgrade/Trash/Yonda.app")
        if tree_sha256(upgrade_data) != user_data_hash:
            raise LifecycleError("卸载改变了用户数据")

    return {
        "platform": "macOS",
        "scope": "isolated_install_lifecycle",
        "channel": channel,
        "build_commit": candidate_audit["build_commit"],
        "version": expected_version,
        "previous_app_sha256": previous_hash,
        "candidate_app_sha256": candidate_hash,
        "fresh_install_passed": True,
        "upgrade_passed": True,
        "rollback_before_candidate_placement_passed": True,
        "rollback_after_candidate_placement_passed": True,
        "uninstall_moved_to_recoverable_location": True,
        "user_data_preserved": True,
        "synthetic_user_data_sha256": user_data_hash,
        "real_user_paths_accessed": False,
        "user_content_recorded": False,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--previous-app", type=Path, required=True)
    parser.add_argument("--candidate-app", type=Path, required=True)
    parser.add_argument("--expected-commit", required=True)
    parser.add_argument("--channel", choices=["dev", "stable"], required=True)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    try:
        result = exercise(
            args.previous_app.resolve(),
            args.candidate_app.resolve(),
            args.expected_commit,
            args.channel,
        )
    except (LifecycleError, ValueError, OSError, KeyError) as error:
        print(f"安装生命周期验证失败：{error}", file=sys.stderr)
        raise SystemExit(1) from error
    rendered = json.dumps(result, ensure_ascii=False, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8")
    else:
        print(rendered, end="")


if __name__ == "__main__":
    main()
