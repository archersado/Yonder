# TM-S7 Windows 统一执行启动 BUA 验证

日期：2026-11-01。结论：**PASS（Windows BUA `created → running → completed` 子范围）**。

正式 Windows Desktop 通过 Named Pipe Gateway 创建任务后保持 `created`；声明首步骤并调用真实 `browser.execute(create)` 时，经共享启动用例提交执行事实并进入 `running`，随后多轮 Observe/步骤推进均保持同一 ego-lite 外部引用，最终 `browser.execute(finish)` 返回 `completed`、序号 34、事件 34 条及 `finished=true`。

机器证据：`apps/desktop/evidence/ds-s2-windows-20261101/native-task-space-result.json`。汇总报告：`../ds-s2-running-pagination/verification-windows-20261101.md`。全程未执行 CUA，外部 Task Space 已关闭。

Document/Command 首次副作用仍受 FI-S1、DO-S2、CM-S1 产品 Proposal 门禁约束；因此本项只关闭 Windows BUA 原生证据，不 Archive TM-S7。
