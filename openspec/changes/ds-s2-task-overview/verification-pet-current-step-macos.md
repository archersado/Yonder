# DS-S2 形象当前步骤独立 Verification Goal（macOS）

日期：2026-09-28  
结论：PASS（macOS 增量）；Windows 证据按既有安排继续暂缓，DS-S2 保持 `verifying`。

## 验证目标

- 当前执行请求只显示其精确任务的可信步骤，不显示任务名、ID、动作参数或其他任务步骤。
- `computer.step` 使用经 Rust 协议、deadline 与连接身份校验的显式步骤；计划执行使用 SQLite 不可变片段当前槽位；其他执行使用任务已声明步骤。
- 离开 `executing`、没有可信步骤或形象收起时不残留步骤文本。
- 展示失败不阻塞执行，React/JavaScript 不成为任务状态所有者。

## 自动化与集成结果

- `cargo test -p yonder-application`：44 项通过；新增覆盖执行提示只接受可解码、未过期且连接身份一致的请求。
- `cargo test -p yonder-desktop`：lib 11 项、bin 11 项通过；覆盖 created→running 的精确任务步骤投影、非运行态清理及发布契约回归。
- `node --check apps/desktop/ui/pet.js apps/desktop/check-task-space.mjs`：通过。
- `git diff --check`：通过。

## macOS 原生 WebKit 证据

执行：

```text
swiftc apps/desktop/check-pet-current-step-macos.swift -o /private/tmp/check-pet-current-step-macos
/private/tmp/check-pet-current-step-macos apps/desktop/evidence/ds-s2-pet-current-step-macos-20260928
```

结构化结果 `result.json`：

- executing：文本为“正在：打开企业微信”，CSS `display=block`，`data-has-step=true`；无障碍名称包含“当前步骤：打开企业微信”。
- waiting_for_user：文本为空，CSS `display=none`，`data-has-step=false`；无障碍名称不再包含当前步骤。
- 隔离夹具未连接或修改正式 TaskStore；真实状态/步骤投影由 Rust 集成测试覆盖。

截图：`apps/desktop/evidence/ds-s2-pet-current-step-macos-20260928/native-webkit-current-step.png`。

## 边界

- 本 Goal 不宣称 Windows UI 已验证。
- 不新增协议字段、SQLite 表或任务状态；步骤状态条只是现有 Application 事实的窗口级瞬时投影。
