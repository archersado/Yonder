# Windows IDENTITY-02 旧引用只读诊断（2026-11-05）

## 方法

扩展自建 WinForms fixture 的受控命令：先保持原目标窗口不变，fixture 销毁旧按钮并创建同标识、同标签、同位置的新按钮；随后 fixture 自行关闭原目标窗口，并在同一进程创建同名、同按钮标识但代际递增的新目标。诱饵窗口继续存在。诊断通过上游公开 `snapshot::run_with_context` 保存原目标的限定引用，在两次替换后分别调用公开 `resolve_saved_ref` 重新解析旧引用。不调用 `execute_by_ref`、`execute_action` 或任何桌面输入 API；命令只控制自建 fixture 的窗口与按钮生命周期。

## Windows x86_64/MSVC 实测

`cargo test --test windows_stale_ref -- --exact old_ref_is_rejected_after_fixture_rebuild_without_action --nocapture` 通过。按钮替换阶段输出 `button_replaced=true`、`window_generation_unchanged=true`、`old_element_rebound=true`：上游公开解析函数接受旧限定引用，并在当前完整窗口中找到同谓词元素。窗口重建阶段输出 `old_hwnd_reused=false`、`generation_changed=true`、`old_ref_rejected=true`：上游返回 `StaleRef` 或 `WindowNotFound` 范围内的拒绝。两个阶段目标和诱饵点击计数都保持 0。

追加 3 次独立进程复测均得到相同布尔结果；测试现将 `old_element_rebound=true` 固定为断言，以便固定上游行为变化时重新审阅本证据。

进一步在按钮替换前保留由 `resolve_saved_ref` 返回的原生 `NativeHandle`：替换前 `get_live_element` 可读，替换后同一保留句柄返回 `StaleRef`，而对同一旧限定引用重新调用 `resolve_saved_ref` 仍成功。此结果仅适用于当前 WinForms fixture 的只读路径，提示保留原对象可能避免按谓词重绑定；尚未验证在动作交付期间、其他 UIA provider 或 macOS AX 上成立。

继续检查整窗重建后的保留句柄时，首次样本在读取计数阶段发现诱饵计数为 1 次点击，按零副作用规则作废；本测试未调用动作 API，点击来源未确认，不归因于 Driver。重新启动独立 fixture 后，同一保留句柄在整窗重建后也返回 `StaleRef`，目标和诱饵计数都为 0。新旧完整 HWND 仍不同，因此这仍不能覆盖完整 HWND 复用。

## 证据边界

按钮替换后的成功解析证明固定上游的旧引用可以按同谓词重新绑定当前元素，但没有执行动作，不能直接记为误投递。此次没有出现完整 HWND 复用；不能把普通旧 HWND 失效时的拒绝推断为完整复用时也安全。只读解析没有执行最终 UIA Invoke/Legacy 交付，不证明动作路径的窗口代际门禁。`IDENTITY-02` 正式动作样本、用户输入抢占和 macOS 同样本仍未验证；原生 dispatch 继续 `unsupported/not_delivered`。
