# TM-S3 快慢脑过程投影 macOS 证据

- 日期：2026-09-28
- 入口：正式打包的 `Yonda.app`、Unix Domain Socket、协议 1.31
- 场景：慢脑经 Gateway 提交 8 槽位计划，首槽位含 2 个候选并真实调用面板配置的 Jev
- 结果：顶部控制条展示“慢脑已提交 8 个受限步骤”、计划列表、快脑决策“已交回慢脑重新观察或规划”及当前准备执行步骤
- 安全边界：只展示有界摘要，不展示模型思维链、候选参数、输入正文或完整 Agent Payload
- 清理：观察完成后通过正式 `task.cancel` 取消验证任务，状态为 `cancelled`

结构化结果见 `result.json` 与 `native/window-result.json`；截图见 `native/cua-control.png`。
