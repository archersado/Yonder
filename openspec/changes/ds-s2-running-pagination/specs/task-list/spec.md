# 运行中任务分页 Delta Spec

## ADDED Requirements

### Requirement：运行状态在分页前过滤

系统 SHALL 在授权范围内先筛选 `running` 当前状态，再执行任务 ID 排序、游标和页大小限制。

#### Scenario：前序非运行任务超过一页

- **GIVEN** 多个非运行任务的 ID 排在一个运行任务之前且数量超过页大小
- **WHEN** 调用方请求 `task.list` 且 `running_only=true`
- **THEN** 第一页返回该运行任务
- **AND** 不返回其他状态任务

#### Scenario：旧请求未提供过滤字段

- **WHEN** 调用方省略 `running_only`
- **THEN** 系统沿用既有 `include_finished` 行为
