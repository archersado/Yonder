# Agent 语音回复 Delta

## ADDED Requirements

### Requirement: 显式启用
Yonder MUST 默认关闭语音回复；只有用户在 Agent 设置显式开启后，当前 Agent 的用户可见回复才可进入本地 TTS。

#### Scenario: 默认状态收到回复
- **WHEN** 用户未开启语音回复且 AG-S9 收到回复 delta
- **THEN** 文字正常显示，系统 TTS 不初始化也不播放

### Requirement: 流式有序朗读
Yonder MUST 在稳定句界产生有界片段并按当前 reply generation 的 sequence 顺序朗读；Agent 文字显示不得等待 TTS。

#### Scenario: 首句到达
- **WHEN** 当前回复形成第一个稳定可朗读句子
- **THEN** 该句进入本地 TTS，回复卡继续接收后续文字 delta

### Requirement: 停止先于终态解释
用户停止、关闭卡片、关闭设置、Agent 失败/中断或开始新一轮收音时，Yonder MUST 先使当前 generation 失效再停止原生播放；旧回调不得形成新轮次状态。

#### Scenario: 停止后收到 didFinish
- **WHEN** 原生 Adapter 在停止后为旧 generation 返回完成回调
- **THEN** Application 丢弃该回调，并保持当前轮次未被污染

### Requirement: 文字优先降级
TTS 不可用、超时或失败 MUST 保留文字回复并显示稳定提示；不得阻塞 Agent、Gateway、任务执行或下一轮输入。

#### Scenario: 没有本地中文声音
- **WHEN** 用户开启语音回复但系统没有已安装的中文声音
- **THEN** 回复卡显示“本地语音不可用，已保留文字回复”，文字继续完成

### Requirement: 短生命与无旁路
回复正文、合成音频、声音名称和完整错误 MUST NOT 进入 SQLite、事件、Outbox、索引或普通日志；Adapter MUST NOT 下载声音或调用网络 TTS。

#### Scenario: 语音回复结束
- **WHEN** 本轮完成、停止、失败或窗口关闭
- **THEN** Yonder 清理该轮语音缓冲，且任务事实和持久化内容没有新增正文或音频

### Requirement: 本地播放独立试听
Yonder MUST 在语音回复设置中提供显式试听，以同一正式 TTS Runtime 播放固定无业务内容文案，不依赖 Agent 回复链路。

#### Scenario: 试听本地中文声音
- **WHEN** 用户已开启语音回复、本地中文声音可用并点击“试听”
- **THEN** Yonder 停止当前播放并用新 generation 朗读固定测试句
- **AND** 设置页可见显示准备、播放和完成状态
- **AND** 不创建 Agent 输入、任务、事件、Outbox 或持久化正文
