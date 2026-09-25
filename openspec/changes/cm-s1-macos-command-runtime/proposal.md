# CM-S1 macOS 结构化命令 Runtime

Story：CM-S1。Architecture Impact：architecture-change。关联 Accepted AD-CM-01、TM-S7、AD-TM-13。

## 动机

macOS Spike 已证明 Rust 标准库可保持字面参数、停止完整进程组并限制输出，但产品代码尚无 Command Port/Adapter。用户明确 Windows 验证可延期，因此先实现 macOS-only Runtime，并让其他平台明确 unavailable。

## 范围

- Application 定义结构化命令请求、稳定结果分类、取消信号和 Command Port。
- macOS Adapter 校验路径/参数/环境，清空继承环境，并行有界读取 stdout/stderr。
- 超时、取消或输出超限时停止并确认完整进程组。
- 覆盖字面参数、正常/非零退出、启动失败、超时、取消、双流超限与后代清理。

不新增 Gateway/CLI/MCP、协议版本、SQLite、Shell、风险确认 UI 或任务事件。本增量不允许任何 Agent 绕过 Gateway 直接调用 Adapter。
