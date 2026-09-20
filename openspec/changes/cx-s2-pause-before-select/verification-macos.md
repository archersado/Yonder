# macOS Verification Goal：圈选前暂停桌面任务

日期：2026-09-20  
状态：待独立复核

## 实施者证据

- 小龙与托盘入口均经正式UDS创建任务并执行真实`computer.step`；圈选层出现前，任务进入`paused`，控制记录为`pause/stopped`，桌面租约随后释放。
- 两条路径均只有一次CUA attempt，未触发接管聚焦或Recording。
- 两条路径的任务`sequence`、events数量、outbox数量及各自最大序列均为6，状态、事件与Outbox保持同事务事实。
- Host合约覆盖已停止、已Observe与`unknown`边界：前两者可暂停，`unknown`拒绝打开圈选并保留租约。
- 无桌面租约时，原生accepted文字提交回归通过，不产生附件帧且不记录正文。
- 结构化证据位于`apps/desktop/evidence/cx-s2-pause-before-select-macos-20260920/`，不含截图、正文或完整Agent Payload。

## 自动检查

- `cargo test --workspace --locked`
- Swift、Python与两个前端脚本语法检查
- `openspec validate cx-s2-pause-before-select --strict`
- `scripts/check_architecture.py`
- `git diff --check`

Windows按用户决定暂缓；多显示器、VI-S1语音组合和云端WSS不在本增量范围。完整CX-S2继续保持`verifying`。
