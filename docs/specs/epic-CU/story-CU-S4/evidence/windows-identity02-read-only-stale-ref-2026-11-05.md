# Windows IDENTITY-02 旧引用只读诊断（2026-11-05）

## 方法

扩展自建 WinForms fixture 的受控命令：fixture 自行关闭原目标窗口，并在同一进程创建同名、同按钮标识但代际递增的新目标；诱饵窗口继续存在。诊断通过上游公开 `snapshot::run_with_context` 保存原目标的限定引用，重建后调用公开 `resolve_saved_ref` 重新解析旧引用。不调用 `execute_by_ref`、`execute_action` 或任何桌面输入 API；命令只控制自建 fixture 的窗口生命周期。

## Windows x86_64/MSVC 实测

`cargo test --test windows_stale_ref -- --exact old_ref_is_rejected_after_fixture_rebuild_without_action --nocapture` 通过。安全诊断输出：`old_hwnd_reused=false`、`generation_changed=true`、`old_ref_rejected=true`。上游返回 `StaleRef` 或 `WindowNotFound` 范围内的拒绝；重建后的目标和诱饵点击计数都保持 0。

## 证据边界

此次没有出现完整 HWND 复用；不能把普通旧 HWND 失效时的拒绝推断为完整复用时也安全。只读解析也没有执行最终 UIA Invoke/Legacy 交付，不证明动作路径的窗口代际门禁。`IDENTITY-02` 正式动作样本、用户输入抢占和 macOS 同样本仍未验证；原生 dispatch 继续 `unsupported/not_delivered`。
