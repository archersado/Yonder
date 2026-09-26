# ST-S3 MVP 未加密任务存储

Story: ST-S3
Epic: ST
Status: verifying
OpenSpec: st-s3-mvp-task-storage

按Accepted AD-ST-01实施独立存储子范围。三份设计已完成工程审阅，不涉及UI、认证、桌面执行器或密钥；复用TaskStore与现有事务。2026-09-26 已在基于最新 `dev` 的独立 worktree 完成隔离审阅；原实现提交 `00d917b` 已为 `dev` 的祖先，本次审阅记录随独立提交合入。

Change：openspec/changes/st-s3-mvp-task-storage/；实现后创建独立Verification Goal，不以本Story通过替代DS-S2。

显式未加密入口已实施。[独立Verification Goal](../../../../openspec/changes/st-s3-mvp-task-storage/verification-goal.md)记录历史本机核心验证；本轮隔离审阅新增 1 项目标存储合约和 63 项 `yonder-adapters` 回归、架构检查及严格 OpenSpec 校验，证据见 [审阅记录](../../../../openspec/changes/st-s3-mvp-task-storage/verification-review-20260926.md)。Windows 验证按当前全局安排延期；本 Change 明确暂不 Archive，不据此宣称桌面任务总览已接通。
