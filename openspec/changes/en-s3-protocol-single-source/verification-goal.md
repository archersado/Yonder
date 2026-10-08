# EN-S3 阶段 1 协议单一来源独立 Verification Goal

Story: EN-S3  
日期：2026-10-08  
Result: PASS

## 验证范围

本 Goal 只复核 EN-S3 阶段 1：协议版本常量、ID 规则、执行未知原因映射及 Codex bridge 的有意版本例外。阶段 2～5 不在本 Change 的通过结论内，EN-S3 Story 继续保持 `design-review`。

## 独立复核发现与修正

复核最初发现两处后续回归，因此先退回实施并修正：

1. TM-S9 新增 `execution_runtime.rs` 时复制了 `valid_id` 字符集实现；现已删除并引用 application 从 `yonder-protocol` 再导出的唯一函数。
2. Codex bridge 为接入 `replan-input` 升级后使用 `PROTOCOL_VERSION.minor`，会随产品升版声明未实现的新能力；现按实际能力明确钉定协议 1.33，major 继续引用单一来源。

架构门禁新增 `check_protocol_single_source`，以后会拒绝重复 `valid_id`、重复 `PROTOCOL_VERSION`、第二份 UnknownReason 映射，以及 bridge 漂浮 minor。门禁自测同时覆盖接受和四类拒绝样本。

## 等价性证据

- Wire/Schema：`cargo run --offline --locked -p yonder-protocol --example generate -- --check` 通过，生成的 JSON Schema 与 TypeScript 无漂移。
- 映射：`internal_reasons_map_one_to_one_to_wire_reasons` 逐值通过；代码扫描确认 `UnknownReason` 枚举及其 wire `From` 只位于 `unknown_reason.rs`。
- ID：代码扫描确认 `valid_id` 只定义于 `crates/protocol/src/lib.rs`，Runtime/Application 均引用该来源。
- 版本：`PROTOCOL_VERSION` 只定义于 protocol；主 CLI 跟随当前版本，bridge 明确钉定 1.33；协议测试证明 `replan-input` 的最低版本正是 1.33。
- 依赖方向：`python3 scripts/check_architecture.py` 通过，CLI→protocol、application→protocol 等既有方向不变。
- 持久化：本 Change 与本轮修正均未修改 SQL schema、迁移或 `user_version`；Adapters 全量测试通过。

## 执行结果

- `cargo test --workspace --offline --locked`：221 项通过，Doc tests 通过。
- `python3 scripts/test_check_architecture.py`：21 项通过。
- `python3 scripts/check_architecture.py`：通过。
- `openspec validate en-s3-protocol-single-source --strict`：通过。

## 结论与边界

EN-S3 阶段 1 满足协议单一来源及外部行为等价护栏，可以归档本 Change。此 PASS 不授权或证明能力协商表、Gateway 穷举分发、TaskStore 拆分、宿主并发模型或前端门禁等阶段 2～5；这些阶段仍须完成设计审阅后分别实施和验证。
