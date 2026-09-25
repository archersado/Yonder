# EX-S2 macOS-only Jev 决策接线

Story：EX-S2；依据 2026-09-24 用户变更与 Accepted（macOS-only）AD-EX-02。

实现 macOS-only 的远端 Jev 有界候选决策接线：Application 校验配置、能力范围与 2..10 个候选，并应用 0.75 置信阈值与交回策略；独立设置面板可一次性写入 macOS Keychain，macOS Adapter 仅从该凭据入口读取 API Key，通过固定 `@typesafe-ai/sdk@0.6.0` 调用 `https://api.typesafe.ai/v1/systemone`。本 Change 不新增任务状态、事件、Outbox、Gateway 或执行栈；Windows 明确后补，不编译、不注册、不验证。

Architecture Impact：macos-only-runtime。API Key、证书和刷新令牌不进入 Jev 配置、SQLite、任务数据、日志或 UI。
