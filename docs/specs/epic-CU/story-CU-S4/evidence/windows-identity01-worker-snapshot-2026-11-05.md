# Windows IDENTITY-01 原生 Worker 只读快照（2026-11-05）

## 测试范围

自建 WinForms fixture 在同一进程显示两个同名窗口，每个窗口有一个 `AutomationId = YonderIdentity01Button` 的按钮。fixture 自行设置角色、代际和点击计数 window property；Host 与 Worker 仅只读这些 property。无桌面动作派发。

## 结果

- 首次人工探针采样发现诱饵点击计数已变化，该次样本作废；未将原因归给 Driver。重新启动 fixture 后，独立 Rust oracle 读到目标/诱饵角色 `1/2`、互异代际摘要，两个计数均为初始值 `1`（表示 0 次点击）。
- `native_identity01_snapshot_uses_frozen_fixture_window` 在真实 Windows Worker 进程中通过 stdio JSONL 完成 `hello → snapshot_result`；Worker 独立核对目标 PID、HWND、进程启动摘要、fixture role/代际摘要和生命周期屏障，并在冻结窗口的完整 UIA 快照中返回恰好一个限定引用。测试结束后两个窗口的点击计数仍为 0。
- 首轮 Worker 快照因 `private file DACL grants an untrusted principal` 拒绝 RefStore 写入。原因是 Worker 预建状态目录继承宽 ACL。现改由上游 `WindowsPrivateFile::ensure_private` 创建缺失的原生状态根；预存在宽权限目录仍拒绝，不降低 ACL 门槛。相同测试随后通过。
- 一次完整 `--all-targets` 测试中，此原生用例返回 `Driver` 拒绝；当时 Worker 标准错误被测试丢弃，无法定位阶段。随后将该受控用例的 Worker 标准错误接到测试输出（实现只输出阶段、错误类别和错误码），单独复测 5 次及连同 Worker 契约用例复测 3 次均通过。偶发拒绝的根因尚未确认；不据此声称稳定性门禁已通过。
- 再次运行 `cargo test --all-targets --locked --offline` 全部通过：25 个库单元测试、1 个 Windows containment 测试、7 个 Worker 契约测试。`cargo clippy --all-targets -- -D warnings`、`cargo fmt --check`、生成物 `--check` 和 `git diff --check` 通过。
- 后续在同一自建 fixture 的三个独立 Worker 会话中，真实目标身份取得唯一引用；把目标的窗口代际摘要替换为诱饵摘要时，Worker 返回 `identity` 拒绝和空引用；提交诱饵自身真实身份时同样返回 `identity` 拒绝和空引用。两个拒绝会话终止后，目标与诱饵点击计数均为 0。该反例只验证 snapshot 前置身份门禁。

## 边界

本结果仅证明自建 Windows fixture 的原生只读快照、唯一引用和私有状态根接线。`NativeBackend::dispatch` 仍返回 `unsupported/not_delivered`；没有执行 `dispatch → Observe`，没有证明动作路径窗口代际、用户输入抢占或 macOS 同样本。fixture property 不作为其他应用的身份凭据。
