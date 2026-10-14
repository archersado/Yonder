# 设计：TaskStore 聚合端口与 SQLite 分区

## 端口边界

- `TaskCoreStore`：登记、读取、分页、事件、提交、步骤与展示。
- `TaskExecutionStore`：attempt、Observe、Browser 引用、控制、定位与等待。
- `TaskAuditStore`：审计、产物清单与用户确认。
- `TaskPlanFragmentStore`：计划片段与目标核验。
- `TaskStore` 仅作为上述 trait 的组合 supertrait；Gateway/Application 现有泛型签名保持不变。

## 能力快照

`StoreCapabilities` 是只读值对象，由 Store 在握手开始时一次返回。Gateway 不再逐项散布调用 `supports_*`；能力值仍与重构前完全相同。

## SQLite 文件边界

`task_store.rs` 转为模块入口，按 `core.rs`、`execution.rs`、`audit.rs`、`plan_fragment.rs`、`schema.rs`、`mapping.rs` 分区。迁移只移动实现，不改 SQL 文本和测试断言。内部共用连接与私有辅助函数通过受限 `pub(super)` 暴露。

## 验证

阶段提交依次为能力快照、trait 拆分、SQLite 分区和独立 Verification Goal。任一阶段若改变协议、`user_version`、SQL/事务或既有回归结果，立即停止并返回实施修正。
