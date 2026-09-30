#!/usr/bin/env python3
import json
import pathlib

root = pathlib.Path(__file__).resolve().parent
evidence = root / "evidence"
schema = json.loads((evidence / "schema-0.30.4.json").read_text())
baseline = json.loads((evidence / "baseline-0.25.0.json").read_text())
candidate = json.loads((evidence / "candidate-0.30.4.json").read_text())
recovery = json.loads((evidence / "recovery-failure-0.25.0.json").read_text())

assert schema["passed"] is True
assert baseline["passed"] is True and baseline["version"] == "0.25.0"
assert candidate["passed"] is True and candidate["version"] == "0.30.4"
assert all(baseline[field] is True for field in [
    "wrong_target_rejected", "target_unchanged_after_rejection", "action_confirmed",
    "sdk_observe_matches", "fixture_matches", "decoy_restored",
    "after_shutdown_rejected", "cleanup",
])
assert recovery["passed"] is False
assert recovery["action_confirmed"] is False
assert recovery["fixture_matches"] is False
assert recovery["decoy_restored"] is False
assert recovery["cleanup"] is True
for item in [schema, baseline, candidate, recovery]:
    encoded = json.dumps(item, ensure_ascii=False)
    assert "YONDER_SDK_INPUT_A" not in encoded
    assert "window_id" not in encoded and "pid" not in encoded

print(json.dumps({
    "schema_frozen": True,
    "baseline_0_25_passed": True,
    "candidate_0_30_passed": True,
    "upgrade_required": False,
    "recovery_failure_blocks_pass": True,
    "bounded_evidence": True,
}))
