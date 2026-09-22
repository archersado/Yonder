# 设计

## 确认与清单

- 只允许可信本机用户对终态任务提交一次结果确认；确认绑定被确认的 `result_sequence`、当前 `manifest_version`、可选 1..2048 UTF-8 字节意见和稳定 `confirmation_id`。
- 重复 `confirmation_id` 返回既有回执，不追加事件；不同内容冲突拒绝。
- 产物清单以 `(task_id, manifest_version)` 唯一，首次为 1；清单发布后不可变。条目只保存顺序键、`reference_id` 和可用性结论，不把路径当作版本身份。
- 新增、替换或失效产物生成新清单版本，旧确认继续绑定旧版本。

## 存储与事务

- schema 16 新增 `task_user_confirmations`、`task_artifact_manifests`、`task_artifact_manifest_items`、`task_audit_quota_state`。
- 确认、清单、事件、任务序号和 Outbox 在同一个 `IMMEDIATE` 事务内提交；失败整体回滚。
- 旧任务没有确认/清单时显式为空，不回填、不伪造。

## 配额与迁移

- 任务审计库上限 2 GiB；写入前可用磁盘保留 1 GiB。容量不足时返回 `QuotaExceeded`，不自动清理历史、用户产物或未同步/固定内容。
- 协议升级到 1.20；1.19 及以下客户端继续使用旧投影，不看见 `user_confirmation` 与 `artifact_manifest`。
- 仅允许 schema 15→16 迁移；迁移前创建备份，未知版本拒绝启动，失败整体回滚。

## UI 与验证

- 终态任务显示“结果待确认/已确认”，确认后展示 `sequence`、产物清单版本和意见。
- 产物变化提示需重新检查，不自动把当前文件当作旧版本已验证。
- 验证覆盖真实迁移、幂等确认、清单不变、配额拒绝、旧客户端字段剥除、Outbox 回滚和 macOS 原生 UI。
