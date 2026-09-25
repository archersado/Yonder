# Jev macOS 真实远端验证

日期：2026-09-24

- 使用产品 `MacosJevPort`、macOS Keychain 服务 `Yonder`/账号 `jev`、固定 `@typesafe-ai/sdk@0.6.0` 和 `jev-latest`。
- 只提交固定的有界候选，不执行任务动作，不记录 API Key、请求正文或响应正文。
- 首次按 1500ms 单次决策上限调用，结果为 `TimedOut`；未自动重试。
- 随后仅做一次无凭据、无正文的端点可达性检查：TLS 约 1349ms、总耗时约 1795ms，已超过当前 1500ms 决策上限；端点返回 405，证明网络与 TLS 可达。
- 根据端点实测先修订 AD-EX-02、Story 与 OpenSpec，将 macOS 产品 SDK/Worker 单次上限调整为 3000ms、重试仍为 0；随后发起一次新的人工验证，远端调用成功并返回受支持候选 `handback`。
- 第二次验证总耗时为 4823ms；该值包含 Keychain 读取与 Worker 启动时间，SDK/Worker 内部远端调用仍受 3000ms 上限约束。
- 两次验证均未记录 API Key、请求正文或响应正文；第二次是规格调整后的新验证，不是第一次调用的自动重试。
