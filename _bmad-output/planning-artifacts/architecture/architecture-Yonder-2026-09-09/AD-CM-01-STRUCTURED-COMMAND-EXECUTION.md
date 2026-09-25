# AD-CM-01 结构化命令执行与进程树停止

状态：Accepted（macOS-only Runtime，Windows 与 Agent Gateway 后补）
日期：2026-09-17  
关联：CM-S1

## 决策问题

Yonder需要执行`program + args + cwd + env`，限制输出并在超时或取消时停止完整进程树。Shell只能由独立、显式且经用户确认的入口请求，不能由结构化命令参数隐式拼接。

## 决策

产品采用 Rust 标准库 `std::process::Command`。本次只接受 macOS Runtime：子进程建立独立进程组；超时、取消或任一输出超过预算时先 TERM、再 KILL 整个进程组，并等待父进程退出及进程组消失。Windows 在 Job Object 统一样本通过前必须返回 unavailable，不编译或注册未经验证的执行路线。

stdout/stderr 由独立有界读取器并行排空，各自最多保留 64 KiB；任一路超过预算即停止进程组并返回 `output-limit-exceeded`，不把截断当普通成功。程序和 cwd 必须是调用前已存在且规范化后的绝对路径；程序必须是普通文件、cwd 必须是目录。首批环境完全清空，只注入显式提供的有界键值，不继承 Agent 或宿主完整环境。stdin 固定为空。

Application 定义 Command Port 和结构化请求/结果；Adapter 只执行已经通过上层授权和确认的请求，不读取任务库、不决定风险、不写事件或日志。结果区分 `exited / timed-out / cancelled / output-limit-exceeded / unknown`；启动失败作为稳定错误返回。非零退出仍是已知 `exited`，不是传输失败。停止后无法确认整个进程组消失必须返回 `unknown`，不得自动重试。

结构化入口不得接受命令字符串、重定向、管道或命令替换。Shell是后续独立能力，需要可信本地确认事实，Agent不能自报确认。提权、安装、删除、支付和发送同样必须先取得用户确认。由于通用命令的风险确认协议尚未定案，本决定只授权内部 Runtime/Port，不开放 Agent Gateway、CLI/MCP 或 Jev Command 候选。

## Spike与平台门禁

macOS 统一样本已经验证参数不经 Shell 解释、父子进程组停止和 64 KiB 输出限制。用户明确 Windows 验证可以延期，因此本 ADR 只接受 macOS Runtime 子范围；这不是对双平台可用性的推断。Windows Job Object 与产品 Gateway 仍分别受平台证据和可信风险确认协议门禁，CM-S1 不因此 Done/Archive。

## 限额

- `program`、`cwd`：UTF-8 绝对路径，单项最多 4096 字节；规范化后必须保持对应文件类型。
- `args`：最多 128 项，单项最多 4096 字节，总计最多 32 KiB；按字面参数传入。
- `env`：最多 64 项，键 1..128 ASCII 字节且只允许字母、数字、下划线，值单项最多 4096 字节，总计最多 16 KiB；键不得包含 `=`，键值不得包含 NUL。
- `timeout`：100 ms..300 s；输出 stdout/stderr 各 64 KiB。
- 停止：TERM 等待 250 ms，仍存活则 KILL；最多再等待 1 s。无法确认进程组消失返回 unknown。
