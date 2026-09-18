# 独立 Verification Goal：任务详情显示 Agent 当前步骤

日期：2026-09-15  
关联 Story：DS-S2 STEP-UI01～03、AG-S3  
结论：PASS（页面回归与 macOS 正式桌面；Windows 暂缓）

## 结果

- 详情选择只发一次 `task.step.get`，同一响应提供任务快照和最近声明；列表不逐项读取步骤。
- ego-browser Task Space 21 回归通过：21项分页、步骤标签与身份、读取失败保留旧列表、筛选重置均通过。测试夹具补齐已有 Agent 名称规则，不回退用任务 ID 冒充名称。
- macOS 使用唯一 bundle ID `com.yonder.desktop` 连接正式 Yonda；在“全部”中选择真实保留任务“Agent步骤声明验证”，原生辅助功能树与截图均显示：任务 `cancelled@3`、步骤“打开目标文档”、`open-document · 接受序号 2`。
- 任务状态与步骤接受序号分开呈现，没有把声明表达为正在执行；没有人工创建或编辑步骤。

## 证据

- `apps/desktop/evidence/task-step-ui-20260915/browser-result.json`
- `apps/desktop/evidence/task-step-ui-20260915/native-cua-result.json`
- 本次 CUA 原生截图与辅助功能树取证，目标 bundle ID 为 `com.yonder.desktop`。
- `node --check apps/desktop/ui/task-space.js`
- `cargo build --offline --locked -p yonder-desktop`

首次按显示名称连接 CUA 时误启动旧 `Yonder E0` Spike，形成两个桌宠；该 PID 已关闭并确认只保留正式 Yonda。后续原生验证固定使用 bundle ID，旧 Spike 不作为通过证据。

Windows 依用户决定继续暂缓；本项不关闭 DS-S2 的生产认证、执行/接管或双平台总门禁。
