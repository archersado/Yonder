# EX-S5 已验证执行经验召回

## Why

执行成功经验目前只存在于一次慢脑会话或人工回归记录，发生 handback 后不能自动为后续 replan 提供帮助。用户要求先召回历史成功策略，再由慢脑基于新鲜任务事实规划。

## What Changes

- 新增 Application 经验值对象与 `ExecutionExperienceStore` Port。
- 新增 SQLite 有界建议投影、精确索引和聚合计数。
- handback 唤醒慢脑前召回最多三条经验，并通过既有 AG-S5 输入通道注入。
- 召回失败不阻塞任务；不保存或复用坐标、正文、截图、transcript、临时元素身份。

## Architecture Impact

关联 EX-S5、Accepted AD-EX-06 与 ARCHITECTURE-SPINE。新增的是事后建议投影，不改变 ExecutionRuntime 活动状态所有权、Gateway 边界或 Driver/Jev 职责。
