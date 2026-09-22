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

### Requirement: 产品门禁

在双平台统一样本和ADR通过前，产品语音入口 MUST NOT 实现。

#### Scenario: 门禁未通过

- **WHEN** Windows/macOS统一样本或ADR尚未通过
- **THEN** 不得创建产品语音协议、Gateway入口或默认权限请求
