# Jev 设置面板 macOS 验证

日期：2026-09-24

- `jev-settings-empty.png`：重新打开设置窗口后密码框为空，不回填或保留旧输入。
- `jev-settings-status.png`：`TaskHost` 未就绪时，Keychain 状态仍独立显示为“API Key：未配置”，非敏感配置错误单独反馈。
- `jev-settings-recovered-v17.png`：本机任务库从未授权的实验 schema v19 恢复为正式 v17 后，原有 Jev 远端配置可重新读取，API Key 密码框仍为空。
- 验证未输入、保存或记录任何真实 API Key。

## 本机任务库恢复审计

- 恢复前完整备份：`tasks.db.pre-recover-v19-20260924.db`，SHA-256 `da6d114089c32575b5a3759c67988fb41f978833db66e04563f63ee6cf6ea21b`。
- v19 额外的 `task_artifact_manifests`、`task_artifact_manifest_items`、`task_user_confirmations`、`task_plans` 均为 0 行；`task_audit_quota_state` 为 1 行实验配置。
- 经用户确认后，以单事务删除上述五张未过门禁的实验表并将 `user_version` 恢复为 17。
- 恢复后 `integrity_check=ok`、外键检查无结果；任务 630 条、事件 2382 条、Outbox 2382 条，与备份一致；数据库对象集合与全新 v17 库一致。
