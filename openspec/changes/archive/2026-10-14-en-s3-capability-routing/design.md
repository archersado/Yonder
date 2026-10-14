# 设计：声明式能力快照与穷举请求门禁

## 能力快照

握手先从协议版本、平台、Adapter 支持位和运行时可用性计算一次 `NegotiatedCapabilities`。所有 `can_*`、响应版本和能力列表只读取该快照；能力表条目按既有响应顺序声明名称、最低版本、展示版本、可用性与 reason。

`TaskRead` 的可见字段版本与 hello 顶层协商版本虽然历史上存在差异，仍分别由同一份有序 feature-level 表取最大已启用版本，保留 1.48 截图坐标只影响顶层/计划能力而不提升 `TaskRead` 的既有行为。

## 请求门禁

对 `Request` 全变体建立穷举的 `required_capability` 映射。Hello 与纯读取仍遵循既有协商/降级规则；写入和执行请求先经统一映射检查，再进入现有业务分支。任何新增 Request 变体都必须在编译期补充映射。

## 等价护栏

- golden 覆盖 minor 0 至当前协议 minor，并自动随 `PROTOCOL_VERSION` 扩大。
- 保留 capability 顺序、版本、availability、reason、错误码和消息。
- 不改 protocol serde、SQLite schema/user_version、Cargo 依赖或 UI。
