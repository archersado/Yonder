# Browser Gateway Delta Spec

## ADDED Requirements

### Requirement: Agent通过统一Gateway执行BUA

Gateway MUST 只为已认证任务所有者执行Browser动作，MUST 由Yonder生成执行身份并读取已持久化引用。

#### Scenario: 多步Browser任务

- **WHEN** Agent声明步骤并执行create，确认边界后依次声明和执行后续动作
- **THEN** 所有动作进入同一任务时间线、同一Browser资源占用和同一外部Task Space
