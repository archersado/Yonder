# CU-S5 视觉与交互设计

本 Story 无产品 UI。调用方是隔离 Spike Supervisor；可观察状态为 `source_verified`、`identity_unproven`、`input_observer_unready`、`environment_unavailable`、`gate_passed`、`rejected`。每次探针返回状态和无敏感信息的证据文件路径；拒绝原因用封闭错误码，不输出目标窗口内容或输入正文。

当目标对象连续性或输入来源不可证时，Supervisor 不显示“准备就绪”，Worker 对动作返回 `unsupported/not_delivered`；用户接管仍按现有 Application 语义冻结新派发。在途副作用不明时显示 `unknown` 且不自动重试。缺 macOS 原生环境时明确显示 `environment_unavailable`，不以 Windows 结果替代。

没有截图或视觉稿需求；Windows/macOS 原生证据仅在门禁允许的 fixture 阶段采集结构化日志，后续如修改 UI 再单独设计交互。
