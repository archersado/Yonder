# 接管定位可见性核验 Delta

## ADDED Requirements

### Requirement: 前置后的真实可见性

系统 MUST 在 macOS WorkRef 前置后核验同一 `pid + window_id` 的 layer 0 窗口出现在 WindowServer 的 on-screen 列表；AX 前台/焦点成功本身 MUST NOT 作为跨 Space 定位成功的唯一依据。

### Requirement: 保守失败

若可见性核验失败，系统 MUST 返回既有 `VerificationFailed` 并保持任务 paused；系统 MUST NOT 移动窗口、挑选同名窗口、自动重试、启动 Recording 或自动恢复 Agent。
