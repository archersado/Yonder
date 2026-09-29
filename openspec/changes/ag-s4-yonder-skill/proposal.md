# Proposal：AG-S4 Yonder Agent Skill

关联 Story AG-S4、Accepted AD-AG-05/AD-AG-09、EX-S2 计划片段和四类 macOS Gateway 独立验证。Architecture Impact：conforming；新增仓库发布的 Agent 使用说明与元数据，不改变 Runtime、协议、状态所有者、持久化或依赖方向。

## Why

Yonder 已发布 BUA、CUA、Document 与 Command 的 macOS Gateway，但外部慢脑缺少统一、可发现且与产品边界一致的编排说明，容易产生重复任务、逐动作规划、旁路执行或把动作回执误当成功。

## What Changes

- 在 `skills/yonder/` 发布版本化 Skill，包含主生命周期与四个按需能力模块。
- 最低协议声明为 1.31，实际能力继续按 MCP 工具发现；缺能力失败关闭。
- CUA 明确一次提交完整多步计划片段、同槽位替代候选、元素优先与视觉兜底、片段内连续 Observe 及片段外交回。
- 固定记录 ego-browser `2.0.0` 来源，保留单 Task Space、同空间恢复、Page复用、交接与清理语义。
- 安装包不含执行代码、凭据、用户数据、协议副本或 Driver。

## Non-goals

不新增Yonder工具、协议、Planner、Jev循环、Runtime或任务状态；不把Windows标为已验证；不以Skill文本代替四类正式产品E2E。
