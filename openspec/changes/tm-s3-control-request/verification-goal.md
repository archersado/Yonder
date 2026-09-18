# TM-S3 外部控制请求独立 Verification Goal

日期：2026-09-16。关联 Story TM-S3、Accepted AD-TM-08 与 `openspec/changes/tm-s3-control-request/`。Windows 按用户要求暂缓。

## 结果

PASS（外部 pending 控制子范围）。工作区 36 项 Rust 测试全部通过。Adapter 集成覆盖 pending 控制、任务序号、`running→running` 事件和 Outbox 同事务；相同控制重投幂等，不同控制与旧执行身份拒绝；pending 后禁止继续派发；停止确认必须匹配 pending 控制，并在同一事务更新为 stopped；unknown 和 Outbox 故障均不释放 Permit。

协议 1.6、Rust 派生 JSON Schema/TypeScript 与 Yonder CLI MCP 工具同步完成。真实 bundle 内 CLI 经 stdio→UDS→正式 Gateway 验证，8 个工具可见且 `task_control` 已发布；创建、读回和保留数据取消仍通过。[MCP 证据](../../../apps/desktop/evidence/control-request-mcp-20260916/result.json)。

任务卡片的运行中“接管/取消”已调用 `task.control`，pending 时显示“正在停止”，不显示“已接管”；面板刷新或隐藏再显示仍禁用冲突操作。ego-browser 夹具验证请求能力、参数、刷新保持、按钮反馈、分页与详情共存，并保留[截图证据](../../../apps/desktop/evidence/control-request-browser-20260916.png)。正式 Yonda 已单实例运行；正式未加密库从 schema9 升级 schema10，迁移前备份存在，原 14 个任务完整保留，验证后新增的 CLI 任务按契约取消并保留，未伪造 running/attempt/control 数据。

## 完成边界

本 Goal 只证明停止请求已持久化并冻结后续派发。接管后的工作定位、焦点核验、人工行为 Recording、交回 Agent 与 Windows 证据仍未完成；TM-S3 保持 implementing，不 Archive。
