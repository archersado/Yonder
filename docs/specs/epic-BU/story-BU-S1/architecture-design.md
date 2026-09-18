# BU-S1 架构设计

## 边界与依赖

依据 Accepted AD-E0-03，macOS Adapter 通过可信绝对路径调用 `ego-browser nodejs` 的嵌入式 SDK；固定 Worker 与经 JSON 双重转义的数据字面量写入权限0600的短生命周期文件作为标准输入，启动后立即unlink，不经过 Shell；不移植第二浏览器，不以 CUA 静默回退；BUA MVP 复用 `Resource::Browser` 单并发。

## 状态与契约

SQLite 保持任务当前事实源；UI 仅持展示快照。Application 定义 Browser Bridge 契约并复用既有执行尝试、pending control 与结果提交，Adapter 只映射 ego-lite Task Space。外部引用持久化待 TM-S1 定案，本增量不另建状态系统或 Agent wire 方法。

## 失败与验证

验证真实 create → observe → handOff → takeOver → finish，且检查引用连续、所有权变化、结束清理及依赖不可用。Windows Runtime 延期，不能宣称双平台 BUA 可用。
失败不得隐式重试未知副作用。验证覆盖正常、拒绝和中断路径；沿用架构依赖与关联检查。

## 架构影响

本次影响 Application Browser Bridge Port 与 macOS Adapter；不改变外部协议、SQLite schema 或依赖方向。
