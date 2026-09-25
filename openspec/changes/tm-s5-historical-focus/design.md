# 设计

- 新表 `task_focus_events` 以 `(task_id, sequence)` 保存控制身份、locating/focused/failed 和稳定失败分类；外键绑定既有控制与事件。
- `begin_focus/finish_focus` 在既有任务、事件、Outbox事务中追加历史及更新当前投影；任一写入失败全部回滚。schema 18→19 不回填旧当前值。
- Adapter 按事件序号只读关联；Application 仅在协议 1.23 输出可选 `focus_event`，旧协议缺省，继续执行授权、连续性和编码预算。
- UI 只显示已提交定位阶段和失败原因，不推断 Recording、用户操作或交回。
