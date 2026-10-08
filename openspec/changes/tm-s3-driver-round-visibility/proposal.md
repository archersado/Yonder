# Proposal：TM-S3 Driver 轮次可见性

## Why

用户在真实 CUA 任务中只能看到当前步骤和快慢脑摘要，无法判断 Driver 每轮是否真正派发、是否完成 Observe、为何进入下一轮。

## What Changes

- 顶部控制条展示当前任务最近六轮 Driver 摘要：轮次、步骤、封闭动作类别、阶段和脱敏结论。
- 派发前显示 executing，Driver 返回后更新为 observed、failed 或 unknown。
- 同任务 replan 保留此前轮次；任务终结或接管后清理。
- Driver 轮次成功不等于用户目标完成，最终结论仍由慢脑目标核验。

## Impact

影响 `apps/desktop` 的 `CuaControlHub`、CUA Port 装饰器和顶部控制条 UI。不新增 wire 协议、SQLite 字段、事件类型或第二状态机。
