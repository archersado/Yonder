# FI-S1 架构设计

## 边界与依赖

可信 File Adapter 规范身份，处理软链接和硬链接；同文件写串行，后续任务接线依赖 TM-S2/TM-S7。Accepted AD-FI-01 当前只授权 macOS Runtime，Windows 返回 unavailable；Accepted AD-FI-02 已定案授权引用核心，但原生选择器、Gateway 与可信确认产品接线完成前仍不开放文件执行入口。

Accepted AD-FI-02 新增 Application 内存授权 Registry：LocalUser 为当前非终态任务签发绑定归属 Agent、用途、文件身份/目标父目录和有效期的引用。既有文件签发复用 File Port 快照但丢弃正文；新目标通过 File Port `inspect_create_target` 规范化父目录并确认目标不存在。Registry 不访问文件系统，Adapter 不访问任务或授权状态。

Accepted AD-FI-03 将该核心接入桌面组合根：`TaskHost` 持有 Registry 与 `ControlledFileAdapter`，Tauri 只从 `task-space` 窗口接收任务和用途，路径来自命令内部原生选择器，父目录由 Rust 派生。替换/回收站经原生二次确认。Agent 禁用或撤权先按 owner 清空引用，再更新登记状态并断开会话。

## 状态与契约

MVP 按 Accepted AD-ST-01 使用未加密 SQLite 保持任务当前事实源，SQLCipher 延期至 ST-S2；UI 仅持展示快照。协议 1.27 从 Rust 派生 `file.grant.read`、`task.file.grants` 与安全摘要，只返回引用、用途和到期时间。Gateway 先验证握手、当前登记状态、会话 Agent 与任务归属，再查询内存 Registry；协议读取不消费引用且不写 SQLite、事件或 Outbox。

## 失败与验证

Application 定义 `FilePort`、平台无关 `FileIdentity`、有界读取快照、`create-new/replace` 写请求、校验回调和回收站请求。macOS Adapter 使用 `st_dev + st_ino`；新输出规范化既有父目录并以父身份+文件名占用临时写身份。临时文件必须与目标同目录。Yonder 写租约与 Office/WPS 宿主锁均须满足，不能以其中一个替代另一个。

替换必须匹配预期身份与 SHA-256；写入临时文件并同步后，由调用方对暂存字节执行格式校验，再原子替换并同步父目录。提交后同步失败为 unknown。回收站只经 `AuthContext::LocalUser` 用例进入系统 `NSFileManager`，没有永久删除或 Agent 确认布尔值。
失败不得隐式重试未知副作用。验证覆盖正常、拒绝和中断路径；沿用架构依赖与关联检查。

## 架构影响

Runtime 增量新增 Application File Port 和 macOS Adapter，依赖方向保持 adapters→application；Windows 编译路径稳定返回 unavailable。

临时授权核心只新增 Application 内存状态及 File Port 新建目标预检，不修改 SQLite、协议或 Gateway。写用途引用解析时原子消费；实际文件操作继续使用身份/hash/父目录重新校验，授权不能代替并发保护。

原生入口增量按 AD-FI-03 将协议升至 1.27，并由 desktop→adapters/application 的既有组合根持有 Adapter 与 Registry；不改变 SQLite schema、任务状态所有者或依赖方向，不新增第二条 Agent 通道。文件执行仍未开放。
