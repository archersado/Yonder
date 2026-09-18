# DS-S2 任务总览独立 Verification Goal

日期：2026-09-18  
结论：PASS（macOS 首批任务总览子范围）。Windows 依用户决定暂缓；完整 Story 不 Archive。

## 验收结果

- 真实宿主：`TaskHost` 以可信本机身份打开未加密 SQLite，恢复两个不同 Agent 的真实任务并拒绝伪造身份；进程内及跨进程锁竞争均被拒绝。
- 真实桌面：macOS 正式 `com.yonder.desktop` 显示本地 Agent 创建的两个任务；悬停打开、面板操作、移出隐藏及同一 Space 通过。
- 列表与详情：现有浏览器回归覆盖 21 项跨页、详情、时间线部分失败、列表失败保留过期快照、筛选回首页、“进行中”仅显示 `running`。
- 并发刷新：回归人为延迟旧“进行中”请求，紧接发起“全部”请求；旧响应较晚返回后仍保持“全部”20项，证明轮次令牌阻止旧响应覆盖。
- 键盘可访问：Enter/Space 打开时聚焦面板，Escape/关闭走同一原生关闭路径；桌面不提供全屏形态。

## 证据

- `verification-host-core.md`
- `verification-hover-macos.md`、`verification-task-state-hover-macos.md`、`verification-running-filter-macos.md`
- `apps/desktop/evidence/live-local-agent-20260914/agent-result.json`
- `apps/desktop/evidence/live-local-agent-menu-text-20260914/result.json`
- `apps/desktop/evidence/task-filter-20260917/result.json`
- `apps/desktop/evidence/task-overview-verification-20260918/browser-result.json`
- `apps/desktop/check-task-space.mjs`

21 项跨页、失败注入和响应竞态使用显式测试夹具，不声称为 21 个真实执行任务。真实 SQLite 、两任务、授权边界和 macOS 原生菜单由独立证据覆盖。逐任务控制、外部 ego-lite 关联和 Windows 最终证据属完整 Story 剩余范围。
