# TM-S3 CUA 待核实投影 macOS 证据

2026-09-29 使用正式 debug `Yonda.app`、生产 Local Socket 与真实 trycua 0.25.0 复跑相同隔离样本。Worker 未能给出可接受的确认时，Yonder 将 attempt 记录为 unknown、计划片段 HandBack，并在顶部控制条显示黄色 !；AX 可通过“待核实：连续 CUA 步骤 1”读取同等语义，未出现成功 ✓。验证结束后任务为 `cancelled`。

- `result.json`：无正文的 unknown/HandBack 结论。
- `native/window-result.json`：顶部位置、尺寸和可访问语义结果。
- `native/cua-control.png`：原生窗口截图。

该证据验证失败闭合，不将截图成功、Worker 退出或工具调用本身当成业务成功。Windows 按既有决定暂缓。
