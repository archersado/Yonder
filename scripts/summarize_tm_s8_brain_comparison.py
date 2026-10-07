#!/usr/bin/env python3
"""汇总 TM-S8 快慢脑/纯慢脑脱敏配对试验；不连接或操作 Yonder。"""

from __future__ import annotations

import argparse
import json
import math
import statistics
import sys
from collections import Counter, defaultdict
from pathlib import Path

MODES = ("fast-slow", "slow-only")
OUTCOMES = ("achieved", "not-achieved", "unknown", "timeout")


def percentile_nearest_rank(values: list[int], percent: float) -> int | None:
    if not values:
        return None
    ordered = sorted(values)
    return ordered[max(0, math.ceil(percent * len(ordered)) - 1)]


def wilson(successes: int, total: int) -> list[float] | None:
    if total == 0:
        return None
    z = 1.959963984540054
    observed = successes / total
    denominator = 1 + z * z / total
    centre = (observed + z * z / (2 * total)) / denominator
    margin = z * math.sqrt(
        observed * (1 - observed) / total + z * z / (4 * total * total)
    ) / denominator
    return [round(max(0.0, centre - margin), 6), round(min(1.0, centre + margin), 6)]


def require_non_negative_int(trial: dict, key: str) -> int:
    value = trial.get(key)
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise ValueError(f"{key}须为非负整数")
    return value


def validate(document: dict) -> list[dict]:
    if document.get("schema_version") != 1 or not isinstance(document.get("trials"), list):
        raise ValueError("仅支持schema_version=1且trials须为数组")
    seen: set[tuple[str, str, str]] = set()
    trials = document["trials"]
    for trial in trials:
        if not isinstance(trial, dict):
            raise ValueError("trial须为对象")
        task_class = trial.get("task_class")
        pair_id = trial.get("pair_id")
        mode = trial.get("mode")
        outcome = trial.get("outcome")
        if not isinstance(task_class, str) or not task_class or not isinstance(pair_id, str) or not pair_id:
            raise ValueError("task_class和pair_id不能为空")
        if mode not in MODES or outcome not in OUTCOMES:
            raise ValueError("mode或outcome无效")
        key = (task_class, pair_id, mode)
        if key in seen:
            raise ValueError(f"重复trial: {key}")
        seen.add(key)
        wall = require_non_negative_int(trial, "wall_elapsed_ms")
        wait = require_non_negative_int(trial, "user_wait_ms")
        if wait > wall:
            raise ValueError("user_wait_ms不能大于wall_elapsed_ms")
        for field in ("slow_brain_calls", "handbacks", "actions", "unknowns"):
            require_non_negative_int(trial, field)
        if trial.get("product_path") is not True:
            raise ValueError("非正式产品链路样本不得进入汇总")
    return trials


def mode_summary(trials: list[dict], mode: str) -> dict:
    selected = [trial for trial in trials if trial["mode"] == mode]
    completed = [trial for trial in selected if trial["outcome"] == "achieved"]
    active = [trial["wall_elapsed_ms"] - trial["user_wait_ms"] for trial in completed]
    wall = [trial["wall_elapsed_ms"] for trial in completed]
    successes = len(completed)
    total = len(selected)
    return {
        "trials": total,
        "achieved": successes,
        "completion_rate": round(successes / total, 6) if total else None,
        "completion_rate_wilson_95": wilson(successes, total),
        "outcomes": dict(sorted(Counter(trial["outcome"] for trial in selected).items())),
        "active_elapsed_ms": {
            "median": round(statistics.median(active), 3) if active else None,
            "p95_nearest_rank": percentile_nearest_rank(active, 0.95),
        },
        "wall_elapsed_ms": {
            "median": round(statistics.median(wall), 3) if wall else None,
            "p95_nearest_rank": percentile_nearest_rank(wall, 0.95),
        },
        "totals": {
            field: sum(trial[field] for trial in selected)
            for field in ("slow_brain_calls", "handbacks", "actions", "unknowns", "user_wait_ms")
        },
    }


def summarize(document: dict) -> dict:
    trials = validate(document)
    pairs: dict[tuple[str, str], dict[str, dict]] = defaultdict(dict)
    for trial in trials:
        pairs[(trial["task_class"], trial["pair_id"])][trial["mode"]] = trial
    complete_pairs = [value for value in pairs.values() if all(mode in value for mode in MODES)]
    ratios = []
    for pair in complete_pairs:
        fast, slow = pair["fast-slow"], pair["slow-only"]
        if fast["outcome"] == slow["outcome"] == "achieved":
            fast_active = fast["wall_elapsed_ms"] - fast["user_wait_ms"]
            slow_active = slow["wall_elapsed_ms"] - slow["user_wait_ms"]
            if fast_active > 0:
                ratios.append(slow_active / fast_active)
    by_class = {}
    for task_class in sorted({trial["task_class"] for trial in trials}):
        selected = [trial for trial in trials if trial["task_class"] == task_class]
        by_class[task_class] = {mode: mode_summary(selected, mode) for mode in MODES}
    return {
        "schema_version": 1,
        "paired_trials": len(complete_pairs),
        "double_success_pairs": len(ratios),
        "modes": {mode: mode_summary(trials, mode) for mode in MODES},
        "paired_active_speedup_slow_over_fast": {
            "median": round(statistics.median(ratios), 6) if ratios else None,
            "ratios": [round(value, 6) for value in ratios],
        },
        "by_task_class": by_class,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("manifest", type=Path)
    args = parser.parse_args()
    try:
        result = summarize(json.loads(args.manifest.read_text(encoding="utf-8")))
    except (OSError, json.JSONDecodeError, ValueError) as error:
        print(str(error), file=sys.stderr)
        return 2
    print(json.dumps(result, ensure_ascii=False, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
