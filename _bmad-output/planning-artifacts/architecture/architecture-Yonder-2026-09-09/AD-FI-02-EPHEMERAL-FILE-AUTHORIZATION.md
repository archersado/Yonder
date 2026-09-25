# AD-FI-02 临时文件授权引用

状态：Accepted（Application 核心；产品选择器与 Gateway 接线仍待后续 Change）  
日期：2026-09-25  
Architecture Impact：architecture-change（新增任务绑定的内存授权状态；不新增持久化、协议或第二文件执行栈）

## 问题

FI-S1 与 DO-S2 已具备受控文件 Runtime，但 Agent Gateway 不能接收请求自报的绝对路径、授权根或 `confirmed=true`。任务历史中的路径或授权快照也不是当前通行证。产品需要一个由可信本机用户入口签发、可撤销且不会跨重启复活的文件授权引用。

## 决策

Application 持有进程内 `FileAuthorizationRegistry`。可信 LocalUser 入口针对当前任务签发授权，引用同时绑定 `grant_id`、`task_id`、归属 `agent_id`、用途、精确文件位置/身份和过期时间。Agent 只能在身份、任务、用途和有效期全部匹配时解析引用；请求字段不能构造 LocalUser 或改写绑定。

既有文件授权在签发时经 File Port 读取，保存规范路径、文件身份与 SHA-256，不保存正文。新建目标在签发时经 File Port 规范化既有父目录、验证授权根和目标不存在，保存规范目标路径及父目录身份。真正执行仍由 File Port 在提交时重新检查身份、哈希、父目录、目标竞争和宿主锁；授权引用不替代 TOCTOU 防护。

读取授权在有效期内可重复解析；`create-new`、`replace` 与 `trash` 授权在首次成功解析时原子消费，即使后续结果 unknown 也不得重放。引用仅驻留内存，进程退出、显式撤销、Agent 撤权或到期后失效。首版每实例最多 256 个授权、最长 15 分钟；满额失败关闭，不自动淘汰仍有效授权。

## 边界

- Registry 属于 Application；File Adapter 只解析文件身份与位置，不访问任务库或 Agent 会话。
- 本决定不新增 SQLite 表、Outbox 载荷、协议字段、文件选择器或 Gateway 方法。
- 后续产品入口必须由可信桌面组合根从原生选择器结果签发，并只向对应 Agent 会话返回 `grant_id`；日志、任务事件和 UI 错误不得记录完整路径或正文。
- 覆盖、回收站、Shell、提权、安装、支付和发送仍遵守各自确认门禁；文件授权引用不能充当命令风险确认。
- Windows 文件 Runtime 仍保持 unavailable；用户决定 Windows 验证延期不改变该边界。

## 验证

Application 合约覆盖 LocalUser-only 签发、任务/Agent/用途隔离、到期与撤销、读取复用、写授权一次消费和容量失败。macOS File Adapter 覆盖不存在目标的父目录规范化、授权根逃逸、既有目标竞争及父目录身份。产品选择器、Gateway、撤权广播与 Windows 分别由后续 Change 取得原生证据。
