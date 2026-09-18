# 设计

`task.fail` 参数与 `task.complete` 相同但使用独立 capability。Application 从 Store 读取当前任务、pending control 和最新 attempt；只有 running、当前序号、Desktop Permit、stopped 且持久化结论为 `Observed(false)` 时执行 `Action::Fail`。现有 Store commit 保证 tasks/events/outbox 同事务；提交后再释放 Admission。

协议 1.18 协商并发布能力，旧会话拒绝。CLI/MCP 增加 `task_fail`。desktop 终态判定接受成功 Fail 响应并以 `task_id/sequence/failed` 去重。没有失败正文或新表；具体依据从现有步骤与 attempt_result 查询。
