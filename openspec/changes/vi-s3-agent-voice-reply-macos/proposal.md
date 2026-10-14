# Proposal：VI-S3 macOS Agent 语音回复

关联 [VI-S3](../../../docs/specs/epic-VI/story-VI-S3/README.md) 与 Accepted AD-VI-03 macOS 子范围。Architecture Impact：conforming；新增 Application 短生命语音回复会话与 macOS 平台 Adapter，不改变协议、SQLite、Outbox、Agent Gateway、任务状态或依赖方向。

## Why

AG-S9 已把当前 Agent 的流式用户可见回复送回 Yonder，但产品只能显示文字。用户明确要求正式实现语音回复；已有 macOS Spike 证明系统 TTS 核心候选可保留，并揭示停止回调必须由 generation 门禁约束。

## What Changes

- Application 新增有界 `VoiceReplySession`，负责启停、流式句界、generation 和 sequence。
- macOS Desktop 通过公开系统 TTS 播放已安装中文声音，并把开始、结束、停止和稳定错误投影到回复卡。
- Agent 设置增加默认关闭的“语音回复”开关；未开启时不初始化 TTS。
- AG-S9 delta 在显示文字的同时进入会话；完成时冲刷尾段，失败/中断/关闭/新一轮收音立即停止。
- TTS 不可用或失败只保留文字回复，不阻塞 Agent 与任务运行时。

## Non-goals

不修改 Agent 协议，不朗读 reasoning、工具状态或任务步骤，不持久化正文或音频，不下载声音，不调用云端 TTS，不实现声音克隆、Live2D 或 Windows Adapter。
