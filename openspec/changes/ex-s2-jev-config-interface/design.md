# 设计

Application 定义 `JevConfig`，包含：

- `enabled`
- `service_mode`
- `endpoint`
- `step_limit`
- `time_limit_ms`
- `token_limit`
- `capabilities`

Application 负责字段校验；SQLite Adapter 保存为单行 `jev_config` JSON。Task Space 增加一个本机设置区，只通过 Tauri 命令读取和保存，不直接访问 Adapter 或文件。API Key、证书、刷新令牌、完整端点查询串和模型请求正文不入配置、不入日志。
