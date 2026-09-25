# file-authorization Delta

## ADDED Requirements

### Requirement: 原生入口签发任务绑定文件授权

系统 SHALL 只允许可信 Task Space 的本机用户选择文件并为当前非终态任务签发临时授权；路径和授权根不得由 Agent 或 WebView 请求字段提供。

#### Scenario: 用户取消原生选择

- **WHEN** 用户关闭文件选择器或覆盖/回收站确认框
- **THEN** 系统不签发授权、不改变任务状态，也不把取消记录为路径错误

### Requirement: 高风险文件用途要求本机确认

系统 SHALL 在替换和移入回收站用途完成选择后要求原生本机用户二次确认，且 Agent 不能用字段伪造确认。

#### Scenario: Agent伪造确认字段

- **WHEN** Agent 向 Gateway 提交包含路径、用途或确认字段的文件授权请求
- **THEN** 协议拒绝未知字段或不提供写入入口，且不签发或消费授权

### Requirement: 归属 Agent 只读取安全授权摘要

系统 SHALL 在协议 1.27 的 `task.file.grants` 中只向已认证归属 Agent 返回当前任务的授权 ID、用途和到期时间；响应不得包含路径、授权根、文件身份、哈希或正文。

#### Scenario: 非归属Agent查询

- **WHEN** 已认证但非任务归属 Agent 查询该任务的文件授权
- **THEN** Gateway 拒绝请求且不泄露授权摘要

### Requirement: 授权查询保持一次性授权未消费

系统 SHALL 将授权查询视为只读操作，不消费新建、替换或回收站用途的授权。

#### Scenario: 查询后解析新建授权

- **WHEN** Agent 读取包含新建用途的授权摘要后，由后续执行入口首次解析该引用
- **THEN** 首次解析仍可获得授权，后续解析才按 AD-FI-02 拒绝

### Requirement: Agent撤权清理临时文件授权

系统 SHALL 在本机用户禁用或撤销 Agent 前清空其全部内存文件授权；禁用 Agent 不得通过授权查询或后续执行解析重新取得能力。

#### Scenario: 禁用拥有授权的Agent

- **WHEN** 本机用户把持有有效文件授权的 Agent 设置为 disabled 或 revoked
- **THEN** 该 Agent 的授权均被撤销，已建立会话断开且后续查询失败
