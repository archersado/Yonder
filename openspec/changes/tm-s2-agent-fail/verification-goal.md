# Verification Goal：Agent 提交已观察失败终态

状态：PASS（macOS 子范围，2026-09-18）。完整 TM-S2 不 Archive。

## 结论

- 协议1.18只向满足执行门禁的会话发布 `task.fail`，Rust派生 JSON Schema 与 TypeScript 已同步；Yonder CLI/MCP暴露 `task_fail`。
- SQLite集成样本以真实attempt顺序验证 `Observed(false) → stopped → failed`，状态、事件、Outbox复用既有事务，成功后释放Desktop资源；失败动作不能调用complete，成功动作、unknown与未停止结果不能调用fail。
- Gateway终态展示只接受请求与结果相符的响应；ego-browser Task Space 83验证success/failed分别进入、1.8秒恢复及相同事实不重播。
- Workspace 52项测试、架构与关联检查、JavaScript语法和diff检查通过。结构化证据见 [result.json](../../../apps/desktop/evidence/tm-s2-agent-fail-20260918/result.json)。

## 保留门禁

Windows原生证据按用户要求暂缓，因此完整Story不Done/Archive。失败原因正文与unknown自动归类均未加入；需要独立需求与审计契约后再设计。
