# file-authorization Delta

## ADDED Requirements

### Requirement: 文件授权只由可信本机用户签发

系统 SHALL 只允许可信 LocalUser 为当前任务签发绑定归属 Agent 和用途的临时文件引用，Agent 请求字段不能签发或扩大授权。

#### Scenario: Agent尝试签发

- **WHEN** Agent 身份调用文件授权签发
- **THEN** 在注册引用前拒绝且不访问正式文件能力

### Requirement: 授权绑定精确文件事实

系统 SHALL 对既有文件保存规范路径、身份和 SHA-256，对新目标保存规范路径和父目录身份，不保存文件正文。

#### Scenario: 新目标越出授权根或已存在

- **WHEN** 签发新建授权时父目录逃逸或目标已经存在
- **THEN** 拒绝签发且不创建、截断或替换任何文件

### Requirement: 解析重新校验任务和会话边界

系统 SHALL 在每次解析时校验当前 Agent、任务归属、任务标识、用途和有效期。

#### Scenario: 跨任务或用途使用

- **WHEN** 同一 Agent 把读取引用用于另一个任务或写入用途
- **THEN** 返回权限拒绝且授权保持原绑定

### Requirement: 副作用授权只能消费一次

系统 SHALL 允许读取引用在有效期内复用，并在首次成功解析新建、替换或回收站引用时原子消费。

#### Scenario: 并发解析写引用

- **WHEN** 两个请求同时解析同一写引用
- **THEN** 至多一个获得授权，另一个返回引用不存在，不自动重放

### Requirement: 授权有界且不跨重启

系统 SHALL 将授权限制为进程内最多 256 项和最长 15 分钟，到期、撤销或重启后失效。

#### Scenario: Registry达到容量

- **WHEN** 仍有效授权已达上限
- **THEN** 新签发失败关闭，不静默删除任一有效授权
