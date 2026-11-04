# BU-S1 架构设计

## 边界与依赖

依据 Accepted AD-E0-03，macOS/Windows Adapter 通过可信绝对路径调用 `ego-browser nodejs` 的嵌入式 SDK。固定 Worker 与经 JSON 双重转义的数据字面量写入当前用户私有短生命周期 UTF-8 `.mjs`；macOS 继续结构化直接启动并以该文件作为标准输入，Windows 则由固定 `powershell.exe` 命令模板执行只含 Adapter 生成文件 URL 的 ASCII `await import('file:///...')` 入口，子进程结束后删除脚本。Agent、网页和任务输入不得形成 Shell 命令或可执行路径；不移植第二浏览器，不以 CUA 静默回退；BUA MVP 复用 `Resource::Browser` 单并发。Windows 优先解析固定当前用户 active-version 指针，且仅当其真实目标仍位于固定 Citro Labs 安装根的合法数字版本目录时采用；否则从该安装根选择最高合法版本回退。调用方和 PATH 均不得传入或覆盖 Runtime。

## 状态与契约

SQLite 保持任务当前事实源；UI 仅持展示快照。Application 定义 Browser Bridge 契约并复用既有执行尝试、pending control 与结果提交，Adapter 只映射 ego-lite Task Space。外部引用持久化待 TM-S1 定案，本增量不另建状态系统或 Agent wire 方法。

## 失败与验证

分别在 macOS 与 Windows 验证真实 create → observe → handOff → takeOver → finish，且检查引用连续、所有权变化、结束清理及依赖不可用。Windows 还须覆盖“较高版本已安装但 active 指针仍指向较低已激活版本”，证明组合根使用 active 目标且拒绝越出固定安装根的指针。平台 Runtime 不可达时失败关闭，不自动重试。
失败不得隐式重试未知副作用。验证覆盖正常、拒绝和中断路径；沿用架构依赖与关联检查。

## 架构影响

本次影响 Application Browser Bridge Port 与 macOS/Windows Adapter；不改变外部协议、SQLite schema 或依赖方向。
