#!/usr/bin/env python3
"""原生隔离验证；证据只保存分类与布尔，不保存输入或窗口树。"""
import json
import pathlib
import subprocess
import sys
import time

root = pathlib.Path(__file__).resolve().parents[2]
node = pathlib.Path(sys.argv[1]).resolve()
fixture_bin = pathlib.Path("/private/tmp/yonda-cu-dispatch-fixture")
fixture_src = root / "apps/desktop/tests/input-fixture-macos.swift"
worker = root / "crates/adapters/src/cua_worker.mjs"
sdk = root / "apps/desktop/cua/node_modules/@trycua/cua-driver/dist/index.js"
example = root / "target/debug/examples/cua_dispatch_check"
evidence = root / "apps/desktop/evidence/cua-dispatch-20260915/result.json"

subprocess.run(["swiftc", str(fixture_src), "-o", str(fixture_bin)], check=True)
fixture = subprocess.Popen([str(fixture_bin)], stdout=subprocess.PIPE, text=True)
result = {"platform": "macos", "sdk_version": "0.25.0", "upstream_app_started": False, "recording_started": False, "screenshot_requested": False}
try:
    deadline = time.time() + 15
    state = None
    while time.time() < deadline:
        state = json.loads(fixture.stdout.readline())
        if state.get("ready"): break
    if not state or not state.get("ready"): raise RuntimeError("fixture_not_ready")
    run = subprocess.run([str(example), str(node), str(worker), str(sdk), str(state["pid"]), str(state["window_id"])], capture_output=True, text=True, timeout=40)
    product = json.loads(run.stdout.strip()) if run.stdout.strip() else {}
    deadline = time.time() + 10
    native = False
    while time.time() < deadline and not native:
        native = json.loads(fixture.stdout.readline()).get("matches", False)
    result.update({"worker_exited": run.returncode == 0, "product_known_success": product.get("known_success") is True, "result_persisted": product.get("result_persisted") is True, "occupancy_retained": product.get("occupancy_retained") is True, "native_target_matches": native})
    result["passed"] = all(result[key] for key in ("worker_exited", "product_known_success", "result_persisted", "occupancy_retained", "native_target_matches"))
finally:
    fixture.terminate()
    try: fixture.wait(timeout=3)
    except subprocess.TimeoutExpired: fixture.kill(); fixture.wait()
    evidence.parent.mkdir(parents=True, exist_ok=True)
    evidence.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
print(json.dumps(result, ensure_ascii=False))
sys.exit(0 if result.get("passed") else 1)
