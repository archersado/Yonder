# Design：EX-S2 可配置 Jev 模型

`JevConfig` 增加 `model` 字段，采用 1..128 字节的受限 ASCII 标识。Serde 缺省值为 `jev-latest`，使已有单行 JSON 无需数据库结构迁移即可读取；保存后仍由 SQLite Adapter 原子替换同一行。

Application 只调用 `JevConfig::validate` 统一检查模式、端点、模型、预算和能力。`remote` 继续限于 TypeSafe HTTPS；`local` 接受无凭据、查询串和 fragment 的 HTTP/HTTPS base URL。Adapter 在候选选择和步骤核验的 Worker 请求中都传入配置模型；Worker 仅消费该字段，不维护默认模型。

独立设置窗口增加模型输入框并进行与 Application 一致的最小前端校验。保存只表示结构配置有效，不探测或加载模型。正式验证使用本地 `/v1/models` 已发布模型，经 Yonder MCP/Gateway 的真实多 AX 候选任务证明 Jev 选择；失败仍按既有 handback 处理。

`JevConfig` 同时增加 `confidence_field` 与 `confidence_threshold`。字段名为空或满足 `[A-Za-z_][A-Za-z0-9_]{0,63}`，阈值必须为有限的 `0..1`。Serde 兼容旧 `confidence_metric`；历史 JSON 缺字段、新配置留空和 `Default` 都使用 `confidence + 0.75`。Adapter 把 Application 归一化后的字段名传给一次性 Worker，Worker 严格读取选定字段并输出统一 `confidence`；字段缺失、非数值、非有限或越界均作为无效响应安全交回。Application 不再使用编译期固定阈值，而从同一配置快照读取阈值。
