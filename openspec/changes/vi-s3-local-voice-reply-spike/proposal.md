# Proposal：VI-S3 本地语音回复 Spike

关联 [VI-S3](../../../docs/specs/epic-VI/story-VI-S3/README.md) 与 Proposed AD-VI-03。Architecture Impact：spike-only；不修改产品协议、SQLite、Agent Gateway、任务状态或正式 Desktop 组合根。

## Why

VI-S3 需要在 Windows/macOS 上把 AG-S9 的有界 Agent 回复异步转换为本机语音，并能可靠停止、按序播放且不静默联网。系统原生 TTS 的中文声音、回调顺序、停止语义、离线性和资源释放尚未用统一样本验证，不能直接进入产品实现。

## What Changes

- 建立不含用户正文的统一中文编号样本和稳定度量格式。
- 用隔离原生探针验证 macOS 系统 TTS 的声音发现、队列顺序、停止、迟到回调与资源释放。
- 用纯逻辑夹具验证 `reply_id + generation + sequence` 重排、失效和有界失败契约。
- 在可用 Windows 环境运行同一样本并对照结论。
- 根据双平台证据接受、修订或拒绝 AD-VI-03；只有接受后才建立产品实施 Proposal。

## Non-goals

不接入真实 Agent 回复，不播放或保存用户正文，不修改 VI-S1 收音，不实现嘴型，不下载声音包，不调用云端 TTS，也不把探针作为长期产品 Adapter。
