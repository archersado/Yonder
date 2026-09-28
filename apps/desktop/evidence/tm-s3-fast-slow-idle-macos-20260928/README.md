# TM-S3 步骤间显式接管 macOS 证据

- 日期：2026-09-28
- 入口：正式打包的 `Yonda.app`、Unix Domain Socket、协议 1.31
- 场景：真实 `computer.step` 返回后不再发送任何 Gateway 帧，再从顶部控制条点击“接管电脑”
- 结果：控制条主动回放慢脑、快脑与执行投影；AX 点击成功；任务进入 `paused`，控制阶段进入 `stopped`
- 结论：步骤间接管不依赖下一条 Agent 请求，也不会因首个 WebView 事件早于监听器而显示空白。

结构化结果见 `result.json` 与 `native/window-result.json`；截图见 `native/cua-control.png`。
