# VI-S3 架构设计

## 边界与依赖

Application 新增 `VoiceReplySession` 用例和 `VoiceSynthesisPort`。Windows/macOS Adapter 只负责枚举本地可用声音、合成短句、播放、停止和释放；不得访问 AgentSession、任务存储、Gateway 或另一 Adapter。Desktop 组合根把 AG-S9 的有界回复帧投递给 Application，并投影视图状态。

```text
AgentSession / AG-S9 reply frames
  → VoiceReplySession（Application，状态与顺序所有者）
      → SpeakableTextSegmenter（纯逻辑、无 Markdown 执行）
      → VoiceSynthesisPort
          ├─ macOS 系统原生 TTS Adapter
          └─ Windows 系统原生 TTS Adapter
      → 有界播放状态与可选振幅提示
```

不新增本地 HTTP/TCP 服务，不把 TTS 放入 Agent Connector、Driver 或 React。Rust 类型仍是跨边界模型的唯一来源；若 Spike 证明 UI 事件需要新增协议，须先补充本 ADR，再由 Rust 生成 Schema 与 TypeScript。

## 状态与契约

Application 内存状态为：

`disabled | idle → buffering → playing → stopping → idle`，任意活动态可进入 `failed` 后回到 `idle`。

- 每轮使用稳定 `reply_id + generation`；每个可朗读片段获得严格递增 `sequence`。
- 合成可受控并行，完成结果进入有界重排缓冲；只有 `next_sequence` 可开始播放。
- Agent 终态只表示不再接收正文，不代表音频已经播放完；界面分别展示回复完成和播放完成。
- 新 generation、用户停止、连接断开或开始收音先使旧 generation 失效，再请求 Adapter 停止。迟到回调只能被丢弃。
- 单句失败生成受控跳过结果，后续句子可继续；连续失败达到 Spike 确定的门槛后终止本轮语音并保留文字。

## 文本与音频边界

- 输入只能是 AG-S9 已筛选的最终用户可见正文，不接收 reasoning、工具状态或任务事件。
- 句界处理产生 `display_text` 与 `spoken_text`；去除 Markdown 表现符号不得改变数字、否定词、路径或业务事实。代码块、URL 和表格默认不朗读，只给出短提示。
- 正文、音频和振幅不得持久化。普通日志仅记录 generation 的不透明摘要、阶段、片段数量、时长区间和稳定错误分类。
- 队列、单句字符数、总字符数、音频时长和并发数均须有界；具体数值由双平台 Spike 决定，不能凭单平台实现写死。

## 与语音输入及形象的关系

- VI-S1 请求开始收音时，Application 先调用 VoiceReplySession 停止；停止达到安全边界后再打开麦克风，避免回声被识别为用户输入。
- 播放停止失败不得永久阻止用户收音；进入有界失败态并明确提示可能仍有系统音频，随后由平台证据决定是否允许继续。
- `voice_speaking` 是短生命呈现提示，不是任务状态，也不覆盖执行中的可信任务形象。首版可以只显示回复卡播放控制；嘴型/振幅动画须通过 DS 模块的独立设计接入。

## 失败与验证

候选优先使用 macOS 与 Windows 系统原生本地 TTS。Spike 必须用同一中文样本验证：

1. 本地声音可发现性、离线行为和缺失语音包。
2. 首句与连续十句延迟、并发准备后的严格顺序和内存上限。
3. 播放中停止、立即开始收音、轮次切换、迟到回调和进程退出。
4. Markdown、代码、URL、中英混排、数字与标点的可朗读转换。
5. 系统输出设备切换、不可用和恢复后的稳定错误。

任一平台需要常驻第二进程、隐藏云凭据、无法可靠停止、必须把正文或音频落盘，或无法证明不静默联网时淘汰。双平台结论与独立审阅通过前，AD-VI-03 保持 Proposed，VI-S3 不生成产品实施 Proposal。
