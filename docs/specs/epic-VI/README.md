# Epic VI：语音交互

Epic: VI

## 模块边界

负责 Windows/macOS 上用户显式启动的音频采集、转写会话与语音状态；不实现 Agent、语义规划、任务创建或会议纪要生成。原始音频默认只驻留内存，语音内容不得进入普通日志。

## Stories

- [VI-S1 单轮语音输入与转写](story-VI-S1/README.md)
- [VI-S2 会议伴听与实时转写](story-VI-S2/README.md)

## 验收与依赖

实施前必须完成 Windows/macOS 统一样本 Spike 和 Architecture Decision。跨模块只通过 Application Port；React 不拥有采集会话，Adapter 不直接创建任务或调用另一 Adapter。单平台证据不能关闭Story。
