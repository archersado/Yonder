# 设计

## Application 边界

- 新增可信内部 `publish_artifact_manifest` 用例，不接受 `AuthContext::Agent`，不通过 Gateway 暴露。
- 输入是完整集合快照，最多 4096 个唯一 `reference_id`；Application 分配从 1 开始的稳定顺序键。
- 新增 `artifact_manifest_page` 只读用例，先验证当前任务读取权限，再按固定版本和排他顺序键读取 1..100 项。

## SQLite 与事务

- 复用 schema 18 的 `task_artifact_manifests` 与 `task_artifact_manifest_items`，不迁移表形状。
- 发布在单个 `IMMEDIATE` 事务中校验 `expected_sequence`、分配下一版本、写清单和全部条目、递增任务序号、追加事件与 Outbox。
- 任一条目、事件或 Outbox 写入失败时整体回滚；配额不足在写入前拒绝。
- 用户确认优先绑定已存在的最新清单；只有没有任何清单时才原子生成空版本 1。

## 失败与兼容

- 重复引用、非法 ID、超过 4096 项、零版本、非法分页或 created 任务发布均稳定拒绝。
- 无权任务与不存在任务统一不可见；不存在的清单版本不返回其他版本。
- 协议 1.20 及旧客户端形状不变，快照/事件仍只包含版本和条目数。
