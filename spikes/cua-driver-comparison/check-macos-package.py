"""固定版本供应链Spike；只校验/解包，不运行下载构件。"""
import hashlib
import json
from pathlib import Path, PurePosixPath
import subprocess
import sys
import tarfile

EXPECTED_SHA256 = {
    "cua-driver-rs-0.25.0-darwin-universal-binary.tar.gz": "29984f5363c12d9901588e814a3a519b8015a1255d7a59d658fbf2d3e51f8983",
    "cua-driver-rs-0.25.0-darwin-universal.tar.gz": "02e3dc00f57e967d7e1d30fbc42b1329fdac1ac6052935e1356b48d21c862ee4",
}


def safe_member(member):
    path = PurePosixPath(member.name)
    return not path.is_absolute() and ".." not in path.parts and (member.isfile() or member.isdir())


def self_check():
    for name, allowed in [("bin/cua-driver", True), ("../escape", False), ("/escape", False)]:
        assert safe_member(tarfile.TarInfo(name)) == allowed
    link = tarfile.TarInfo("bin/link")
    link.type = tarfile.SYMTYPE
    assert not safe_member(link)


if __name__ == "__main__":
    self_check()
    if sys.argv[1:] == ["--self-check"]:
        print("供应链路径/链接拒绝检查通过")
        sys.exit(0)
    archive = Path(sys.argv[1]).resolve()
    with archive.open("rb") as stream:
        digest = hashlib.sha256()
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    if digest.hexdigest() != EXPECTED_SHA256.get(archive.name):
        raise ValueError("发布SHA-256不匹配；不解包、不执行")
    target = archive.parent / "unpacked"
    with tarfile.open(archive, "r:gz") as package:
        members = package.getmembers()
        if len(members) > 10000 or sum(member.size for member in members) > 512 * 1024 * 1024 or not all(safe_member(member) for member in members):
            raise ValueError("归档路径、类型或配额不安全；不解包、不执行")
        target.mkdir()
        package.extractall(target, members=members)
    apps = list(target.rglob("CuaDriver.app"))
    binaries = [apps[0] / "Contents/MacOS/cua-driver"] if len(apps) == 1 else [path for path in target.rglob("cua-driver") if path.is_file()]
    if len(binaries) != 1:
        raise ValueError("构件入口不唯一；不执行")
    binary = binaries[0]
    carrier = apps[0] if len(apps) == 1 else binary
    result = {"version": "0.25.0", "sha256": digest.hexdigest(), "archive_safe": True, "binary_path": str(binary), "binary_executed": False}
    for name, command in {
        "platform": ["/usr/bin/file", str(binary)],
        "signature": ["/usr/bin/codesign", "--verify", "--deep", "--strict", "--all-architectures", "--verbose=2", str(carrier)],
        "identity": ["/usr/bin/codesign", "-dv", "--verbose=4", str(carrier)],
        "gatekeeper": ["/usr/sbin/spctl", "--assess", "--type", "execute", "--verbose=4", str(carrier)],
    }.items():
        response = subprocess.run(command, capture_output=True, text=True, timeout=30)
        result[name] = {"exit_code": response.returncode, "output": (response.stdout + response.stderr)[:16384]}
    result["package_gate_passed"] = all(result[name]["exit_code"] == 0 for name in ("platform", "signature", "identity", "gatekeeper")) and "Mach-O" in result["platform"]["output"] and "arm64" in result["platform"]["output"] and "Developer ID Application:" in result["identity"]["output"]
    if apps:
        result["package_gate_passed"] = result["package_gate_passed"] and len(apps) == 1 and "Identifier=com.trycua.driver\n" in result["identity"]["output"]
    (archive.parent / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(result, ensure_ascii=False))
    sys.exit(0 if result["package_gate_passed"] else 1)
