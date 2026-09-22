# 结构化命令执行 Spike Delta

## ADDED Requirements

### Requirement: 统一技术样本

Spike MUST 使用统一样本验证结构化命令执行，且样本不得接入产品链路。

#### Scenario: 覆盖关键样本

- **WHEN** Spike执行字面参数、超时子进程树和超量输出样本
- **THEN** 参数不得经过Shell解释，超时必须停止完整进程树，保存输出不得超过预算
- **AND** 样本不得接入产品协议、任务库或Gateway

### Requirement: 双平台门禁

在macOS进程组与Windows Job Object证据全部通过前，CM-S1 MUST NOT 进入产品实现。

#### Scenario: 证据不全保持Proposed

- **WHEN** macOS进程组与Windows Job Object证据未全部通过
- **THEN** AD保持Proposed，CM-S1不得进入产品实现
