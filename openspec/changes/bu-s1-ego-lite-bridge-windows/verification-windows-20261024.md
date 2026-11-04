# 独立 Verification Goal：Windows ego-lite Bridge

日期：2026-10-24
结论：BLOCKED（实现已接通，真实生命周期未通过）

- 已确认安装版本 `0.6.0.22`，固定路径为 `C:\Program Files\Citro Labs\ego lite\Application\0.6.0.22\ego-browser.exe`；`ego.exe` 可启动主窗口。
- Desktop 可信组合根能够从固定安装根选择最高可解析版本；调用方不能注入可执行路径。
- Adapter 已在 Windows 使用私有短生命周期脚本/结果文件，并复用既有 `BrowserUsePort`、身份校验、结果上限和 unknown 分类。
- Windows `ego_lite_bridge_check` 已编译并启动真实 `ego-browser.exe nodejs`，但当前 Agent 执行沙箱无法连接 ego_cli bootstrap，create 在 30 秒后收敛为 `Unknown(TimedOut)`。
- 按用户补充要求，另在 Windows PowerShell 5.1 中分别以非交互和真实 PTY 交互会话执行 `ego-browser.exe nodejs -e "console.log('ego-browser ready')"`；两种方式均返回 `Failed to connect to ego_cli bootstrap`，退出码为 5。因此失败与 stdin 调用方式无关，也不是 Rust 子进程特有问题。
- `ego-browser skill` 在 PowerShell 中成功并确认版本为 v2.1.0。随后按其 Windows PowerShell 5.1 指南，把 UTF-8 JavaScript 写入 `.mjs`，只用 ASCII 单行 `nodejs -e "await import('file:///...')"` 加载；仍返回同一 bootstrap 错误和退出码 5，已排除 PowerShell 参数转义/编码方式。
- 当前 PowerShell 的 `Get-Command ego-browser` 仍找不到命令；本次使用安装目录内 `0.6.0.22\\ego-browser.exe` 的绝对路径完成验证。
- 未执行 CUA 回退，也未把安装存在或 GUI 可启动误报为 Task Space 生命周期通过。

因此 create → observe → handOff → takeOver → finish 与独立 Verification Goal 继续保持未完成。需在用户可正常调用 ego-lite SDK 的交互式 PowerShell/Full Access 环境复跑同一示例后才能通过。
