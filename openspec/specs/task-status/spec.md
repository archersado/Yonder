# task-status Specification

## Purpose

定义任务列表的可信排序与分页语义，使 Agent 新建任务能够从 Gateway/SQLite 权威快照即时出现在任务空间第一页，同时保持旧协议兼容和跨页稳定性。

## Requirements

### Requirement: 任务空间最新任务优先

协议 1.32 的 `task.list` 请求 MUST 支持携带 `newest_first=true`。存储 MUST 在授权范围与状态筛选之后，按创建时间倒序、任务 ID 倒序执行稳定分页；任务空间的“进行中”和“全部”均使用该模式。

#### Scenario: 新任务出现在第一页顶部

- **GIVEN** 已有任务超过一页
- **WHEN** Agent 成功创建一条新任务且任务空间刷新
- **THEN** 第一页首项是该新任务的真实快照
- **AND** 前端未伪造、反转或临时置顶卡片

#### Scenario: 相同时间跨页稳定

- **GIVEN** 多个任务具有相同创建时间
- **WHEN** 调用方使用 `next_after_task_id` 连续翻页
- **THEN** 任务 ID 倒序打破并列
- **AND** 跨页无重复、无遗漏

#### Scenario: 历史未知时间

- **GIVEN** schema 21 迁移前任务没有可信创建时间
- **WHEN** 请求最新任务排序
- **THEN** 历史任务位于有可信时间任务之后
- **AND** 系统不推测或暴露伪造时间

#### Scenario: 旧会话兼容

- **WHEN** 协议 1.31 或更早会话请求 `newest_first=true`
- **THEN** Gateway 以版本错误拒绝
- **AND** 省略字段的请求保持任务 ID 升序语义
