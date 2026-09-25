# AD-TM-20 任务创建来源历史投影

状态：Accepted（2026-09-24，TM-S5 创建来源历史子范围）。关联 TM-S1 / TM-S5、AG-S1、DS-S2。Architecture Impact：architecture-change（协议 1.24）；不改变持久化、创建授权或任务状态所有者。

## 背景

产品简报「MVP 主干链路」第 9 步与「Task Space 与权限模型」要求任务时间线可追溯发起 Agent、来源和生命周期。Accepted AD-TM-01 AC10 已规定 `source` 只能由可信组合根按接入方式写入；schema 15 起，创建事务已经把 `{source}` 与 `#1` 创建事件、任务当前值及 Outbox 原子保存到 `task_presentation_events(kind='source')`。当前 `task.events` 未读取该不可变事实，只能从最新详情看到来源，无法区分旧任务缺少历史事实与已提交创建来源。

## 决定

- Application 历史记录增加可空 `creation_event`，只包含不可变 `owner_agent_id` 与 `source`。`owner_agent_id` 读取任务不可变归属列；`source` 必须严格解析与同一事件序号关联的既有 `kind='source'` payload。
- 只有实际存在 source payload 的创建事件才投影；schema 15 以前迁移为 `legacy` 但没有 source 历史的任务不回填、不从当前快照补造。
- Rust 协议 1.24 为 `TaskEvent` 增加可选 `creation_event`。1.23 及以下字段缺省；Gateway 沿用当前授权、事件连续性和 8 KiB/256 KiB 编码预算。
- Task Space 显示“任务创建：来源 · Agent”，不显示描述、幂等键、凭据或完整 Agent Payload。来源事实不表示当前 Agent 仍有权限，也不替代每次查询的实时授权。
- 不新增表、写入者、端口或第二任务通道；损坏 payload、非法 owner 或部分字段必须明确存储失败，不能退回当前快照猜测。

## 验证

覆盖本地/云端来源、协议 1.23/1.24 隔离、非归属 Agent 拒绝、损坏 payload、旧任务零回填、分页与编码预算。macOS 以隔离数据库、私有 Unix Socket 和正式 Tauri 宿主验证；Windows 按用户决定暂缓，完整 TM-S5 不因此 Archive。
