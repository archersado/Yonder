# 独立 Verification Goal：Agent 步骤声明

日期：2026-09-15  
关联 Story：AG-S3  
结论：PASS（首批核心与 macOS 私有 stdio；完整 Story 不归档）

## 结果

- 协议 1.4 严格解析步骤声明，限制 ID、序号和 256 UTF-8 字节纯文本；Rust 重新生成 JSON Schema 与 TypeScript。
- Gateway 只有完成 1.4 握手且存储支持时开放声明/读取；归属与 LocalUser 写入边界由可信 `AuthContext` 保持。1.3 事件保留声明产生的序号，但不输出 `step_declaration`。
- SQLite 6 在 `IMMEDIATE` 事务内提交任务序号、不可变步骤、`created→created` 事件和 Outbox；同标签重试不新增记录，异标签冲突，取消后重试返回取消快照。合成 Outbox 故障时序号和步骤全部回滚。
- 正式桌面数据库从 schema 5 升级到 6 前生成一致备份。私有 stdio Agent 完成 1.4 握手、创建、声明、当前步骤读取、新旧事件投影和取消；数据库保留 1 条步骤，测试任务状态为 `cancelled`、序号 3。

## 证据

- `cargo run --offline --locked -p yonder-protocol --example generate`
- `cargo test --offline --locked -p yonder-protocol -p yonder-application -p yonder-adapters -p yonder-desktop --lib`
- `apps/desktop/evidence/agent-step-20260915/result.json`
- 正式数据库只读核对：`user_version=6`，步骤 `open-document@2`，任务 `cancelled@3`。
- 升级备份：`tasks.db.pre-step-v5-24717-1789458548533876000.db`。

## 边界

本项只登记 Agent 声明，不解释或执行标签，不产生 attempt、WorkRef、running、Observe、Recording、成功或失败事实。生产连接认证、真实动作链路与 Windows 按各自 Story 继续验证。
