# AD-E0-03 ego-lite BUA

- 状态：Accepted（macOS/Windows Bridge）
- Story：E0-S3
- OpenSpec：`e0-defer-ego-lite-bua`
- 日期：2026-09-10

## 原决定

Windows 首版暂不交付 BUA。Yonder 保留 BUA Bridge 端口，但不实现 ego-lite Runtime、不复制 Browser Task Space，也不引入替代浏览器自动化引擎。

## 2026-09-16 重启决定

macOS 已重新进入实际交付与原生验证范围，满足本 AD 的重启条件。BU-S1 在 macOS 直接调用已安装 ego-lite 的 `ego-browser nodejs` 嵌入式 SDK 入口，复用其 Task Space 的 create/reuse、observe、handOff、takeOver 与 finish；Yonder 只实现 Bridge，不嵌入 Runtime、不复制页面/标签/所有权状态机、不用 CUA 回退。CLI 路径只由可信组合根提供；固定 Worker 与经 JSON 双重转义的数据字面量组成 SDK 程序，写入权限0600的短生命周期文件作为标准输入，启动后立即unlink，名称不能逃出字符串，不能拼接为 Shell。

Bridge 每次返回稳定的 `external_task_ref`、所有权和托管页数量；失败收敛为 dependency unavailable、timeout、invalid response 或 runtime failed，不自动重发可能有副作用的调用。Task 执行仍由同一 Application 准入、尝试身份、步骤边界控制和结果提交负责；本增量不另建任务状态源，也不提前实现尚未定案的外部引用持久化。

Windows 仍明确返回能力不可用，不宣称双平台 BUA。待 ego-lite 提供 Windows Runtime 后补该平台的独立 Story 验证，不以 macOS 证据代替。

## 2026-10-24 Windows 重启决定

用户确认 Windows 已安装且可从 PowerShell 启动 ego lite；本机安装包含 `ego-browser nodejs` Runtime，满足原重启条件。Windows 继续复用同一 Application `BrowserUsePort`、固定 Worker 与 Task Space 所有权模型，只增加可信安装路径发现、Windows 私有临时文件和固定 PowerShell 启动器，不复制 Browser Task Space、不引入 Playwright/Selenium、不回退 CUA。

可信组合根只在固定厂商安装根 `Program Files/Citro Labs/ego lite/Application` 中选择已存在的版本化 `ego-browser.exe`，不得接受 Agent 或网页提供的可执行路径。由于 Windows PowerShell 5.1 会破坏复杂原生参数，Adapter 必须把固定 Worker 与经 JSON 双重转义的数据写入当前用户私有的 UTF-8 `.mjs`，再由系统固定 `powershell.exe` 以固定命令模板调用可信 Runtime，并仅传入 Adapter 生成的 ASCII `await import('file:///...')` 入口；Agent、网页和任务输入均不得形成 Shell 命令或可执行路径。该受限 PowerShell 启动器是本 Bridge 的显式平台例外，macOS 仍使用结构化直接启动。找不到 Runtime、bootstrap 不可达、响应不可信或超时时仍返回明确 unavailable/unknown，副作用不自动重试。Windows create/reuse、observe、handOff、takeOver、finish 和 Yonder 状态映射必须独立验证后才可称为通过。

## 2026-11-01 Windows 激活版本修订

ego-lite 可以同时保留已安装但未激活的新版本；简单选择 `Application` 下最大版本会绕过当前用户已经完成 bootstrap/onboarding 的 active 版本。Windows 组合根须优先读取固定的当前用户 ego active-version 指针 `%USERPROFILE%/.local/share/ego/active_version_dir/ego-browser.exe`，但使用前必须解析其真实路径，并验证最终可执行文件仍位于上述固定 Citro Labs 安装根的合法数字版本目录内；指针缺失、解析失败、越出安装根或文件名不符时不得采用。只有 active 指针不可用时，才按既有规则从固定安装根选择最高合法版本作为兼容回退。该修订不允许 Agent、网页、任务参数或任意 PATH 注入 Runtime，不改变 PowerShell 固定模板、Worker、协议、状态所有者或失败不重试语义。

## 原因

ego-lite 官方当前仅支持 macOS，Windows 尚在 roadmap；本项目已将 macOS 验证移出当前 E0 范围，因此不存在可验证、可发布的目标平台 Runtime。

## 重启条件

满足任一条件后重新建立 Story 与 OpenSpec：ego-lite 提供 Windows Runtime；或 macOS 重新进入交付范围。届时必须验证 CLI 部署、Task Space create/reuse/switch/complete、handOff/takeOver、事件读取和 Yonder 状态映射。
