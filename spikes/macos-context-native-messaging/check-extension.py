#!/usr/bin/env python3
"""静态验证扩展权限、隐私拒绝和Native Host精确来源。"""

import json
import pathlib


ROOT = pathlib.Path(__file__).resolve().parent
manifest = json.loads((ROOT / "extension/manifest.json").read_text())
worker = (ROOT / "extension/service-worker.js").read_text()
test = (ROOT / "extension/test.js").read_text()
host = json.loads((ROOT / "native-host-manifest.json").read_text())
permissions = set(manifest.get("permissions", []))
result = {
    "history_permission_absent": "history" not in permissions,
    "incognito_guard_present": "tab.incognito" in worker,
    "incognito_access_checked": "isAllowedIncognitoAccess" in test,
    "allowed_origin_exact": host.get("allowed_origins")
        == ["chrome-extension://mofdddjaniddgalgegfdjegiegpneokc/"],
    "host_path_absolute_at_install": host.get("path") == "__HOST_PATH__",
}
result["passed"] = all(result.values())
print(json.dumps(result, ensure_ascii=False, sort_keys=True))
if not result["passed"]:
    raise SystemExit(1)
