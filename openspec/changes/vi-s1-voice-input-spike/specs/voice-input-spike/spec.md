# 单轮语音输入 Spike Delta

## ADDED Requirements

### Requirement: 默认关闭

Spike MUST 在用户未显式开始时保持完全关闭。

#### Scenario: 未显式开始

- **WHEN** 用户未显式开始语音输入
- **THEN** Spike不得请求权限、开启麦克风或启动ASR

### Requirement: 统一能力清单

Spike MUST 在双平台返回统一只读能力清单，不采集音频。

#### Scenario: 运行只读探针

- **WHEN** 在Windows或macOS运行只读探针
- **THEN** 返回平台框架、中文识别器、离线能力和权限状态
- **AND** 不采集、保存或上传音频

### Requirement: 显式采集闭环

Spike MUST 提供显式开始、部分结果、最终结果、取消和停止的完整闭环。

#### Scenario: 用户显式采集

- **WHEN** 用户显式开始并授予权限
- **THEN** 候选必须提供部分结果、最终结果、取消和停止
- **AND** 停止后释放采集资源与有界音频队列

#### Scenario: 有效语音后的短暂停顿

- **WHEN** 本轮已经检测到持续有效语音，随后达到候选静音阈值
- **THEN** Spike必须只结束一次采集并等待ASR最终结果
- **AND** 只把整轮最终非空转写作为一条输入投递
- **AND** 部分转写不得触发投递

#### Scenario: 初始静音或噪声

- **WHEN** 本轮在最长会话时限内没有达到有效语音门槛
- **THEN** Spike必须结束本轮且不得投递任何输入
- **AND** 不得把最长会话时限冒充为说话后静音判定

#### Scenario: 多个结束信号竞争

- **WHEN** 手动停止、说话后静音、最长会话时限或取消并发到达
- **THEN** 同一会话最多执行一次采集收尾
- **AND** 同一会话最多产生一个最终结果和一次Agent输入投递

#### Scenario: 迟到回调

- **WHEN** 已结束会话的ASR回调在下一会话开始后迟到
- **THEN** Spike必须按稳定会话标识拒绝该回调
- **AND** 不得把旧转写投递到新会话

### Requirement: 产品门禁

在双平台统一样本和ADR通过前，产品语音入口 MUST NOT 实现。

#### Scenario: 门禁未通过

- **WHEN** Windows/macOS统一样本或ADR尚未通过
- **THEN** 不得创建产品语音协议、Gateway入口或默认权限请求
