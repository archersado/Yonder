# 设计

`TaskHost` 是唯一组合点，持有 `ControlledFileAdapter`、`FileAuthorizationRegistry` 与 SQLite TaskStore。签发前从 TaskStore 重新读取 task；选择器仅传入 Rust `PathBuf`，TaskHost 将其父目录作为授权根。Registry 继续处理文件快照、身份、有效期、容量和一次消费。UI 命令及其成功响应都不包含路径。

协议以 Rust 为唯一来源，版本 1.27 新增 `Capability::FileGrantRead`、`Request::FileGrants` 与 `QueryResult::FileGrants`。Gateway 只在带 Registry 的 runtime 入口处理该请求：先完成握手、检查 1.27 能力和 Agent 登记状态，再按现有任务读取校验 owner，最后取得 Registry 的安全摘要。摘要不调用 resolve，因此不会消费写用途引用；返回只含 ID、用途、到期时间。

直接 `GatewaySession::handle` 与历史 runtime 入口都保持不提供该请求。TaskHost 使用扩展 runtime 入口传入 Registry，避免测试夹具和不持有本机文件授权的组合根意外开放能力。MCP 将 `task_file_grants` 映射为同一个有类型请求。

替换与回收站由 rfd 原生消息框确认；取消选择或确认返回 `None`。Task Space 根据安全摘要渲染用途和到期时间及撤销动作。撤销、过期、任务终态、非 owner、协议版本不足、Agent 禁用均拒绝或清空，无路径错误回显。
