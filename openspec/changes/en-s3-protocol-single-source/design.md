# 设计

实施已合入 dev（merge b4c3ba6，含 c01f316 重构与 30a4572 Story 文档）；本文件记录定稿设计与偏差记录，供独立 Verification Goal 对照。

## 版本常量

`PROTOCOL_VERSION: ProtocolVersion` 定义于 `crates/protocol/src/lib.rs` 顶部常量区，附注释声明"组合根与 CLI 须引用此常量，不得各写一份 minor 字面量"。`application/gateway.rs` 以 `pub use yonder_protocol::PROTOCOL_VERSION` 再导出，保持 `release_contract.rs` 的 `yonder_application::gateway::PROTOCOL_VERSION` 路径不变。`apps/yonder-cli/src/main.rs` 的 hello 握手改引常量，`ProtocolVersion` 字面量导入随之删除。

偏差记录：`codex_agent_bridge.rs` 保留 `minor: 19`（钉定能力面，非遗漏），major 改为从 `PROTOCOL_VERSION.major` 派生并注明原因。这是有意的部分例外，验收时按"除 bridge 外 apps/ 无 minor 字面量"断言。

## UnknownReason 唯一映射

新模块 `crates/application/src/unknown_reason.rs` 持有内部枚举 `UnknownReason`（8 值）与唯一 `impl From<UnknownReason> for yonder_protocol::AttemptUnknownReason`。`computer_use.rs` 的原枚举定义改为 `pub type UnknownReason = crate::unknown_reason::UnknownReason` 别名，adapters/desktop 的既有引用路径不变。gateway 的 `attempt_result` 与 query 的事件投影均改为 `reason.into()`，删除两份手写 8 分支 match 及对应未用 import。

新增回归 `internal_reasons_map_one_to_one_to_wire_reasons`：逐值断言内部枚举经 From 映射后 serde 序列化等于 kebab-case wire 名（invalid-input / dependency-unavailable / worker-failed / timed-out / invalid-response / identity-mismatch / observe-failed / user-input），锁定映射与 wire 形状不漂移。

## valid_id 单一实现

`protocol/src/lib.rs` 的 `valid_id` 为唯一字符集实现。`application/lib.rs` 的 `validate_id` 改为引用 protocol 判定、只负责映射 `Error::InvalidInput`；`pub use yonder_protocol::valid_id` 保持 `crate::valid_id`（application 内 31 处）与 `yonder_application::valid_id`（adapters/desktop）调用点零改动。

## 护栏

- 全程不触碰 SQLite schema/迁移、wire 枚举 serde 属性、Cargo.toml 依赖。
- 每步以 `cargo test --workspace --locked` 与协议 `--check` 为门禁。
- 若后续在 `apps/` 新增协议握手，必须引用常量；bridge 类有意钉定须注明理由。
