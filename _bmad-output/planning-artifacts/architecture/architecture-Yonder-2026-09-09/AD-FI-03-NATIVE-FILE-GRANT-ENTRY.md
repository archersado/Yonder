# AD-FI-03 原生文件授权入口与 Gateway 可见性

状态：Accepted（macOS 产品入口；Windows 延后）  
日期：2026-09-25  
Architecture Impact：architecture-change（协议 1.27、桌面组合根持有文件授权 Registry；不新增持久化或文件执行方法）

## 问题

Accepted AD-FI-02 已建立任务绑定的临时文件授权核心，但产品仍没有可信选择入口，归属 Agent 也无法通过既有 Gateway 得知用户签发的引用。把路径、授权根或确认布尔值放入 Agent 请求会破坏授权边界；把临时授权写入 SQLite、事件或 Outbox 又会使它跨重启复活并泄露本机位置。

## 决策

macOS Task Space 增加任务详情内的读取、新建、替换和移入回收站授权入口。Tauri 命令只接受 `task_id + purpose`，文件路径仅来自命令内部启动的系统原生选择器；授权根固定为所选目标的父目录，不能由 WebView 或 Agent 提供。替换和回收站在选择后必须由原生确认对话框再次确认，取消选择或确认不签发引用。

桌面 `TaskHost` 组合 `ControlledFileAdapter` 与 `FileAuthorizationRegistry`，从 SQLite 重新读取任务及归属 Agent 后签发最长 15 分钟的内存引用。`grant_id` 使用宿主生成的 UUID；它只是引用而非单独的 bearer credential，解析仍同时校验认证 Agent、任务、用途、期限和新鲜文件身份。Task Space 可列出并撤销当前任务引用，但展示内容只含用途和到期时间，不展示规范路径、授权根、文件身份或哈希。

Rust 唯一协议源升至 1.27，新增只读能力 `file.grant.read` 与请求 `task.file.grants`。只有完成 1.27 握手、当前已认证且仍登记可用的归属 Agent，才可读取该任务的安全摘要：`grant_id`、`purpose`、`expires_at_ms`。请求不接受路径、用途或授权根；读取不消费一次性引用。MCP 只提供同名只读工具。协议 1.26 及更早版本保持原行为且不出现该能力。

Agent 被禁用或撤权时，桌面在持久状态更新前先撤销该 Agent 的全部内存文件引用并断开会话，失败关闭。任务进入终态后 Registry 的解析与列出继续拒绝；残留项仅可在到期清理时释放容量，不形成执行权限。授权签发、撤销和查询均不写任务事件或 Outbox。

## 边界

- 本决定不开放读取、写入、覆盖、删除或文档处理的 Agent 执行方法；实际副作用由 FI-S1/DO-S2 后续执行 Change 使用引用解析后另行接线。
- UI 不接收或回传路径；日志、错误、任务数据、事件和 Outbox 不记录路径、文件正文、身份或哈希。
- 原生选择器取消是正常取消，不显示为错误；目标已存在、链接逃逸、宿主锁、容量不足和任务失效提供稳定分类，但不回显路径。
- Windows Runtime 与选择器保持 unavailable；用户决定延后 Windows 验证不构成通过。

## 验证

协议合约覆盖 1.26/1.27 隔离、能力匹配、非归属 Agent、禁用 Agent、终态任务、过期清理以及响应无路径。Application 覆盖安全摘要不消费一次性引用。桌面集成覆盖原生入口只能由 `task-space` 调用、父目录由所选目标派生、替换/回收站确认、撤销和 Agent 撤权清理。前端回归覆盖四种入口、取消、处理中、成功、错误和撤销反馈。macOS 保存结构化原生证据；Windows 明确延期。
