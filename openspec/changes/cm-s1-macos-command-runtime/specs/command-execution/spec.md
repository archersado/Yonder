# command-execution Delta

## ADDED Requirements

### Requirement: 结构化命令不得经过 Shell

系统 SHALL 只以绝对 program、字面 args、规范化 cwd 和显式 env 启动命令，并清空未声明环境。

#### Scenario: 参数包含 Shell 语法

- **WHEN** 参数包含重定向、管道或命令替换字符
- **THEN** 字符按原样传给 program，不解释、不创建额外进程副作用

#### Scenario: 路径或输入越界

- **WHEN** program/cwd 不是可规范化绝对路径，或参数、环境、超时超过限额
- **THEN** 启动前稳定拒绝，不创建子进程

### Requirement: 输出和进程树停止有界

系统 SHALL 并行排空 stdout/stderr，各保留最多 64 KiB，并在超时、取消或超限时停止完整进程组。

#### Scenario: 输出超过预算

- **WHEN** stdout 或 stderr 超过 64 KiB
- **THEN** 停止整个进程组，返回 `output-limit-exceeded` 和有界前缀，不报告普通成功

#### Scenario: 超时或取消

- **WHEN** 到达超时或可信调用方发出取消
- **THEN** TERM/KILL 父进程及全部后代；确认组消失后返回对应结果，否则返回 unknown

### Requirement: 平台能力失败关闭

系统 SHALL 只在已验证的 macOS 路径提供 Runtime。

#### Scenario: 未验证平台调用

- **WHEN** Windows 或其他未验证平台调用 Command Adapter
- **THEN** 返回 `unsupported-platform`，不得回退到 Shell、CUA 或只停止父进程的实现
