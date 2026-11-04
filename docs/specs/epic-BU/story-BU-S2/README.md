# BU-S2 Task Space 引用与任务事实映射

Story: BU-S2
Epic: BU
Status: verifying
OpenSpec: bu-s2-browser-reference-storage

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

BU-S1双平台Bridge与TM-S2连续步骤边界已验证；本Story的引用持久化、Gateway动作和只读/打开入口均已接入。

Gateway Increment: [bu-s2-agent-browser-gateway](../../../../openspec/changes/bu-s2-agent-browser-gateway/proposal.md)
Read Increment: [bu-s2-browser-reference-read](../../../../openspec/changes/bu-s2-browser-reference-read/proposal.md)

2026-09-17真实用户任务验证PASS：安装包内`yonder mcp`创建“查看今日微博热搜”，Yonder持有`ego:49`并记录三次Observed步骤，最终任务`completed@14`、引用finished且外部空间关闭；见[独立验证](../../../../openspec/changes/bu-s2-agent-browser-gateway/verification-user-task.md)。

2026-09-18引用读取与打开子范围PASS：详情显示Browser引用；用户点击“打开 ego-lite”后通过受监督`handOff()`进入对应空间，引用所有权与事件同步提交。见[读取与打开验证](../../../../openspec/changes/bu-s2-browser-reference-read/verification-goal.md)。

2026-10-25 Windows独立验证PASS：正式Named Pipe Gateway与真实ego-lite完成创建、引用读取、交接、恢复、结束与桌面重启后读取；Task Space引用UI结构化验证通过。Story技术验证完成，剩余PR与Archive门禁。

## OpenSpec 与验证

[Change](../../../../openspec/changes/bu-s2-browser-reference-storage/proposal.md)；独立Verification Goal位于`openspec/changes/bu-s2-browser-reference-storage/`，通过前不归档。
