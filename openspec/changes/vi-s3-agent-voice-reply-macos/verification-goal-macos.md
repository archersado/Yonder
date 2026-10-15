# Verification Goal：VI-S3 macOS Agent 语音回复

状态：PARTIAL（实现与产品设置通过；真实有声回复链路待复跑）  
日期：2026-10-15
范围：`vi-s3-agent-voice-reply-macos`

## Goal

独立于实现逻辑验证 macOS 产品包遵守默认关闭、显式开启、本地声音、流式有序、可停止、文字优先和无持久化旁路的契约。Windows 不在本 Goal 中，也不得用本结果替代。

## 已通过

- `cargo test -p yonder-application voice_reply --locked`：7/7 PASS，覆盖默认关闭、句界与尾段、Markdown/代码/URL、有界 generation、停止后迟到回调、原生空闲先于回复终态与独立试听 generation。
- `cargo test -p yonder-desktop --lib --locked`：37/37 PASS；`cargo test -p yonder-desktop --bin yonder-desktop --locked`：11/11 PASS。
- `node apps/desktop/check-voice-reply.mjs`、`check-voice-session.mjs`、`check-region-submit-recovery.mjs`、`check-region-preview-drag.mjs`：PASS，覆盖设置、停止入口、回复卡投影和既有圈选交互不回归。
- `cargo check -p yonder-desktop --locked`：PASS。
- 从当前分支构建并临时签名正式 debug `Yonda.app` 后，`swift apps/desktop/check-voice-reply-macos.swift <evidence-dir>`：PASS。产品 Agent 设置中开关、仅本地说明和持久化均可由 Accessibility 读取；见 `apps/desktop/evidence/voice-reply-macos-20261014/result.json`。
- 实现只把 generation、sequence 和封闭 phase 投影给 UI；正文、声音名称和音频不进入 SQLite、事件、Outbox 或普通日志。
- 2026-10-15 从 `dev` 提交 `c95d567` 构建并临时签名正式 debug `Yonda.app`；语音回复开关为开启，macOS 可枚举本地中文声音。用户从 Yonder Agent 管理的“试听”入口启动固定测试句，明确确认可正常播放。该路径直接复用正式 `VoiceReplyRuntime → NativeVoiceSynthesisAdapter → AVSpeechSynthesizer`，不依赖 Agent 回传；因此已排除本地 TTS、中文语音包与基本播放路径故障。

## 尚未通过

- 真实 `AgentSession → agent.reply started/delta/completed → AVSpeechSynthesizer → 停止` 样本未完成。候选包重启后，当前正式 Yonder MCP 会话返回 `Broken pipe (os error 32)`；随后从产品菜单打开圈选入口时未能跨过既有桌面暂停边界，未生成可提交的回复卡。未使用直连 Socket、SQLite 或伪造通知绕过该阻断。
- 当前环境未授予屏幕录制权限，产品设置验证只有结构化 Accessibility 日志，没有截图。
- 真实断网、输出设备切换、系统静音、屏幕阅读器重复朗读与 Agent 回复中的有声停止仍待人工/设备样本。
- Windows Adapter 与同样本验证为 Deferred。

## 结论

macOS 产品实现、自动化合约、正式 GUI 设置与独立本地发声已通过；故障范围收窄为 Connector 没有把 Agent 输出通知交给 Yonder。本 Goal 尚不能标记 PASS，OpenSpec 不 Archive，VI-S3 不标记 Done；必须修复回传后复跑真实回复与停止样本。
