# document-file-runtime Delta

## ADDED Requirements

### Requirement: 文档读取复用受控文件快照

系统 SHALL 只把 File Port 返回的有界字节交给 Document Port，并返回同一身份与 SHA-256 对应的语义快照。

#### Scenario: 读取受支持OOXML

- **WHEN** 授权根内的 DOCX、XLSX 或 PPTX 通过 File Port 读取
- **THEN** 返回格式、语义文本、规范路径、文件身份和哈希，不暴露XML

### Requirement: 默认另存绑定源快照

系统 SHALL 默认创建新输出，并在源身份租约与共享宿主锁下保持源快照到提交边界。

#### Scenario: 源文件在转换期间变化

- **WHEN** 提交前源身份或 SHA-256 不再匹配请求
- **THEN** 返回冲突且不创建正式输出

#### Scenario: 目标已经存在

- **WHEN** 另存目标在提交前已经存在
- **THEN** 拒绝覆盖，保留目标内容并清理本次暂存文件

### Requirement: OOXML 暂存结构必须复验

系统 SHALL 在原子提交前用 Document Port 重新 inspect 暂存字节，并要求格式与源格式一致。

#### Scenario: 转换结果不是有效同格式OOXML

- **WHEN** 暂存内容无法解析或格式变化
- **THEN** 返回校验失败，不产生正式输出

### Requirement: 覆盖原文件只接受可信本机用户

系统 SHALL 只允许可信 LocalUser 覆盖源文件，并绑定读取时身份与 SHA-256。

#### Scenario: Agent请求覆盖

- **WHEN** Agent身份或Agent字段请求覆盖
- **THEN** 在调用File Adapter前拒绝，原文件不变

#### Scenario: 本机用户覆盖陈旧文件

- **WHEN** 当前身份、哈希或宿主锁不满足
- **THEN** 返回对应冲突或锁定错误，不绕过锁、不自动重试
