# Proposal：agent-desktop 受监管 Driver Spike

状态：Proposed。期限：2026-11-05 至 2026-11-11。

关联 Story：[CU-S4](../../../docs/specs/epic-CU/story-CU-S4/README.md)；关联 AD：[Proposed AD-CU-07](../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-CU-07-AGENT-DESKTOP-DRIVER-SPIKE.md)。

## Why

独立研究确认 `agent-desktop` 具有跨平台 Rust 内核、C ABI、qualified ref、交付分类和跨进程 interaction lease，值得作为低层 CUA Driver 候选；但它不提供 Yonder 所需的用户输入立即暂停、在途协作式取消、任务级 ownership 或强制逐步 Observe。必须先用统一双平台样本证明 Yonder 监管包装层能补齐这些硬门槛，不能直接接入产品。

## What Changes

- 建立完全隔离的 Windows/macOS Spike harness 与受监管最小 Rust bridge Worker；固定同一源码 revision 和 inherited stdio JSONL 入口。
- 验证 snapshot/ref 动作、目标重识别、真实用户输入抢占、分级取消、Worker 终止清理、动作后 Observe、`unknown` 和分发责任链。
- 禁用 raw 坐标、无目标按键、`--force`、CDP、上游 MCP 与 hit-test fail-open；显式关闭默认 trace，保持 screenshots 禁用。
- 输出版本化结构化证据和独立 Verification Goal，并提出“双平台共享”或“整体 Rejected”的 ADR 结论建议。

本 Change 不修改产品 Gateway、协议、SQLite、Adapter、Task Space 或 UI，不替换 trycua，不处理真实用户任务。

## Architecture Impact

`architecture-change`：只验证候选 CUA 引擎、Worker 监督与平台分发边界。运行时架构在后继 Accepted ADR 和独立 Apply Change 前保持不变。
