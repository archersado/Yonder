# macOS Verification Goal：圈选前暂停桌面任务

日期：2026-09-21
状态：PASS（独立复核）

## 实施者证据

- 小龙与托盘入口均经正式UDS创建任务并执行真实`computer.step`；圈选层出现前，任务进入`paused`，控制记录为`pause/stopped`，桌面租约随后释放。
- 两条路径均只有一次CUA attempt，未触发接管聚焦；当前Accepted架构尚未授权产品Recording，正式SQLite中没有Recording表，本Change没有虚构第二份状态源。
- 两条路径的任务`sequence`、events数量、outbox数量及各自最大序列均为6，状态、事件与Outbox保持同事务事实。
- Host合约覆盖已停止、已Observe、`prepared`与`unknown`边界：前两者可暂停；后两者返回错误，任务保持running且桌面租约不释放。
- 正式Gateway构造`unknown`结果后，从小龙入口得到稳定反馈；结构化证据确认圈选层未打开、`pause`保持pending、第二个CUA任务得到精确`-32012`停止/准入错误，且未触发聚焦。Recording产品Schema仍不存在。
- 无桌面租约时，原生accepted文字提交回归通过，不产生附件帧且不记录正文。
- 结构化证据位于`apps/desktop/evidence/cx-s2-pause-before-select-macos-20260920/`，不含截图、正文或完整Agent Payload。

## 自动检查

- `cargo test --workspace --locked`
- Swift、Python与两个前端脚本语法检查
- `openspec validate cx-s2-pause-before-select --strict`
- `scripts/check_architecture.py`
- `git diff --check`

Windows按用户决定暂缓；多显示器、VI-S1语音组合和云端WSS不在本增量范围。完整CX-S2继续保持`verifying`。

## 独立复核

非实现者复核HEAD `34643a2`：`prepared/unknown`均明确返回错误、任务保持running且Admission桌面租约不释放；原生unknown入口不显示圈选层，第二个CUA任务收到精确`-32012`，控制保持`pause/pending`，无聚焦事实，事件与Outbox同序。Recording Schema尚未获Accepted架构授权且正式库不存在对应表，本Change未建立或调用Recording入口。结论PASS，允许归档本Change。
