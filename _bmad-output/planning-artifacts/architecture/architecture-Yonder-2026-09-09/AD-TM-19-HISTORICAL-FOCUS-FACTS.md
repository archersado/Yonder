# AD-TM-19 接管定位阶段的不可变历史事实

状态：Accepted；日期：2026-09-24；Story：TM-S5。来源：产品简报「MVP 主干链路」第 9 步、「Task Space 与权限模型」，TM5-AC02/07/08；承接 AD-TM-07、AD-TM-08、AD-TM-18。

## 问题

现有 `task_controls.focus_phase/focus_failure/focus_sequence` 是当前投影。定位从 `locating` 更新到 `focused` 或 `failed` 时会覆盖前一阶段及序号，不能满足历史不可覆盖要求，也不能从最终值可靠反推中间事实。

## 决策

SQLite schema 19 新增 `task_focus_events`，仅保存 Yonder 已提交的有界定位事实：`task_id`、事件 `sequence`、`control_id`、`phase` 与稳定失败分类。`begin_focus`、`finish_focus` 在既有任务序号/事件/Outbox事务中同时追加该表；当前投影继续保留供详情读取。迁移不回填旧任务，避免从最后阶段伪造旧的 locating 或结果事实。

`task.events` 协议 1.23 增加可选 `focus_event`；Adapter 只按 `(task_id, sequence)` 关联不可变记录，Application 仅对已授权、协商 1.23 的会话输出。Task Space 分别显示“正在定位”“定位成功”或明确失败原因；这些事实不表示 Recording 已开始、用户已操作或任务已交回。

## 验证与边界

验证覆盖 locating→focused、locating→failed、事务回滚、旧库不补造、1.22兼容、越权、损坏数据、分页/字节预算和 macOS 正式宿主可见证据。Windows按用户决定暂缓。多显示器/跨Space能力证据仍归TM-S3/CU-S2；本决策只保存已经产生的定位事实。
