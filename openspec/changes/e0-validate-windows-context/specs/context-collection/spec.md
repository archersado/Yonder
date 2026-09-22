# 上下文采集规范增量

## ADDED Requirements

### Requirement: Windows 前台上下文

系统 SHALL 通过 Windows 原生 API 采集当前前台应用和窗口元数据，并以事件驱动方式更新。

#### Scenario: 前台窗口变化

- **WHEN** 用户切换前台窗口
- **THEN** 系统在 300ms 内更新应用、进程与窗口元数据
- **AND** 不扫描文件系统或浏览器 History 数据库

### Requirement: 浏览器上下文通道

系统 SHALL 由 Chrome/Edge 扩展通过 Native Messaging 上报浏览上下文。

#### Scenario: 普通窗口上报

- **WHEN** 已授权扩展在普通窗口产生上下文事件
- **THEN** Host 接受长度前缀 JSON 消息
- **AND** 仅精确匹配的扩展来源可以连接

#### Scenario: 隐私窗口

- **WHEN** 事件来自隐私窗口
- **THEN** 扩展不得向 Host 发送该事件

### Requirement: 显式 Recording

系统 SHALL 默认关闭用户操作采集，且仅在用户手动开始 Recording 后安装操作钩子。

#### Scenario: 停止录制

- **WHEN** 用户停止 Recording 或进程异常恢复
- **THEN** 操作钩子被解除
- **AND** 后续用户输入不被采集
