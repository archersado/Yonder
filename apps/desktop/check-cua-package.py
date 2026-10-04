#!/usr/bin/env python3
"""校验正式预览包只携带Sky产品Worker与锁定Jev SDK。"""
import json
import subprocess
from pathlib import Path

root = Path(__file__).resolve().parents[2]
cua = root / "target/debug/Yonda.app/Contents/Resources/cua"
jev_package = json.loads((cua / "node_modules/@typesafe-ai/sdk/package.json").read_text())
sky_package = Path("/Applications/ChatGPT.app/Contents/Resources/cua_node/lib/node_modules/@oai/sky")
sky_node = Path("/Applications/ChatGPT.app/Contents/Resources/cua_node/bin/node")
sky_bridge = sky_package / "Codex Computer Use.app/Contents/SharedSupport/SkyComputerUseClient.app/Contents/MacOS/SkyComputerUseClient"
sky_node_identity = subprocess.run(
    ["/usr/bin/codesign", "-dv", "--verbose=4", str(sky_node)],
    capture_output=True, text=True, check=False,
) if sky_node.is_file() else None
sky_identity = subprocess.run(
    ["/usr/bin/codesign", "-dv", "--verbose=4", str(sky_bridge)],
    capture_output=True, text=True, check=False,
) if sky_bridge.is_file() else None
sky_entitlements = subprocess.run(
    ["/usr/bin/codesign", "-d", "--entitlements", ":-", str(sky_bridge)],
    capture_output=True, text=True, check=False,
) if sky_bridge.is_file() else None
sky_package_identity = json.loads((sky_package / "package.json").read_text()) if (sky_package / "package.json").is_file() else {}
result = {
    "cua_driver": "sky-external-0.7.1",
    "jev_sdk_version": jev_package.get("version"),
    "node_present": (cua / "node").is_file(),
    "sky_worker_matches": (cua / "sky_cua_worker.mjs").read_bytes() == (root / "crates/adapters/src/sky_cua_worker.mjs").read_bytes(),
    "jev_worker_matches": (cua / "jev_worker.mjs").read_bytes() == (root / "crates/adapters/src/jev_worker.mjs").read_bytes(),
    "trycua_absent": not (cua / "node_modules/@trycua").exists() and not (cua / "cua_worker.mjs").exists(),
    "qwen_absent": not any((cua / "node_modules").rglob("*qwen*")),
    "sky_bridge_version": sky_package_identity.get("version"),
    "sky_node_team": "2DC432GLL2" if sky_node_identity and "TeamIdentifier=2DC432GLL2" in sky_node_identity.stderr else None,
    "sky_node_identifier_valid": bool(sky_node_identity and "Identifier=node" in sky_node_identity.stderr),
    "sky_bridge_team": "2DC432GLL2" if sky_identity and "TeamIdentifier=2DC432GLL2" in sky_identity.stderr else None,
    "sky_bridge_identifier_valid": bool(sky_identity and "Identifier=com.openai.sky.CUAService.cli" in sky_identity.stderr),
    "sky_bridge_app_group_valid": bool(sky_entitlements and "2DC432GLL2.com.openai.sky.CUAService" in (sky_entitlements.stdout + sky_entitlements.stderr)),
}
result["passed"] = result["jev_sdk_version"] == "0.6.0" and result["sky_bridge_version"] == "0.7.1" and all(result[key] for key in ("node_present", "sky_worker_matches", "jev_worker_matches", "trycua_absent", "qwen_absent", "sky_node_team", "sky_node_identifier_valid", "sky_bridge_team", "sky_bridge_identifier_valid", "sky_bridge_app_group_valid"))
evidence = root / "apps/desktop/evidence/cu-s4-sky-package-20260930/result.json"
evidence.parent.mkdir(parents=True, exist_ok=True)
evidence.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
print(json.dumps(result, ensure_ascii=False))
raise SystemExit(0 if result["passed"] else 1)
