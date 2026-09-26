# 文件授权增量规范

## ADDED Requirements

### Requirement: 归属 Agent 只能用受授权引用执行有界文件操作

Yonder MUST 仅允许完成协议 1.28 握手、已登记且拥有任务的 Agent，通过当前任务的 `grant_id` 执行与授权用途严格一致的读取、创建、替换或回收站操作。请求 MUST NOT 含路径、授权根、文件身份、哈希、确认布尔值、Shell 或额外字段。

#### Scenario：跨 Agent 或用途执行被拒绝

- **WHEN** 非归属 Agent 或以 `read` 授权请求替换/回收站
- **THEN** Gateway 在调用 File Port 前拒绝，且不泄露位置事实

### Requirement: 正文传输有界且不泄露到任务事实

Yonder MUST 将 `task.file.execute` 的读写原始正文限制为 48 KiB，使用规范 Base64 表示；超限或非规范编码 MUST 被拒绝且不截断。路径与完整正文 MUST NOT 写入任务状态、事件、Outbox、日志或 Task Space。

#### Scenario：超限写入不触碰文件系统

- **WHEN** Agent 提交解码后大于 48 KiB 的写入正文
- **THEN** 请求失败关闭，授权和目标文件均不被消费或修改

### Requirement: 副作用遵循统一启动与一次性授权语义

Yonder MUST 在首次文件副作用前使用 TM-S7 的同事务启动事实。创建、替换与回收站授权 MUST 在成功解析时原子消费；读取授权可复用。未知副作用 MUST NOT 自动重试。

#### Scenario：替换后结果未知

- **WHEN** 已消费的替换授权在 File Port 提交后返回 unknown
- **THEN** Yonder 保留可观察 unknown 结果且拒绝以同一授权再次执行
