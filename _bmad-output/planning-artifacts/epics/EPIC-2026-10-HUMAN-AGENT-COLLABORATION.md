# 2026 年 10 月 Epic：人机协作闭环

## 目标

在 Windows 首版 CUA 主干上完成一次可见、可暂停、可接管、可归还、可验证的人机协作任务。10 月交付重点不是扩大自动化覆盖率，而是让用户能够安全介入 Agent 的执行过程。

## 主干场景

第三方本地 Agent 通过 Agent Gateway 创建任务；Yonder 展示当前目标、步骤和状态。Agent 执行每一步后 observe，并按需局部 replan。用户可暂停并接管桌面，完成登录、纠错或人工判断；用户明确归还后，Yonder 重新 observe，Agent 从新状态继续。结束时用户能检查结果与时间线。

## Stories

### OCT-S1 任务状态可见

桌宠和 Agent Gateway 同步展示 `created`、`running`、`waiting-for-user`、`paused`、`completed`、`failed`、`cancelled`，并包含当前步骤、最近观察和下一步意图。

### OCT-S2 暂停、取消与用户接管

用户可以暂停或取消 CUA；Yonder 停止派发新动作。接管期间 Agent 不得继续输入，状态进入 `waiting-for-user` 或 `paused`。

### OCT-S3 明确归还与局部重规划

用户主动点击“继续交给 Agent”后，Yonder 重新观察当前窗口和应用状态。只有受影响的后续步骤允许调整；已确认完成的步骤不得无理由重放。

### OCT-S4 结果确认与协作时间线

任务完成或失败时展示关键动作、用户接管区间、observe 结论、replan 原因和产物。结果未确认时不得把 `unknown` 标记为成功。

### OCT-S5 手动示教最小闭环

用户显式开启 Record，完成一段文件或办公软件操作后停止；Yonder 生成可检查的 CUA 动作草稿。用户确认后才能回放，默认不后台记录。

## 10 月验收演示

1. 本地 Agent 创建一个文件资源管理器或普通权限办公任务。
2. Yonder 展示任务和当前步骤，并持续向 Agent 报告状态。
3. Agent 执行至少两步，每步均有 action → observe → decision 记录。
4. 用户中途接管并改变窗口或目标文件状态。
5. 用户明确归还，Agent 重新观察并仅局部调整计划。
6. 任务完成，用户查看结果和完整协作时间线。
7. 另完成一次手动 Record → 审阅动作草稿 → Replay。

## 不在 10 月范围

- ego-lite BUA：Windows Runtime 尚未提供。
- 云端服务端实现。
- 完整细粒度授权系统。
- High integrity 应用输入、整体提权或未签名辅助进程。
- 多 Agent 并发控制同一桌面。

## 研发顺序

每个 Story 独立建立 OpenSpec Change 和 Verification Goal，按 `OCT-S1 → S2 → S3 → S4 → S5` 实施。S2 未通过前不得实现自动恢复；S3 未通过前不得宣称支持人机协作。
