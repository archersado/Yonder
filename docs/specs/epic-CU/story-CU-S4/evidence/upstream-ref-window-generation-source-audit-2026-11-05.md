# 固定上游源码的 qualified ref 窗口代际审计（2026-11-05）

## 审计对象

固定 `agent-desktop v1.1.0` commit `d3dc190afbf34cd2377252398ddd5c30efbd093a` 加 CU-S4 唯一 hit-test 补丁；源码由 `spikes/agent-desktop-driver/build-manifest.json` 的 `patched_source_tree_sha256` 绑定。本次只读审计没有修改上游源码，也没有执行桌面动作。

## 调用路径

1. `crates/core/src/commands/execute_by_ref.rs` 把限定引用交给 `execute_ref_action_with_context`。
2. `crates/core/src/commands/helpers.rs` 从 `RefStore` 加载该 snapshot 的 `RefEntry`。
3. `crates/core/src/ref_action/mod.rs` 在动作前调用 `resolve_saved_ref`，再进行 actionability preflight 与 `execute_verified_action`。
4. `crates/core/src/refs/resolve_saved_ref.rs` 的 `recorded_window` 重查窗口；`belongs_to_entry` 只比较 owner kind、PID、`source_window_id` 和进程启动令牌。它没有窗口对象代际字段。Windows 快照的 `WindowInfo.id` 是 `w-<HWND>`。
5. Windows adapter 没有覆盖 `resolve_element_strict`；`crates/core/src/adapter/observation.rs` 的默认实现是 `not_supported`。正常 Windows ref 走重新观察与严格谓词匹配路径，谓词唯一性本身不区分已销毁窗口与同 HWND 的新窗口。
6. `crates/core/src/ref_action/mod.rs` 在 `resolve_saved_ref`、稳定预检及可能的滚动恢复后调用 `execute_verified_action`；`crates/core/src/actions/post_action.rs` 随后调用 adapter 的 `execute_action`。Windows adapter 最终在 `crates/windows/src/actions/dispatch.rs` 通过 UIA Invoke 或 Legacy 默认动作交付 Click。公开 `execute_by_ref` 参数没有窗口代际凭据，也没有在最终 UIA 交付点执行外部代际核验的回调。

## 结论与边界

若同一进程销毁目标窗口并复用 HWND，而新窗口出现相同元素谓词，当前上游静态身份字段可能仍全部匹配。这是由源码条件推导出的**误投递风险**，本次没有构造实际 HWND 复用并观察一次误投递，因此不能记为已发生的硬失败。Host 在调用上游命令前单独检查窗口，也不能证明上游随后重新解析和交付动作时仍是原窗口。

包装 `ActionOps::execute_action` 可把一次只读核验放在上游交付前，但核验到 UIA Invoke 之间仍有窗口重建竞态；仅凭包装层调用前后的 fixture property 读取不能宣布这一门槛通过。若要继续评估这一候选，需要先为固定 revision 设计可覆盖最终交付点的可靠门禁，并按架构变更流程审定；不能把现有前置 snapshot 反例或未观察到 HWND 完整值复用当作证明。

下一步必须在冻结样本中验证复用，并取得覆盖 Worker 从 ref 解析到副作用交付期间的窗口代际证据；若固定引擎与唯一允许补丁无法实现，按 AD-CU-07 的身份门槛拒绝并淘汰候选。现有原生 Backend 保持 `unsupported/not_delivered`。
