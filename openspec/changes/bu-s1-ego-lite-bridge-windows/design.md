# 设计

Desktop 组合根只信任固定 `Program Files/Citro Labs/ego lite/Application` 安装根中的数字版本目录。Windows 优先解析固定当前用户 `%USERPROFILE%/.local/share/ego/active_version_dir/ego-browser.exe`；仅当 canonical 真实路径仍位于上述安装根、父目录是合法数字版本且文件名精确匹配时采用。active 指针缺失或无效时，才从固定安装根选择可解析版本中的最新项回退；Agent、网页、任务参数和 PATH 不得提供路径。Adapter 使用固定 Worker 和当前用户私有的短生命周期 UTF-8 `.mjs`/结果文件。macOS 继续以结构化 `program + args` 直接启动 `ego-browser nodejs`；Windows 通过系统固定 `powershell.exe` 和固定命令模板调用可信 Runtime，仅把 Adapter 生成的 ASCII `await import('file:///...')` 入口作为数据参数传入，并在子进程结束后删除脚本。任务数据不进入 PowerShell 命令文本。

Windows 与 macOS 共用 `BrowserUsePort`、操作枚举、响应身份校验、64 KiB 结果上限和单并发准入。bootstrap 不可达、超时、退出失败和响应不可信分别收敛为既有 unknown 分类，不自动重试可能有副作用的调用。无第二浏览器、无 CUA 回退、无额外任务状态源。

验证使用真实 ego-lite Task Space 完成 create → observe → handOff → takeOver → finish，并覆盖 Runtime 缺失和 active 指针不得越出固定安装根。当前机器同时安装 `0.6.1.13` 与未激活的更高版本时，必须证明 Desktop 选择 active 的 `0.6.1.13`，而非按目录最大值误选。若受执行环境 sandbox 阻止 bootstrap，必须记录为环境阻塞，不能伪报产品通过。
