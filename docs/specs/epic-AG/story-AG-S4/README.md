# AG-S4 Yonder Agent Skill

Story: AG-S4
Epic: AG
Status: implementing
OpenSpec: ag-s4-yonder-skill

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

BUA、CUA、Document、Command 的 macOS Gateway 增量与独立 Verification Goal 已通过，2026-09-29 解除 macOS Skill 实施门禁。首版包位于 `skills/yonder/`，只包含 Agent 说明、按需参考、发现元数据与兼容 manifest，不修改 Yonder Runtime。

2026-09-23 设计增量：已明确 Skill 不实现第二套快慢脑或 Driver 循环；相关边界以 AD-EX-01/EX-S2 为准。

同日补充的 Skill 包结构候选已在本轮审阅后定案为仓库目录 `skills/yonder/`；版本 `0.1.0`，最低 Yonder 协议 `1.31`，运行时仍必须按实际工具发现失败关闭。

BUA 继续复用 ego-lite Task Space；Skill 将 ego-browser `2.0.0` 的必要操作规则作为 BUA 子模块，但所有任务登记、身份、状态、Observe、控制和完成均由 Yonder 管理。CUA、文档和命令不得由 Skill 建立旁路执行栈。Windows能力仍暂缓，不据此宣称完整 Story Done。

## OpenSpec 与验证

[Change](../../../../openspec/changes/ag-s4-yonder-skill/proposal.md)。Skill 包需独立验证能力发现、完整任务生命周期、用户接管、失败恢复和禁止旁路；Windows 按当前用户决定暂缓，但不得据此将完整 Story 标记完成。
