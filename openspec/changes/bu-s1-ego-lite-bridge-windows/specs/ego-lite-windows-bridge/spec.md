# Windows ego-lite Bridge Delta

## ADDED Requirements

### Requirement: Windows 复用 ego-lite Task Space

Windows BUA MUST 通过已安装 ego-lite 的 `ego-browser nodejs` Runtime 复用外部 Task Space，不得复制浏览器执行模型或回退 CUA。

#### Scenario: 完成外部 Task Space 生命周期

- **WHEN** Windows 已安装可信 ego-lite Runtime，且 Application 派发合法 BrowserAction
- **THEN** Bridge 必须支持 create、observe、handOff、takeOver 和 finish
- **AND** 返回稳定 external_task_ref、所有权与托管页数量

### Requirement: 可信 Runtime 与有界失败

Windows Bridge MUST 只调用可信安装根中的 Runtime，并对失败关闭。

#### Scenario: 多版本安装优先使用可信 active Runtime

- **WHEN** 固定 Citro Labs 安装根存在多个数字版本，且当前用户固定 active-version 指针解析到其中一个合法 `ego-browser.exe`
- **THEN** Desktop 必须使用该 active Runtime，即使安装根还存在版本号更高但未激活的目录
- **AND** 指针真实目标越出固定安装根、父目录非数字版本或文件名不匹配时必须拒绝该指针
- **AND** Agent、网页、任务参数和 PATH 均不得覆盖 Runtime 路径

#### Scenario: PowerShell 安全启动 UTF-8 Worker

- **WHEN** Windows Bridge 派发包含非 ASCII 任务数据的合法 BrowserAction
- **THEN** Bridge 必须把 Worker 写入当前用户私有 UTF-8 `.mjs`，并通过固定 `powershell.exe` 命令模板执行 Adapter 生成的 ASCII 文件 URL 入口
- **AND** Agent、网页和任务数据不得形成 PowerShell 命令文本或 Runtime 路径
- **AND** Bridge 必须在子进程结束、失败或超时后清理脚本与结果文件

#### Scenario: Runtime 缺失或不可达

- **WHEN** active 指针不可用且固定安装根中没有合法 `ego-browser.exe`，或 bootstrap 不可达、超时、响应不可信
- **THEN** Bridge 必须返回明确 unavailable/unknown
- **AND** 不得通过另一条 Shell 命令、CUA 或另一浏览器引擎重试
