"""生成 Yonder 固定版本发布冻结清单；只使用 Python 标准库。"""

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys


def read_text(path):
    return path.read_text(encoding="utf-8")


def workspace_versions(root):
    workspace = read_text(root / "Cargo.toml")
    members = re.search(r"^members\s*=\s*\[(.*?)\]", workspace, re.MULTILINE | re.DOTALL)
    if not members:
        raise ValueError("找不到 Workspace members")
    versions = {}
    for member in re.findall(r'"([^"]+)"', members.group(1)):
        package = read_text(root / member / "Cargo.toml")
        name = re.search(r'^name\s*=\s*"([^"]+)"', package, re.MULTILINE)
        version = re.search(r'^version\s*=\s*"([^"]+)"', package, re.MULTILINE)
        if not name or not version:
            raise ValueError(f"Package 元数据不完整：{member}")
        versions[name.group(1)] = version.group(1)
    if len(set(versions.values())) != 1:
        raise ValueError(f"Workspace 版本不一致：{versions}")
    return next(iter(versions.values())), versions


def protocol_version(root):
    source = read_text(root / "crates/application/src/gateway.rs")
    match = re.search(
        r"pub const PROTOCOL_VERSION:\s*ProtocolVersion\s*=\s*ProtocolVersion\s*\{"
        r"\s*major:\s*([0-9]+),\s*minor:\s*([0-9]+)",
        source,
    )
    if not match:
        raise ValueError("找不到协议版本常量")
    return {"major": int(match[1]), "minor": int(match[2])}


def sqlite_schema_version(root):
    versions = []
    for path in (root / "crates/adapters/src").glob("task*.sql"):
        versions.extend(int(value) for value in re.findall(r"PRAGMA user_version=([0-9]+)", read_text(path)))
    if not versions:
        raise ValueError("找不到 SQLite schema 版本")
    return max(versions)


def driver_manifest(root, version):
    path = root / "apps/desktop/driver-manifest.json"
    manifest = json.loads(read_text(path))
    if manifest.get("version") != version:
        raise ValueError(f"Driver manifest 版本不一致：{manifest.get('version')}")
    drivers = manifest.get("drivers")
    if not isinstance(drivers, list) or not drivers:
        raise ValueError("Driver manifest 不能为空")
    return path, drivers


def release_contract(root, version, protocol, schema):
    path = root / "apps/desktop/release-contract.json"
    contract = json.loads(read_text(path))
    if contract.get("version") != version:
        raise ValueError(f"发布契约版本不一致：{contract.get('version')}")
    if contract.get("protocol") != protocol:
        raise ValueError(f"发布契约协议不一致：{contract.get('protocol')}")
    if contract.get("sqlite_schema") != schema:
        raise ValueError(f"发布契约 schema 不一致：{contract.get('sqlite_schema')}")
    return path


def git_commit(root):
    return subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def tree_sha256(root):
    digest = hashlib.sha256()
    for path in sorted(item for item in root.rglob("*") if item.is_file()):
        digest.update(path.relative_to(root).as_posix().encode("utf-8"))
        digest.update(b"\0")
        with path.open("rb") as stream:
            for block in iter(lambda: stream.read(1024 * 1024), b""):
                digest.update(block)
        digest.update(b"\0")
    return digest.hexdigest()


def collect_manifest(root, include_artifacts=False):
    version, packages = workspace_versions(root)
    desktop = json.loads(read_text(root / "apps/desktop/tauri.conf.json"))
    if desktop.get("version") != version:
        raise ValueError(f"Desktop 版本不一致：{desktop.get('version')}")
    protocol = protocol_version(root)
    schema = sqlite_schema_version(root)
    contract_path = release_contract(root, version, protocol, schema)
    driver_path, drivers = driver_manifest(root, version)
    generated = [
        root / "crates/protocol/generated/protocol.ts",
        root / "crates/protocol/generated/request.schema.json",
        root / "crates/protocol/generated/response.schema.json",
    ]
    migrations = sorted((root / "crates/adapters/src").glob("task*.sql"))
    missing = [str(path) for path in [*generated, contract_path, driver_path, *migrations] if not path.is_file()]
    if missing:
        raise ValueError(f"发布源缺失：{', '.join(missing)}")
    artifacts = {
        "desktop": "target/release/yonder-desktop",
        "cli": "target/release/yonder",
        "mcp": ["target/release/yonder", "mcp"],
    }
    if sys.platform == "darwin":
        artifacts["macos_app"] = "target/release/Yonda.app"
    if include_artifacts:
        desktop_path = root / artifacts["desktop"]
        cli_path = root / artifacts["cli"]
        macos_app = artifacts.get("macos_app")
        artifact_paths = [desktop_path, cli_path]
        if sys.platform == "darwin":
            artifact_paths.append(root / macos_app)
        missing_artifacts = [path for path in artifact_paths if not path.exists()]
        if missing_artifacts:
            raise ValueError(f"发布产物缺失：{', '.join(str(path) for path in missing_artifacts)}")
        artifacts = {
            "desktop": {"path": artifacts["desktop"], "sha256": sha256(desktop_path)},
            "cli": {"path": artifacts["cli"], "sha256": sha256(cli_path)},
            "mcp": {"path": artifacts["cli"], "command": ["yonder", "mcp"], "sha256": sha256(cli_path)},
        }
        if sys.platform == "darwin":
            artifacts["macos_app"] = {
                "path": macos_app,
                "sha256": tree_sha256(root / macos_app),
                "hash_kind": "recursive-files-sha256",
            }
    return {
        "schema": 1,
        "version": version,
        "commit": git_commit(root),
        "packages": packages,
        "protocol": protocol,
        "sqlite_schema": schema,
        "drivers": drivers,
        "sources": {
            "ipc": [sha256(path) for path in generated],
            "migrations": [sha256(path) for path in migrations],
            "driver_manifest": sha256(driver_path),
            "release_contract": sha256(contract_path),
        },
        "artifacts": artifacts,
        "channels": ["dev", "stable"],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--output", type=Path, help="发布冻结清单输出路径；默认打印 JSON")
    parser.add_argument("--artifacts", action="store_true", help="校验并记录 release 产物哈希")
    args = parser.parse_args()
    manifest = collect_manifest(args.root, args.artifacts)
    rendered = json.dumps(manifest, ensure_ascii=False, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8")
    else:
        print(rendered, end="")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, KeyError, subprocess.CalledProcessError) as error:
        print(f"发布冻结失败：{error}", file=sys.stderr)
        sys.exit(1)
