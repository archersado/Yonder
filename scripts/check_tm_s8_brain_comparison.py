#!/usr/bin/env python3
"""TM-S8 对照汇总器的离线合约检查。"""

import importlib.util
import pathlib

SCRIPT = pathlib.Path(__file__).with_name("summarize_tm_s8_brain_comparison.py")
SPEC = importlib.util.spec_from_file_location("brain_comparison", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(MODULE)


def trial(pair, mode, outcome, wall, wait=0, task_class="local-edit", **counts):
    return {
        "task_class": task_class,
        "pair_id": pair,
        "mode": mode,
        "outcome": outcome,
        "wall_elapsed_ms": wall,
        "user_wait_ms": wait,
        "slow_brain_calls": counts.get("slow_brain_calls", 1),
        "handbacks": counts.get("handbacks", 0),
        "actions": counts.get("actions", 2),
        "unknowns": counts.get("unknowns", 0),
        "product_path": True,
    }


manifest = {
    "schema_version": 1,
    "trials": [
        trial("p1", "fast-slow", "achieved", 1000, slow_brain_calls=1),
        trial("p1", "slow-only", "achieved", 2000, slow_brain_calls=3),
        trial("p2", "fast-slow", "timeout", 5000),
        trial("p2", "slow-only", "achieved", 2400),
        trial("p3", "fast-slow", "achieved", 1500, wait=500, task_class="recovery"),
        trial("p3", "slow-only", "unknown", 1600, unknowns=1, task_class="recovery"),
    ],
}
summary = MODULE.summarize(manifest)
assert summary["paired_trials"] == 3
assert summary["double_success_pairs"] == 1
assert summary["modes"]["fast-slow"]["completion_rate"] == 0.666667
assert summary["modes"]["slow-only"]["completion_rate"] == 0.666667
assert summary["modes"]["fast-slow"]["active_elapsed_ms"]["median"] == 1000
assert summary["paired_active_speedup_slow_over_fast"]["median"] == 2
assert summary["modes"]["slow-only"]["totals"]["unknowns"] == 1

invalid = {"schema_version": 1, "trials": [trial("p1", "fast-slow", "achieved", 1)]}
invalid["trials"][0]["product_path"] = False
try:
    MODULE.summarize(invalid)
except ValueError:
    pass
else:
    raise AssertionError("非产品链路样本应被拒绝")

print("TM-S8 brain comparison summary contract: PASS")
