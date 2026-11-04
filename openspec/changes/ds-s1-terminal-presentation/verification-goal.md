# Verification Goal：真实终态动画

状态：PASS（macOS 与 Windows 真实成功终态子范围；Windows 于 2026-11-01 补证）。完整 DS-S1 不 Archive。

## 结论

- 正式 macOS 预览宿主通过同一 UDS Gateway 创建任务、声明/推进步骤，并由 Yonder Bridge 完成 ego-lite Task Space；`browser.execute(finish)` 返回真实 `completed`、序号 10，触发统一终态展示出口。
- Application 检查覆盖完成/失败终态、读取不重播、拒绝响应不播放；完整 Workspace 52 项测试与架构关联检查通过。
- ego-browser Task Space 78 验证 `success` 进入、1.8 秒恢复、相同事实标识不重播，以及边缘隐藏态收到新终态后唤醒；空间已完成并关闭。
- 正式预览包已重建并重启。结构化证据见 [result.json](../../../apps/desktop/evidence/terminal-presentation-20260918/result.json)。
- Windows 正式 Named Pipe Gateway 与真实 ego-lite BUA 完成链路已验证 `completed → success → 恢复`，Task Space 正常关闭；映射报告见 [verification-windows-20261101.md](verification-windows-20261101.md)。

## 保留门禁

公开 Gateway 的失败终结要求 CUA Runtime，本变更不为动画新增协议；`failed` 共享判定已覆盖，待用户恢复 CUA 范围后补 Windows 原生事件证据。因此完整 Story 不 Done/Archive。
