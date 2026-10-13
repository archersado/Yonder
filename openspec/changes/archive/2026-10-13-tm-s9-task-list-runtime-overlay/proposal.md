# Proposal：TM-S9 活动任务列表 Runtime 覆盖

关联 Story TM-S9 与 Accepted AD-TM-23。Architecture Impact：none；补齐既定“活动查询读 Runtime、历史查询读 SQLite”的列表投影缺口，不改变状态所有者、协议、持久化或技术栈。

## Why

正式 Yonder MCP 任务已在 Runtime `cancelled@13`，`task.get/events` 返回新鲜事实，但 `task.list` 仍显示 SQLite 的 `running@12`。任务面板因此可能把已停止任务继续展示为执行中。

## What Changes

- 对 `task.list` 页内活动任务应用 Runtime 快照覆盖。
- 在覆盖后执行 `include_finished/running_only` 状态过滤，并保持既有分页游标稳定。
- 覆盖列表、Gateway 与正式 MCP 回归；不阻塞执行热路径。

## Non-goals

不让 Runtime 扫描或重排全部历史任务，不新增数据库或第二事件源，不把 SQLite ACK 重新放回 Driver 热路径，不改变 Windows 暂缓结论。
