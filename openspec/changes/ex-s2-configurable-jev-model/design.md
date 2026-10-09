# Design：EX-S2 可配置 Jev 模型

`JevConfig` 增加 `model` 字段，采用 1..128 字节的受限 ASCII 标识。Serde 缺省值为 `jev-latest`，使已有单行 JSON 无需数据库结构迁移即可读取；保存后仍由 SQLite Adapter 原子替换同一行。

Application 只调用 `JevConfig::validate` 统一检查模式、端点、模型、预算和能力。`remote` 继续限于 TypeSafe HTTPS；`local` 接受无凭据、查询串和 fragment 的 HTTP/HTTPS base URL。Adapter 在候选选择和步骤核验的 Worker 请求中都传入配置模型；Worker 仅消费该字段，不维护默认模型。

独立设置窗口增加模型输入框并进行与 Application 一致的最小前端校验。保存只表示结构配置有效，不探测或加载模型。正式验证使用本地 `/v1/models` 已发布模型，经 Yonder MCP/Gateway 的真实多 AX 候选任务证明 Jev 选择；失败仍按既有 handback 处理。
