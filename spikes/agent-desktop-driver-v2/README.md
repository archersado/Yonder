# agent-desktop v2 隔离 Spike

当前仅有只读 `source_gate.py`。它先核对 CU-S4 固定源码及补丁哈希，再报告已知动作重解析、自动等待及合成输入标记缺口。`inspection_scope=known_seams_only` 表示这不是完整调用图证明；输出中的两个门禁固定为未通过，不能用来准入原生动作。

运行：`python3 spikes/agent-desktop-driver-v2/source_gate.py`。`--check` 在门禁未通过时返回 2，可作为 CI 拒绝信号。

后续 v2 补丁必须放在本目录的新 patch set 和独立准备源码目录中，固定同一上游 revision，并重建 v2 manifest/样本；不得修改 CU-S4 的来源文件。
