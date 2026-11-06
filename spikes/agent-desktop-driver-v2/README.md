# agent-desktop v2 隔离 Spike

`prepare_upstream.py` 在独立 `target/patched-agent-desktop` 目录准备固定上游源码，并核对补丁文件集合、源码树与 v2 样本哈希。当前 patch set 只按固定哈希引用 v1 hit-test 修复，后续 v2 补丁放在本目录的 `patches/`；`source-manifest.json` 明确标记 `baseline_source_prepared_no_v2_patch`，还不是 v2 候选补丁或可运行构建。`source_gate.py` 只读核对该独立源码，并报告已知动作重解析、自动等待及合成输入标记缺口。`inspection_scope=known_seams_only` 表示这不是完整调用图证明；输出中的两个门禁固定为未通过，不能用来准入原生动作。

运行：`python3 spikes/agent-desktop-driver-v2/prepare_upstream.py`，再运行 `python3 spikes/agent-desktop-driver-v2/source_gate.py`。`--check` 在门禁未通过时返回 2，可作为 CI 拒绝信号。

后续 v2 补丁必须放在本目录的新 patch set 和独立准备源码目录中，固定同一上游 revision，并重建 v2 manifest/样本；不得修改 CU-S4 的来源文件。

`input-visibility/` 是 Windows 自建窗口诊断：仅当前台确认成功时，由本进程与子进程各发送最多十次无文本 `F24` key-up，记录 Raw Input 计数和相同标记。它不接 Worker、任务或真实应用，`admission_ready` 固定为 false。结果见 [原生证据](../../docs/specs/epic-CU/story-CU-S5/evidence/windows-raw-input-forged-tag-2026-11-06.md)。
