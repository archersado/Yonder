# 独立 Verification Goal：macOS 只读任务时间线

日期：2026-09-18  
结论：PASS（首批只读时间线子范围；完整 TM-S5 未通过）

- ego-browser 夹具验证状态变化、步骤声明、observed、unknown、未完整提示及时间线局部失败；详情在局部失败时保留。
- 修复键盘入口沿用悬停自动收起的问题；Enter 打开时聚焦任务面板并保持可操作，悬停入口仍按移出规则收起。
- 正式 macOS 桌宠通过 `yonder mcp` 创建真实验证任务并声明“检查任务时间线”；原生任务详情显示 `#2 Agent 声明步骤：检查任务时间线`。
- Desktop 可信同版本查询读取当前 `task.events` 形状；外部 Agent 仍由 Gateway 协商协议版本。
- 验证后取消任务并保留说明、事件和幂等数据，没有执行删除。
- 证据：[结构化结果](../../../apps/desktop/evidence/task-timeline-20260918/result.json)、[原生截图](../../../apps/desktop/evidence/task-timeline-20260918/native-timeline.jpg)。

验证命令：

```bash
cargo test --offline --locked -p yonder-application -p yonder-desktop
```

Application 11 项、Desktop 6 项通过。Windows 按用户决定暂缓；产物、用户确认、完整分页、配额和清理仍受 TM-S5 设计门禁约束，本 Change 不 Archive。
