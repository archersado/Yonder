# CU-S4 agent-desktop Driver 可行性 Spike

Story: CU-S4
Epic: CU
Status: ready
OpenSpec: cu-s4-agent-desktop-driver-spike
Architecture Decision: Proposed AD-CU-07

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

独立研究结论只支持把 `agent-desktop` 作为低层 Driver 候选进入限时 Spike，不支持直接产品接线。当前 Accepted trycua 基线不变；本 Story 只产生双平台技术证据和 ADR 结论，不修改 Gateway、协议、SQLite、产品 Adapter 或用户界面。

期限为 2026-11-05 至 2026-11-11。上游已冻结为 `d3dc190afbf34cd2377252398ddd5c30efbd093a`（v1.1.0），唯一入口为最小 Rust bridge + inherited stdio JSONL。双平台必须使用同一 bridge/evidence schema、统一样本和淘汰门槛；首次原生运行前还须冻结 harness commit、dirty diff、Cargo.lock、toolchain、features、targets、CRT/链接策略与制品哈希。任一硬淘汰条件命中即停止扩围并保留失败证据。

## OpenSpec 与验证

[OpenSpec Change](../../../../openspec/changes/cu-s4-agent-desktop-driver-spike/) 已建立。Spike 实施后必须建立独立 Verification Goal；验证通过只代表候选技术结论，不代表 CU-S2 或产品链路完成。

执行入口：

- [AD-CU-07](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-CU-07-AGENT-DESKTOP-DRIVER-SPIKE.md)
- [固定样本目录](../../../../openspec/changes/cu-s4-agent-desktop-driver-spike/sample-catalog.md)
- [Bridge 契约](../../../../openspec/changes/cu-s4-agent-desktop-driver-spike/bridge-contract-v1.md)
- [证据 Schema](../../../../openspec/changes/cu-s4-agent-desktop-driver-spike/evidence-schema-v1.json)
- [Windows 补丁构建证据](evidence/windows-patched-build-2026-11-05.md)
- [Windows 原生只读探针证据](evidence/windows-native-read-probe-2026-11-05.md)
- [Windows 窗口 property 代际探针证据](evidence/windows-window-property-oracle-2026-11-05.md)
- [Windows WinEvent 生命周期探针证据](evidence/windows-winevent-lifecycle-2026-11-05.md)
- [Windows Rust 生命周期观察器验证](evidence/windows-rust-lifecycle-observer-2026-11-05.md)
- [Windows 双进程生命周期观察器实验](evidence/windows-dual-lifecycle-observer-2026-11-05.md)
