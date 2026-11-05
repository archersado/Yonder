# CU-S5 架构设计

## 边界

沿用 CU-S4 隔离 Supervisor → inherited stdio JSONL → 受监管 Worker → 固定 `agent-desktop` core。Application 仍拥有任务级 lease/fence、输入 generation、取消、deadline、逐步 Observe 和 `unknown`；Driver 不持有任务状态。v2 只在独立 Spike 目录运行，产品依赖、协议与持久化不变。

## 门禁 A：目标对象连续性

固定源码的 `execute_by_ref` 经 `execute_ref_action_with_context`、`execute_with_auto_wait_and_lease`、`dispatch_resolved_landing` 再解析保存引用。v2 源码探针先固定这条调用链及所有动作终端。试验补丁须在 snapshot 保存同一进程内原生对象，并在派发时仅使用保留对象；任何 RefStore 重新解析、轮询重试、focus 重新定位或 fallback 均拒绝。每个动作终端还必须证明交付点仍绑定旧对象；UIA/AX 对象可读或 RuntimeId 相等都不足以推出动作不会转发替代对象。物理输入若无法绑定原生对象，按 `unsupported`；固定必做样本因此无法完成时淘汰候选。

先用无副作用 fixture 验证同窗按钮替换和整窗重建；随后在双平台原生样本验证 Windows 完整 HWND 复用与 macOS AX 对象替换。不能制造样本时标记未验证，不把“未观察到误投递”记作通过。直到 ADR 后续更新，原生 `dispatch` 固定返回 `unsupported/not_delivered`。

## 门禁 B：输入观察

Host 独立 observer 是准入事实源，维护单调 generation。候选标记必须与当前 attempt 和 Worker 实例绑定；其他 injected、无法辨认或 observer 丢失均递增 generation 并冻结。Windows `dwExtraInfo`、macOS `eventSourceUserData` 只提供候选载体，须通过伪造、碰撞、延迟事件、Worker 崩溃、会话/桌面变化与 observer 故障样本。Windows 低级钩子静默移除风险使其不能单独表明 observer ready；Raw Input 不能在未证明其他 injected 覆盖的情况下单独承担该职责。UIA Invoke/Legacy 和 AX 语义动作分别验证是否产生日志可见的低级输入事件。

## 构建和证据

同一上游 commit、同一跨平台 patch set，v2 独立目录与 patch/source/lock/harness/sample 哈希；拒绝在 v1 `target/patched-agent-desktop` 原位改动。证据只记录封闭枚举、布尔、计数、耗时和哈希，不含输入正文、截图、完整窗口树。沿用 CU-S4 统一样本语义，v2 版本冻结后再跑 Windows/macOS；独立 Verification Goal 复核 V2-01～05。截止日 2026-11-11，结论只更新 ADR，产品接线另建 Story。
