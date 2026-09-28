# Proposal：EN-S3 阶段1 协议单一来源

关联 Story：EN-S3（docs/specs/epic-EN/story-EN-S3/）；关联 AD：无新增 ADR——本 Change 受"外部可观察行为不变"等价性约束，属 conforming 演进。Architecture Impact：none（wire 协议、持久化 schema、依赖方向、能力协商结果均不变；只改变实现位置与组织）。

## 问题

协议单一来源被侵蚀为三处平行事实：

1. `PROTOCOL_VERSION` 定义在 `crates/application/src/gateway.rs`，而 CLI（只依赖 protocol）在 `apps/yonder-cli/src/main.rs` 硬编码 `minor: 32`。协议升版时 CLI 静默落后、丢失能力。
2. `UnknownReason` 存在两套枚举（application 内部 / protocol wire）加两份手写 8 分支映射（`gateway.rs` 与 `query.rs`）。新增 reason 时漏改任一处即静默丢分支，无编译期保护。
3. `valid_id` 双实现（`application/lib.rs` 与 `protocol/lib.rs`），字符集判断重复维护。

## 方案

- 版本常量下沉至 `yonder-protocol`，application `pub use` 再导出，CLI 引用常量；Codex 桥钉定 `minor: 19` 属既有能力边界（只使用 agent.input 文本通道），保留并以 `major: PROTOCOL_VERSION.major` 派生。
- 新增 `crates/application/src/unknown_reason.rs`：内部枚举 + 唯一 `From<UnknownReason> -> yonder_protocol::AttemptUnknownReason` 映射 + 逐值序列化回归；gateway/query 两份手写 match 删除，改为 `reason.into()`。
- `valid_id` 唯一实现回归 protocol；application 只做 `Error::InvalidInput` 映射并 `pub use` 保持调用点零改动。

## 等价性证明（已随实施提交）

- wire 协议未动：`PROTOCOL_VERSION` 数值不变（1.32），全部消息/字段/枚举序列化形状不变。
- 持久化未动：SQLite schema、`user_version`、事件/Outbox 语义不变。
- 依赖方向未动：CLI→protocol 引用常量合法；application→protocol 既有方向。
- 验证：`cargo test --workspace --locked` 161 项通过；`cargo run -p yonder-protocol --example generate --locked -- --check` 通过。

## 非目标

不实施 EN-S3 阶段 2–5（能力协商表、TaskStore 拆分、宿主并发、前端门禁）；不改 Codex 桥的钉定 minor 值；不修 EX-S2 的既有门禁关联缺失。
