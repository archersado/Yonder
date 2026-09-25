# file-runtime Delta

## ADDED Requirements

### Requirement: 文件身份与授权根不可由路径别名绕过

系统 SHALL 只处理规范化后位于授权根内的绝对路径，并以平台文件身份归并既有文件别名。

#### Scenario: 软链接或硬链接指向同一文件

- **WHEN** 调用方经不同别名读取或申请写入同一文件
- **THEN** 返回相同文件身份，且同一时间只允许一个写租约

#### Scenario: 链接逃逸授权根

- **WHEN** 目标或既有父目录经链接解析后位于授权根外
- **THEN** 在读取或创建暂存文件前拒绝

### Requirement: 读取与原子写入有界且抗陈旧

系统 SHALL 将单文件内容限制为 16 MiB，并让替换绑定读取时的身份与 SHA-256。

#### Scenario: 新建文件

- **WHEN** `create-new` 目标不存在且内容通过调用方校验
- **THEN** 在同目录写入、同步并原子提交；并发出现同名目标则拒绝而不是覆盖

#### Scenario: 替换文件

- **WHEN** 当前身份或 SHA-256 与请求不一致，或宿主持有冲突锁
- **THEN** 不提交暂存内容，返回稳定冲突或锁定结果

#### Scenario: 提交持久性不明

- **WHEN** 原子 rename 已发生但父目录同步失败
- **THEN** 返回 unknown，不自动重试或删除可能已提交的目标

### Requirement: 删除只进入系统回收站

系统 SHALL 只允许可信本机用户用例把身份匹配的文件移入系统回收站。

#### Scenario: Agent 请求删除

- **WHEN** Agent 身份或可伪造确认字段请求删除
- **THEN** Application 在调用 Adapter 前拒绝

#### Scenario: 本机用户移入回收站

- **WHEN** 可信 LocalUser 请求的文件身份匹配且位于授权根
- **THEN** 使用系统回收站 API；不提供永久删除兜底

### Requirement: 未验证平台失败关闭

系统 SHALL 只在已验证的 macOS 路径提供 File Runtime。

#### Scenario: Windows 或其他平台调用

- **WHEN** 未验证平台调用 File Adapter
- **THEN** 返回 `unsupported-platform`，不得回退到路径字符串锁、直接删除或非原子复制
