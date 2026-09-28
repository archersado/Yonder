# TM-S3 CUA 规划与执行步骤展示 macOS 证据

日期：2026-09-28。验证构建：`story/tm-s3-cua-step-presentation` 的正式打包 `Yonda.app`。

真实 Local Socket Agent 创建 8 槽位 CUA 计划片段；原生 AX 验证器捕获控制条并读取“规划步骤”与“连续 CUA 步骤 1”，随后以真实“接管电脑”按钮请求接管。

- `result.json`：任务 `paused`、控制 `stopped`、未跑完全部槽位、零新增 `unknown/user-input`，通过。
- `native/window-result.json`：控制条 560×174，当前工作区顶部居中，并确认标题、规划标题、首个计划步骤和按钮。
- `native/cua-control.png`：窗口级原生截图，显示执行步骤、四条有界计划步骤及“另有 4 步”。

本次接管后的 `focus_phase=failed` 是现有 WorkRef 目标不满足定位前提所致；任务仍按安全语义停在 `paused/stopped`。本增量不把该既有定位限制宣称为成功，也不改变其链路。Windows 按用户决定暂缓。
