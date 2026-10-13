# Verification Goal：真实终态动画

状态：PASS（macOS 子范围，2026-09-18）。完整 DS-S1 不 Archive。

## 结论

- 正式 macOS 预览宿主通过同一 UDS Gateway 创建任务、声明/推进步骤，并由 Yonder Bridge 完成 ego-lite Task Space；`browser.execute(finish)` 返回真实 `completed`、序号 10，触发统一终态展示出口。
- Application 检查覆盖完成/失败终态、读取不重播、拒绝响应不播放；完整 Workspace 52 项测试与架构关联检查通过。
- ego-browser Task Space 78 验证 `success` 进入、1.8 秒恢复、相同事实标识不重播，以及边缘隐藏态收到新终态后唤醒；空间已完成并关闭。
- 正式预览包已重建并重启。结构化证据见 [result.json](../../../apps/desktop/evidence/terminal-presentation-20260918/result.json)。

## 保留门禁

公开 Gateway 尚无失败终态写方法，本变更不为动画新增协议；`failed` 共享判定已覆盖，待真实失败写 Story 接入后补原生事件证据。Windows 按用户要求暂缓，因此完整 Story 不 Done/Archive。

## 2026-10-13 回归补充

- 修复终态动画恢复响应瞬间旧快照、只读请求覆盖终态、Task Space 缺少 Runtime overlay，以及连续 BUA 第二次尝试缺少步骤边界导致 projector 冲突。
- Application 84 项、desktop 37 项测试通过；OpenSpec strict 与架构关联检查通过。
- 正式 `Yonda.app` 经 MCP 完成 BUA `create → finish`，返回 `completed/sequence 12`；Yonda 清除旧步骤，只读查询不恢复 executing。
- 通过应用菜单正常退出并重启后，同一任务仍为 `completed/sequence 12`。结构化证据见 [result.json](../../../apps/desktop/evidence/ds-s1-terminal-runtime-overlay-macos-20261013/result.json)。
