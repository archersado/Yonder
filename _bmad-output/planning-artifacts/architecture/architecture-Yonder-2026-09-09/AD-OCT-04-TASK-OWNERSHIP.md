# AD-OCT-04 任务归属与可信查询身份

状态：Accepted（核心归属/授权边界；不代表 OS、云端认证或 Gateway 已接线）。关联 OCT-S1、AD-OCT-01/02/03。Architecture Impact：architecture-change。

## 决定

- Application 定义不可从 JSON 反序列化的 AuthContext：Agent 身份只访问该 agent_id 的任务；LocalUser 身份用于受信任的本机用户界面，可查看该用户数据库全部任务。身份由未来已完成认证的 Gateway/组合根提供，不从请求的 agent_id、参数或 capability 推导。
- 查询分派必须显式接收 AuthContext；请求 agent_id 必须与绑定身份的客户端标识一致，否则返回 -32003。已有 task.read 契约仍限制为只读。本批不建立凭据、认证会话或传输入口。
- 创建任务时从 AuthContext 写入不可变 owner_agent_id；任务迁移不得更换归属。内部存储 Port 接受可信归属，禁止将它直接映射为外部创建请求参数。
- Agent 的 task.list 在 SQL 中先按 owner_agent_id 限定，再筛选、排序和分页，游标不泄露其他 Agent 的任务。task.get/task.events 对他人任务和不存在任务都返回 -32004，避免通过不同错误判断任务存在。三个查询共用同一身份约束。
- 本机用户的任务空间仍能看到全部任务；允许看到全部任务的身份不能由 JSON 请求声明。返回快照包含 owner_agent_id，Rust 生成 Schema/TypeScript。

## 数据格式与旧库保护

新建任务库 schema_version=2，在 tasks 增加必需 owner_agent_id，并建立 (owner_agent_id,id) 索引。状态、事件、Outbox 同事务语义不变；无新的状态事实源。

schema_version=1 没有可信归属，不能把旧任务自动分配给当前调用者。此次 open 只初始化空库或接受 v2；v1 和未知版本拒绝，保留文件。不得自动重建、清空或修改现有任务库。本批仅使用合成新库测试，不打开用户实际任务库。依 AD-OCT-01，旧库迁移必须另建 Change，先备份并确定旧任务归属，再原子迁移；本批不实现迁移，不能将这次库格式升级当作可直接部署的兼容更新。

## 验证门槛

两个 Agent 和一个 LocalUser 使用同一真实 SQLCipher 库：验证列表隔离、逐任务读取及事件拒绝、伪造 agent_id 拒绝、本机全量总览、归属跨状态迁移/重开保持不变、跨页不混入他人任务，以及 v1 拒绝且原数据保持。外部认证和双平台 E2E 仍单独验收。
