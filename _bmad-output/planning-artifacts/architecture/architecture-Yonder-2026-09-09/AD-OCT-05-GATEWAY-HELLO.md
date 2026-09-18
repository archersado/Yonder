# AD-OCT-05 Gateway 握手与连接内查询门禁

状态：Accepted（Application 连接内逻辑；非实际认证、IPC 或宿主接线）。关联 OCT-S1、AD-OCT-02/04。Architecture Impact：architecture-change。

增加 gateway.hello，Rust 唯一定义请求与响应。请求仍携带 id、agent_id、capability=task.read、deadline，另带 protocol_version={major,minor}，两字段均为 u16。当前候选协议为 1.0，不是 JSON-RPC 的 2.0 字段，也不代表已经发布稳定协议。

Application GatewaySession 由完成外部认证的组合根提供固定 AuthContext 和宿主平台（Windows/macOS），每连接独立持有握手标记，不持有任务快照、数据库或身份凭据。连接关闭即丢弃，重连必须重新认证并握手。

会话先复用既有请求校验与身份绑定；未握手的 task.get/list/events 返回 -32002，不访问存储。hello 主版本非 1 时返回 -32010 并清除握手标记；主版本匹配时协商 minor=min(客户端 minor,0)，成功后才放行查询。重复兼容 hello 幂等，不改变身份。格式/身份/过期错误保持已有校验语义。

hello 返回 kind=hello、协商版本、平台、能力清单。能力项包含 name/version/availability/reason。当前只声明 task.read 1.0 available，不宣称支持尚未接线的 CUA/BUA/文件/文档或写操作；握手不读取任务库、不启动 Worker 或外部应用。未来能力变化通知另行定义，不在本批假实现。

握手不是认证，协议版本和 agent_id 不能充当凭据。本批没有真实连接、Socket、HTTP 服务或云端接线。生产传输入口必须使用 GatewaySession；既有 query::handle 只保留为可信进程内用例入口，不能直接对外开放。桌面与 IPC 实施仍遵循 E0 门禁。
