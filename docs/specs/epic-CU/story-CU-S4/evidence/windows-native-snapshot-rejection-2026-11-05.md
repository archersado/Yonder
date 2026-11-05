# Windows 原生 Worker 快照拒绝响应验证（2026-11-05）

## 范围

在 `x86_64-pc-windows-msvc` 上，通过真实 Windows Worker 进程和继承的 stdio JSONL 验证原生 Backend 当前未开放动作时的快照拒绝路径。测试使用临时私有状态目录，显式关闭 trace 和 screenshots；未选择真实应用、未执行桌面动作。

## 结果

- `hello_result.status = ok`，随后 `snapshot_result.status = rejected`、`error_class = unsupported`、`target_identity_verdict = unavailable`，`snapshot_id` 与 `qualified_refs` 均为空；Worker 随后终止当前会话。
- Supervisor 的独立单元测试确认上述拒绝响应使快照准入失败，后续 `dispatch` 返回 `not_dispatched`。
- `cargo.exe +1.89.0 test --manifest-path spikes/agent-desktop-driver/Cargo.toml --locked --all-targets --offline --quiet`：25 个单元测试、1 个隔离测试、6 个 Worker 合约测试通过。
- `cargo.exe +1.89.0 clippy --manifest-path spikes/agent-desktop-driver/Cargo.toml --locked --all-targets --offline -- -D warnings`：通过。
- `generate-schema --check`：通过；bridge 合同哈希、harness tree 哈希及证据 fixture 已更新到同一构建清单。

## 尚未证明

本验证只覆盖拒绝响应及零派发。`window_generation_proven` 仍为 `false`；真实原生 snapshot、动作投递、Observe、用户抢占和 macOS 同样本均未通过，因此不改变 Spike 与产品接线门禁。
