# 文档执行增量规范

## ADDED Requirements

### Requirement: Agent 文档另存必须同时具备源和输出授权

Yonder MUST 仅允许完成协议握手且拥有任务的 Agent，以同一任务的 `read` 源授权和 `create-new` 输出授权执行唯一文本替换另存。请求 MUST NOT 包含路径、授权根、XML、ZIP 或覆盖确认字段。

#### Scenario: 缺少输出授权

- **WHEN** Agent 只提供源读取授权请求文档执行
- **THEN** Gateway 在调用 Document 或 File Port 前拒绝请求且不泄露位置

### Requirement: 文档 Gateway 保持源快照与原子另存边界

Yonder MUST 要求 `expected_hash` 等于源授权快照，并使用受控 File Port 原子新建输出。源变化、目标存在、锁冲突、格式校验失败或未知提交 MUST NOT 伪报成功或自动重试。

#### Scenario: 源文档发生变化

- **WHEN** 授权后源哈希与请求的 `expected_hash` 不一致
- **THEN** 不产生输出并返回可观察失败分类
