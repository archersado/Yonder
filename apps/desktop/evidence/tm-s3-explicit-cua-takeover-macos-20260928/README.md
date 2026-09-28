# TM-S3 CUA 显式接管 macOS 证据

- `result.json`：正式 `Yonda.app`、生产 UDS 与真实 trycua 的结构化结果。计划片段在第一个已观察动作后响应控制条接管，没有继续执行全部 8 个槽位；任务为 `paused`、控制为 `stopped`，新尝试没有 `unknown/user-input`。
- `native/window-result.json`：原生 AX/WindowServer 结果。控制条尺寸 460×68，位于当前工作区顶部 16pt 且横向居中；主文案和按钮可访问，AXPress 成功。
- `native/cua-control.png`：同轮控制条窗口截图，只含状态和步骤摘要，不含输入正文或桌面内容。用户现场确认顶部位置与圈选工具条交互样式符合预期。

本轮初始工作目标不满足既有 WorkRef 精确定位条件，因此 `focus_phase=failed`；按既有 AD-TM-08 语义任务保持暂停且不伪报定位成功。精确工作定位已有 `tm-s3-takeover-work-focus` 独立成功证据，本增量验收的是停止触发与控制条入口，不以定位失败掩盖接管停止结果。
