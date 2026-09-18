# AG-S4 Yonder Agent Skill

Story: AG-S4
Epic: AG
Status: draft
OpenSpec: -

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

本 Story 只记录统一 Agent Skill 的产品与架构规格，当前不生成 Skill、不创建 OpenSpec、不修改运行时代码。

实施前必须完成并验证 Yonder 的 BUA、CUA、Document 与 Command 四类 Gateway 能力。BUA 继续复用 ego-lite Task Space；Skill 将 ego-browser 的操作规则作为 BUA 子模块，但所有任务登记、身份、状态、Observe、控制和完成均由 Yonder 管理。CUA、文档和命令不得由 Skill 建立旁路执行栈。

## OpenSpec 与验证

待上述四类能力的协议和真实运行证据齐备后，再审阅三份设计并创建单独 OpenSpec Change。Skill 包需独立验证能力发现、完整任务生命周期、用户接管、失败恢复和禁止旁路；Windows 按当前用户决定暂缓，但不得据此将完整 Story 标记完成。
