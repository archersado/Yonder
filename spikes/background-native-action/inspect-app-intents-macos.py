#!/usr/bin/env python3
"""只读检查 App Intents 公开 SDK 是否提供跨应用通用执行入口。"""

import json
from pathlib import Path
import re
import subprocess
import sys


output = Path(sys.argv[1] if len(sys.argv) > 1 else "evidence/app-intents-macos-20260918/result.json")
sdk = Path(subprocess.check_output(["/usr/bin/xcrun", "--sdk", "macosx", "--show-sdk-path"], text=True).strip())
sdk_version = subprocess.check_output(["/usr/bin/xcrun", "--sdk", "macosx", "--show-sdk-version"], text=True).strip()
interfaces = sorted((sdk / "System/Library/Frameworks/AppIntents.framework").glob("**/arm64e-apple-macos.swiftinterface"))
assert len(interfaces) == 1
interface = interfaces[0].read_text()

metadata = Path("/System/Applications/Calculator.app/Contents/Resources/Metadata.appintents/extract.actionsdata")
actions = json.loads(metadata.read_text()).get("actions", {})
shortcuts = subprocess.run(["/usr/bin/shortcuts", "help"], text=True, capture_output=True, check=True).stdout
generic_entrypoints = re.findall(r"public (?:static )?func (?:invoke|execute)\s*\(", interface)

result = {
    "platform": "macos",
    "sdk_version": sdk_version,
    "app_intent_protocol_present": "public protocol AppIntent" in interface,
    "concrete_perform_requirement_present": "func perform() async throws -> Self.PerformResult" in interface,
    "concrete_call_as_function_present": "public func callAsFunction" in interface,
    "donation_manager_present": "public struct IntentDonationManager" in interface,
    "metadata_action_count": len(actions),
    "generic_cross_app_entrypoint_count": len(generic_entrypoints),
    "shortcuts_command_route_present": "run                     Run a shortcut." in shortcuts,
}
result["generic_cross_app_invocation_supported"] = result["generic_cross_app_entrypoint_count"] > 0
result["passed"] = all([
    result["app_intent_protocol_present"],
    result["concrete_perform_requirement_present"],
    result["concrete_call_as_function_present"],
    result["donation_manager_present"],
    result["metadata_action_count"] > 0,
    not result["generic_cross_app_invocation_supported"],
    result["shortcuts_command_route_present"],
])

output.parent.mkdir(parents=True, exist_ok=True)
output.write_text(json.dumps(result, ensure_ascii=False, indent=2, sort_keys=True) + "\n")
print(json.dumps(result, ensure_ascii=False, sort_keys=True))
raise SystemExit(0 if result["passed"] else 1)
