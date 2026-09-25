# 设计

Application 定义 `JevDecisionPort` 与有界候选模型。每次请求绑定 `task_id`、`step_id` 与能力，候选数量为 2..10、ID 唯一且必须包含唯一 `handback`。Jev 只能返回已提交候选 ID 与置信度；Application 统一执行 0.75 置信阈值，未知候选、低置信或不可派发候选均交回，不自动重试。

macOS Adapter 固定校验 `@typesafe-ai/sdk@0.6.0`，从 Keychain 读取服务名 `Yonder`、账号名 `jev` 的 API Key，再以 Worker 调用 `https://api.typesafe.ai/v1/systemone` 和 `jev-latest`。单次 SDK 与 Worker 超时统一为 3000ms，重试为 0；超时立即返回并交回。请求不写模型正文、截图或 Key；失败映射为凭据缺失、依赖缺失、超时、远端错误或无效响应。Windows 路径不编译、不注册、不验证。

本 Change 只建立决策接线路径，不接入现有任务执行，不修改任务状态、事件或 Outbox。后续片段执行用例必须在配置、租约、确认、文件锁和每步 Observe 通过后调用本 Port。
