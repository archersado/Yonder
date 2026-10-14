# VI-S3 macOS 语音回复设置证据

2026-10-14 从 `story/vi-s3-agent-voice-reply-macos` 构建、打包并临时签名正式 debug `Yonda.app`，通过 macOS Accessibility 操作产品“Agent 管理”窗口。

- `result.json` 仅记录候选包、显式开关、本地边界说明、设置持久化与截图可用性等布尔事实，不包含回复正文、声音名称、音频或窗口树。
- 显式“使用本地中文声音朗读 Agent 回复”开关可见；开启后 `yonda.voiceReply.enabled` 持久化为 true。
- “只使用本机已安装的中文声音”边界文案可见。
- 当前运行环境没有授予屏幕录制权限，因此未保存窗口截图；结构化 Accessibility 结果为 PASS。

本证据不声明真实 Agent 回复已经有声播放。当前正式 MCP 会话在候选包重启后返回 `Broken pipe`，圈选入口也未能跨过既有桌面暂停边界；真实 `AgentSession → agent.reply → TTS → 停止` 样本仍在 Verification Goal 中保持待验证。
