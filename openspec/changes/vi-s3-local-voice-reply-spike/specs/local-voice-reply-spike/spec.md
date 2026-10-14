# 本地语音回复 Spike Delta

## ADDED Requirements

### Requirement: 本地候选能力探测
探针 MUST 只使用系统公开本地语音合成能力，并报告中文声音是否可用；不得下载模型、携带云凭据或静默调用网络 TTS。

#### Scenario: 本地中文声音不可用
- **WHEN** 当前系统没有可用的本地中文声音
- **THEN** 探针报告稳定的 `local_voice_unavailable`，且不发起下载或网络回退

### Requirement: 有序异步结果
顺序夹具 MUST 允许片段乱序完成，但只按当前 `generation` 的严格递增 `sequence` 发布；缓存 MUST 有固定上限。

#### Scenario: 后序片段先完成
- **WHEN** sequence 2 在 sequence 1 前完成
- **THEN** sequence 2 保持缓冲，直到 sequence 1 发布后才可发布

### Requirement: 打断与迟到结果隔离
停止或新一轮回复 MUST 先使旧 generation 失效；旧回调不得进入新轮次或形成成功终态。

#### Scenario: 停止后旧回调迟到
- **WHEN** 停止完成后收到旧 generation 的合成或播放回调
- **THEN** 该回调被拒绝，且当前轮次状态不改变

### Requirement: 失败不阻塞文字链路
TTS 不可用、超时、停止失败或单句失败 MUST 形成稳定错误并允许文字回复继续；不得等待 SQLite、任务状态或 Agent 重试。

#### Scenario: 单句合成失败
- **WHEN** 一个已编号片段合成失败
- **THEN** 夹具产生该序号的受控跳过结果，后续序号仍可继续发布

### Requirement: 无正文证据
探针证据 MUST NOT 包含样本文字、声音名称、合成音频、用户正文或完整原生错误 Payload。

#### Scenario: 输出验证结果
- **WHEN** 探针完成统一样本
- **THEN** 输出只包含平台、能力布尔值、计数、毫秒区间和稳定错误分类
