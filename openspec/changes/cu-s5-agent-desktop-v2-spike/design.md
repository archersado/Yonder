# Design：v2 交付与输入门禁

## 源码隔离

固定上游 `d3dc190afbf34cd2377252398ddd5c30efbd093a`。v2 使用独立 `spikes/agent-desktop-driver-v2`，patch set 包含 v1 hit-test 修复和 v2 候选补丁；prepare 程序只接受声明的改动文件并生成独立 source tree hash。不得修改 v1 `prepare_upstream.py`、patch 或 build manifest。[样本定义](sample-definitions-v2.json)升级为 `agent-desktop-spike-samples/v2`；Windows/macOS 使用同一逻辑 action、predicate、oracle 和阈值，平台 fixture 仅实现等价原生机制。

## 阶段门禁

阶段 0：只读源码/fixture 探针。静态清单列出 ref 二次解析、自动轮询、focus fallback、物理注入与所有动作终端；未覆盖路径不得派发。阶段 1：通过独立机制验证，证明原生对象从 snapshot 到最终动作仍为同一对象，并证明输入 observer 存活及事件归属。阶段 2：修订 AD-CU-08 并开展隔离 fixture 真实动作及 CU-S4 全样本。阶段 0/1 无原生动作；不能跳级。

Windows 低级钩子可能被静默移除，因此不能单独作为准入；Raw Input 需另证其他程序注入覆盖。`dwExtraInfo` 与 `eventSourceUserData` 是可碰撞的元数据，不能仅凭值等于当前 token 宣称当前 Worker 归属。observer 丢失、事件未知、权限不足或桌面切换一律拒绝。无法证明最终交付绑定的物理输入动作也拒绝；固定样本因此无法满足则淘汰。

## 结果

保持 CU-S4 `unsupported/not_delivered` 和 `unknown` 语义。证据仅记枚举、布尔、计数、耗时、哈希、样本版本。Windows/macOS 原生验证、固定应用集、取消、Observe、清理与分发继续沿用 CU-S4 硬门槛；独立 Verification Goal 复核后才归档。
