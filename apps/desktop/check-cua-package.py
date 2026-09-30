#!/usr/bin/env python3
"""校验正式预览包只携带Sky产品Worker与锁定Jev SDK。"""
import json
from pathlib import Path

root = Path(__file__).resolve().parents[2]
cua = root / "apps/desktop/target/preview/Yonda Task Space.app/Contents/Resources/cua"
jev_package = json.loads((cua / "node_modules/@typesafe-ai/sdk/package.json").read_text())
result = {
    "cua_driver": "sky-external-0.7.1",
    "jev_sdk_version": jev_package.get("version"),
    "node_present": (cua / "node").is_file(),
    "sky_worker_matches": (cua / "sky_cua_worker.mjs").read_bytes() == (root / "crates/adapters/src/sky_cua_worker.mjs").read_bytes(),
    "jev_worker_matches": (cua / "jev_worker.mjs").read_bytes() == (root / "crates/adapters/src/jev_worker.mjs").read_bytes(),
    "qwen_absent": not any((cua / "node_modules").rglob("*qwen*")),
}
result["passed"] = result["jev_sdk_version"] == "0.6.0" and all(result[key] for key in ("node_present", "sky_worker_matches", "jev_worker_matches", "trycua_absent", "qwen_absent"))
evidence = root / "apps/desktop/evidence/cua-formal-runtime-20260918/result.json"
evidence.parent.mkdir(parents=True, exist_ok=True)
evidence.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
print(json.dumps(result, ensure_ascii=False))
raise SystemExit(0 if result["passed"] else 1)
    "trycua_absent": not (cua / "node_modules/@trycua").exists() and not (cua / "cua_worker.mjs").exists(),
