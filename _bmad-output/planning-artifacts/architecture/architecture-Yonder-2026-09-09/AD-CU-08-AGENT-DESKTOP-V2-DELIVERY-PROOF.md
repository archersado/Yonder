# AD-CU-08 agent-desktop v2 交付门禁验证

状态：Proposed；日期：2026-11-05。关联 AD-CU-07、CU-S5、`cu-s5-agent-desktop-v2-spike`。Architecture Impact：architecture-change（仅隔离 Spike 的补丁与原生输入观察方案）。

## 决策

批准在 AD-CU-07 的 **2026-11-11 截止日内**开展一次独立 v2 验证，目标是证明或否定候选 Driver 的两个缺口：原生对象到最终副作用交付点的身份连续性，以及当前 attempt 合成输入与其他输入的可靠区分。v1 固定源码、唯一 hit-test 补丁、manifest 和证据保持原样；v2 使用同一上游 revision `d3dc190afbf34cd2377252398ddd5c30efbd093a`，独立源码目录、补丁集、哈希、manifest 与样本版本。v2 不替换 trycua、不接产品 Adapter/Gateway、不开放真实用户任务。

本决策只批准**只读探针和可拒绝的机制验证**。`dispatch` 继续 `unsupported/not_delivered`，直到本决策所列两项门禁在同一固定补丁集的 Windows 与 macOS 原生样本中均被独立验证。若截止日仍无法证明，按 AD-CU-07 淘汰候选；不延长时间，也不降样本范围。

## 身份连续性门禁

固定上游 `execute_by_ref` 会在 action 路径从 RefStore 再次解析限定引用；Windows 同窗按钮销毁重建已实测发生谓词重绑定。v2 必须使 Worker 的一次动作始终指向 snapshot 时保留的**同一个原生元素对象**，不得在派发路径重新调用 `resolve_saved_ref`、按谓词查询替代元素，或在 stale 后自动等待/重试。保留的 UIA/AX 对象也不能只在派发前做一次读取：须逐个证明动作类型在**实际副作用交付点**仍使用该对象，且 provider 不会把旧对象操作转发给同属性新对象。Windows 需覆盖同窗元素替换、整窗重建与完整 HWND 复用；macOS 需覆盖 AX 对象替换与窗口重建。任一误投递直接 Rejected；无法制造完整 HWND 复用只能记录未证明，不能判通过。

对不能绑定到原生对象的物理鼠标、键盘或 focus 路径，前置身份检查与后置 Observe 不足以关闭检查到发送之间的竞态。v2 必须找到可验证的最终交付约束，否则该动作类型 `unsupported`；若固定必做样本依赖此动作类型，则候选整体淘汰。上游交互租约不是目标对象锁，也不能作为此项证明。

## 输入观察门禁

Host 独立维护单调 `input_generation`；只有同时绑定当前 `attempt_id` 与 `worker_instance_id` 的输入才可归为 `driver_current_attempt`。Windows `SendInput.dwExtraInfo` 和 macOS `CGEvent.eventSourceUserData` 仅作为 v2 补丁候选载体，不是身份认证：必须覆盖 Windows 鼠标、键盘、shell surface 三处发送入口及 macOS HID post、键盘 `post_to_pid`、异常释放路径，并验证完整事件接收、伪造/碰撞、Worker 崩溃、观察器失效、桌面/会话切换、其他进程注入及 UIA/AX 语义动作。未知事件与观察器丢失均冻结。Windows 低级钩子可能被系统静默移除，不能单独作为持续准入事实；Raw Input 若无法覆盖其他程序注入事件，也不能单独通过。无法获得同时覆盖真实用户输入与未知合成输入且有可验证存活状态的来源时，`observer_ready=false`，不派发。

## 实施和证据顺序

1. 先冻结 CU-S5 三份 Story 设计、OpenSpec、v2 样本与拒绝门槛，再建立隔离源码和新补丁。v2 构建输入必须有单独 manifest；不能沿用 v1 构建或把 v1 只读结果计作 v2 动作结果。
2. 先做源码调用链和只读 fixture 探针；只记录布尔、计数、耗时、哈希与封闭错误枚举。不得保存输入正文、窗口树、截图或 Agent Payload。
3. 每个候选补丁须有双平台同语义测试、固定源码哈希与独立 Verification Goal。只有目标身份、输入观察、故障拒绝三个门禁均通过，才能另行更新本 ADR 允许隔离 fixture 的真实动作样本；该更新仍不授权产品接线。
4. 任一硬门槛失败、缺 macOS 原生环境、或 deadline 到期时保持未通过；明确区分 `failed` 与 `environment_unavailable`。

## 依据

- [AD-CU-07](AD-CU-07-AGENT-DESKTOP-DRIVER-SPIKE.md) 的补丁扩围门槛和淘汰条件。
- [Windows 同窗替换与保留对象只读诊断](../../../../docs/specs/epic-CU/story-CU-S4/evidence/windows-identity02-read-only-stale-ref-2026-11-05.md)。
- [Windows 输入归属源码审计](../../../../docs/specs/epic-CU/story-CU-S4/evidence/upstream-windows-input-attribution-source-audit-2026-11-05.md)与[macOS 源码审计](../../../../docs/specs/epic-CU/story-CU-S4/evidence/upstream-macos-input-attribution-source-audit-2026-11-05.md)。
