# DS-S1 Windows 真实终态展示验证

日期：2026-11-01
结论：**PASS（真实 BUA `completed → success → 恢复` 子范围）**

## 验收映射

- `browser.execute(finish)` 经正式 Windows Named Pipe Gateway 和真实 ego-lite Task Space 完成，任务最终为 `completed`，Browser 空间已关闭。
- Windows 原生桌宠收到 `success`，显示完整成功姿态，约 1.8 秒终态脉冲后恢复真实派生状态；不是读取历史或启动快照触发。
- 2026-10-29 原生证据包含 success 与前后 idle 的不同截图、请求在途标记及最终任务/空间收口；2026-11-01 又以独立真实 BUA 会话记录 `executing → success → paused` 的结构化展示顺序，并确认 `finished=true`。
- 全程未执行 CUA。

权威原生报告：`../e0-validate-desktop-foundation/verification-windows-bua-presentation-20261029.md`。本次交叉证据：`apps/desktop/evidence/ds-s2-windows-20261101/native-task-space-result.json`，其中任务最终 `completed`、Browser `finished=true`，宿主诊断依次含 `presentation-change/executing`、`presentation-change/success` 和恢复到 `paused`。

## 保留门禁

真实 `failed` 终态未通过：正式 Gateway 的失败终结要求 CUA Runtime，而本轮按用户要求排除 CUA。现有 Application 判定和拒绝响应不播放的单元检查仍通过，但不能替代 Windows 原生失败事件。解决方案是在用户另行恢复 CUA 范围后，以真实失败 attempt 触发统一终态出口，验证 `failed` 一次性展示、约 1.8 秒恢复、相同事实不重播及空间/任务收口。

因此本报告关闭 Windows 成功终态子范围，不把完整 DS-S1 或 `failed` 原生门禁标为完成。
