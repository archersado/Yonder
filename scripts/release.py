"""生成 Yonder 固定版本发布冻结清单；只使用 Python 标准库。"""

import argparse
import hashlib
import json
from pathlib import Path
import plistlib
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
    source = read_text(root / "crates/protocol/src/lib.rs")
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


def require_clean_tracked_tree(root):
    changed = subprocess.check_output(
        ["git", "status", "--porcelain", "--untracked-files=no"], cwd=root, text=True
    ).strip()
    if changed:
        raise ValueError("发布冻结要求源码树无已跟踪改动")


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


def executable_build_info(path, argument):
    try:
        result = subprocess.run(
            [str(path), argument], check=True, capture_output=True, text=True, timeout=10
        )
        info = json.loads(result.stdout)
    except (OSError, subprocess.CalledProcessError, subprocess.TimeoutExpired, json.JSONDecodeError) as error:
        raise ValueError(f"macOS发布{path.name}构建身份不可用") from error
    if not isinstance(info, dict):
        raise ValueError(f"macOS发布{path.name}构建身份不可用")
    return info


MACOS_BUNDLE_EXACT_FILES = {
    "Contents/Info.plist",
    "Contents/MacOS/yonder-desktop",
    "Contents/MacOS/yonder",
    "Contents/Resources/build-provenance.json",
    "Contents/Resources/channel.json",
    "Contents/Resources/driver-manifest.json",
    "Contents/Resources/release-contract.json",
    "Contents/Resources/cua/node",
    "Contents/Resources/cua/sky_cua_worker.mjs",
    "Contents/Resources/cua/jev_worker.mjs",
}
MACOS_BUNDLE_PREFIXES = (
    "Contents/Resources/cua/node_modules/",
    "Contents/_CodeSignature/",
)
FORBIDDEN_BUNDLE_NAMES = {
    ".env",
    ".env.local",
    ".netrc",
    ".npmrc",
    "credentials",
    "credentials.json",
    "id_ed25519",
    "id_rsa",
    "tasks.db",
    "tasks.db-shm",
    "tasks.db-wal",
}
FORBIDDEN_BUNDLE_SUFFIXES = (
    ".db",
    ".db-shm",
    ".db-wal",
    ".cer",
    ".crt",
    ".jsonl",
    ".key",
    ".log",
    ".p12",
    ".pem",
    ".pfx",
    ".sqlite",
    ".sqlite3",
)
OWNED_TEXT_FILES = {
    "Contents/Info.plist",
    "Contents/Resources/build-provenance.json",
    "Contents/Resources/channel.json",
    "Contents/Resources/driver-manifest.json",
    "Contents/Resources/release-contract.json",
    "Contents/Resources/cua/sky_cua_worker.mjs",
    "Contents/Resources/cua/jev_worker.mjs",
}
SENSITIVE_TEXT_PATTERNS = (
    re.compile(r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----"),
    re.compile(r"\bAKIA[0-9A-Z]{16}\b"),
    re.compile(r"\bsk-[A-Za-z0-9_-]{20,}\b"),
)


def audit_macos_bundle(
    bundle,
    expected_channel=None,
    expected_commit=None,
    expected_version=None,
    verify_signature=False,
):
    if not bundle.is_dir():
        raise ValueError(f"macOS发布包不存在：{bundle}")
    files = []
    for path in sorted(bundle.rglob("*")):
        relative = path.relative_to(bundle).as_posix()
        if path.is_symlink():
            raise ValueError(f"macOS发布包禁止符号链接：{relative}")
        if not path.is_file():
            continue
        if relative not in MACOS_BUNDLE_EXACT_FILES and not relative.startswith(MACOS_BUNDLE_PREFIXES):
            raise ValueError(f"macOS发布包含未允许路径：{relative}")
        lower_name = path.name.lower()
        if lower_name in FORBIDDEN_BUNDLE_NAMES or lower_name.startswith(".env.") or lower_name.endswith(FORBIDDEN_BUNDLE_SUFFIXES):
            raise ValueError(f"macOS发布包含敏感文件：{relative}")
        files.append(relative)

    missing = sorted(MACOS_BUNDLE_EXACT_FILES - set(files))
    if missing:
        raise ValueError(f"macOS发布缺少固定内容：{', '.join(missing)}")

    channel_path = bundle / "Contents/Resources/channel.json"
    channel_payload = json.loads(read_text(channel_path))
    if set(channel_payload) != {"channel"} or channel_payload["channel"] not in {"dev", "stable"}:
        raise ValueError("macOS发布通道元数据无效")
    if expected_channel and channel_payload["channel"] != expected_channel:
        raise ValueError(f"macOS发布通道不一致：期望{expected_channel}，实际{channel_payload['channel']}")

    provenance_path = bundle / "Contents/Resources/build-provenance.json"
    provenance = json.loads(read_text(provenance_path))
    if set(provenance) != {"schema", "version", "profile", "commit", "artifacts"} or provenance.get("schema") != 1:
        raise ValueError("macOS发布构建来源格式无效")
    if provenance.get("profile") != "release":
        raise ValueError("macOS发布构建来源不是release")
    build_commit = provenance.get("commit")
    if not isinstance(build_commit, str) or not re.fullmatch(r"[0-9a-fA-F]{40}", build_commit):
        raise ValueError("macOS发布构建提交无效")
    if expected_commit and build_commit != expected_commit:
        raise ValueError("macOS发布构建提交与冻结提交不一致")
    if expected_version and provenance.get("version") != expected_version:
        raise ValueError("macOS发布构建版本与冻结版本不一致")
    artifact_specs = {
        "desktop": ("yonder-desktop", bundle / "Contents/MacOS/yonder-desktop", "--release-build-info"),
        "cli": ("yonder-cli", bundle / "Contents/MacOS/yonder", "build-info"),
    }
    artifacts = provenance.get("artifacts")
    if not isinstance(artifacts, dict) or set(artifacts) != set(artifact_specs):
        raise ValueError("macOS发布构建来源缺少二进制")

    if verify_signature:
        try:
            subprocess.run(
                ["/usr/bin/codesign", "--verify", "--deep", "--strict", str(bundle)],
                check=True,
                capture_output=True,
                text=True,
                timeout=30,
            )
        except (OSError, subprocess.CalledProcessError, subprocess.TimeoutExpired) as error:
            raise ValueError("macOS发布签名校验失败") from error

    binary_hashes = {}
    for name, (package, path, argument) in artifact_specs.items():
        record = artifacts.get(name)
        if not isinstance(record, dict) or set(record) != {"package"} or record.get("package") != package:
            raise ValueError(f"macOS发布{name}构建来源无效")
        info = executable_build_info(path, argument)
        if set(info) != {"schema", "package", "version", "profile", "commit"} or info.get("schema") != 1:
            raise ValueError(f"macOS发布{name}构建身份格式无效")
        if info.get("package") != package or info.get("version") != provenance.get("version"):
            raise ValueError(f"macOS发布{name}构建版本不一致")
        if info.get("profile") != "release" or info.get("commit") != build_commit:
            raise ValueError(f"macOS发布{name}构建身份不一致")
        binary_hashes[name] = sha256(path)

    with (bundle / "Contents/Info.plist").open("rb") as stream:
        info = plistlib.load(stream)
    if "LSEnvironment" in info:
        raise ValueError("macOS发布包禁止嵌入环境变量")

    for relative in sorted(OWNED_TEXT_FILES):
        path = bundle / relative
        if relative == "Contents/Info.plist":
            content = json.dumps(info, ensure_ascii=False)
        else:
            content = read_text(path)
        if any(pattern.search(content) for pattern in SENSITIVE_TEXT_PATTERNS):
            raise ValueError(f"macOS发布自有文本包含敏感内容：{relative}")

    return {
        "passed": True,
        "file_count": len(files),
        "channel": channel_payload["channel"],
        "build_commit": build_commit,
        "binary_sha256": binary_hashes,
        "signature_verified": verify_signature,
        "allowlist_version": 1,
        "sensitive_values_recorded": False,
    }


def collect_manifest(root, include_artifacts=False, expected_channel=None):
    if expected_channel and not include_artifacts:
        raise ValueError("校验发布通道必须同时启用产物审计")
    version, packages = workspace_versions(root)
    desktop = json.loads(read_text(root / "apps/desktop/tauri.conf.json"))
    if desktop.get("version") != version:
        raise ValueError(f"Desktop 版本不一致：{desktop.get('version')}")
    protocol = protocol_version(root)
    schema = sqlite_schema_version(root)
    commit = git_commit(root)
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
        require_clean_tracked_tree(root)
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
            bundle_audit = audit_macos_bundle(
                root / macos_app, expected_channel, commit, version, verify_signature=True
            )
            artifacts["macos_app"] = {
                "path": macos_app,
                "sha256": tree_sha256(root / macos_app),
                "hash_kind": "recursive-files-sha256",
                "content_audit": bundle_audit,
            }
    return {
        "schema": 1,
        "version": version,
        "commit": commit,
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
    parser.add_argument("--channel", choices=["dev", "stable"], help="校验产物通道与发布请求一致；须与--artifacts同用")
    args = parser.parse_args()
    manifest = collect_manifest(args.root, args.artifacts, args.channel)
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
