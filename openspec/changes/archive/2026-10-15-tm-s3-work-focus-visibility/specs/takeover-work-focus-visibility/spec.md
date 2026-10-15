# 接管定位可见性核验 Delta

## ADDED Requirements

### Requirement: 前置后的真实可见性

系统 MUST 在 macOS WorkRef 前置后核验同一 `pid + window_id` 的 layer 0 窗口出现在 WindowServer 的 on-screen 列表；AX 前台/焦点成功本身 MUST NOT 作为跨 Space 定位成功的唯一依据。

#### Scenario: on-screen缺失即失败

- **WHEN** AX前台或焦点调用返回成功，但同一`pid + window_id`的layer 0窗口不在WindowServer on-screen列表
- **THEN** 定位失败，不发布focused事实

### Requirement: 保守失败

若可见性核验失败，系统 MUST 返回既有 `VerificationFailed` 并保持任务 paused；系统 MUST NOT 移动窗口、挑选同名窗口、自动重试、启动 Recording 或自动恢复 Agent。

#### Scenario: 可见性失败不自动恢复

- **WHEN** 可见性核验失败
- **THEN** 任务保持paused并返回VerificationFailed，不移动窗口、不自动重试、不启动Recording或恢复Agent
