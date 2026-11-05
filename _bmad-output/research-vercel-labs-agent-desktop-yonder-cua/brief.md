# 研究简报：vercel-labs/agent-desktop 作为 Yonder CUA 候选

## 决策

判断 `vercel-labs/agent-desktop` 能否作为 Yonder 的 Windows CUA Driver，并区分“Windows 平台 Adapter”“跨平台替代现有 trycua”“仅可借鉴实现”三种结论。

## 硬门槛

- 每步执行后可强制 Observe，并能返回有界截图或可访问性事实。
- 动作绑定可信窗口身份，动作前可复核 HWND/PID，避免焦点漂移。
- 用户真实输入可立即暂停；同一时间只有一个前台桌面租约。
- 不新增公网、本地 HTTP/TCP、P2P 或第二条 Agent 通道。
- 可作为受监管 Worker运行，支持超时、断连、停止、清理与 `unknown`。
- 产品可由 Yonder 自有签名和品牌承担控制提示，不显示第三方控制主体。
- 许可证和依赖许可允许产品分发；日志与证据不泄露正文、截图或完整 Payload。

## 研究维度

1. API、架构、动作与 Observe 链路。
2. Windows/macOS 支持、目标保护、用户接管与控制提示。
3. 许可证、活跃度、打包、运行依赖和供应链风险。
4. 对 Yonder 六边形边界的适配差距、建议 Spike 与淘汰条件。

## 方法与预算

技术研究，深度优先，标准预设；三条独立研究角度，每维最多两轮，优先仓库源码、提交历史、Release、Issue与官方文档。访问日期为2026-11-04。
