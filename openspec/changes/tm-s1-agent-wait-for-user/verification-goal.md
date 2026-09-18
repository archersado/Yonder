# Verification Goal：Agent 等待用户

状态：PASS（macOS 子范围，2026-09-18）。完整 TM-S1 不 Archive。

## 结论

- 真实 SQLite schema 14 合约测试覆盖：活动 attempt 拒绝且保留占用；Observe 并推进后，协议 1.17 原子提交 `waiting-for-user`、等待原因事件及 Outbox，随后释放 Admission。
- 协议 1.16 调用返回版本错误，读取同一事件不含 `wait_reason`；1.17 可读取原因。越权、CAS、空值、控制字符和 512 字节上限沿协议/Application/Adapter 三层拒绝。
- Yonder MCP 增加 `task_wait_for_user`，不提供 Resume。ego-browser Task Space 74 回归通过，时间线显示“等待用户：请确认发送内容”，既有分页、局部失败和竞态保护均通过并已完成空间清理。
- Workspace 51 项测试与架构关联检查通过。正式 macOS 预览包已重建并重启为 PID 62198；现有 110 条任务迁移到 schema 14，迁移前 schema 13 备份保留。

证据：[结构化结果](../../../apps/desktop/evidence/agent-wait-for-user-20260918/result.json)。

## 保留门禁

Resume、用户回答投递和新鲜 Observe 仍属于 TM-S4/AG-S5；“等待外部响应”没有新增任务状态。Windows 原生证据按用户要求暂缓，因此完整 Story 不 Done/Archive。
