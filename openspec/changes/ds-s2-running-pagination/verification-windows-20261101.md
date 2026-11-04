# Windows 独立 Verification Goal：任务总览过滤、取消与时间线

日期：2026-11-01
平台：Windows
范围：DS-S2 / TM-S3 created 取消 / TM-S5 只读时间线；不执行 CUA。
结论：本文件列出的 Windows 子范围 PASS；执行中停止、接管、Recording 和完整 Story/PR 不在本结论内。

## 通过结果

- 正式 Named Pipe Gateway 1.19 使用专用测试 Agent 创建 22 个真实任务。按任务 ID 排序后，前 20 项正式 `task.cancel`，最大 ID 任务通过已安装 ego-lite 进入真实 BUA `running`。
- `task.list(running_only=true, limit=20)` 第一页直接返回唯一运行任务；同一授权范围的“全部”第一页为 20 项且不含该运行任务，并提供下一页。证明过滤发生在游标和页大小之前。
- Windows 原生 Task Space 通过小龙键盘 Enter 入口打开；“进行中”显示 `第 1 页 · 1 项`，切换“全部”显示 `第 1 页 · 20 项`，并成功进入第 2 页及后续页面。
- 卡片“取消任务”把 `task_eb83029f047f63a93f339081559e3b8b` 从 `created` 提交为 `cancelled`；刷新后卡片仍可见。Gateway 读回序号 2、事件 2 条，证明取消不删除任务或历史。
- 真实 BUA `task_f956b3736988abefef2724bfd21b7b9b` 在原生详情初次显示 20 条事件，点击“加载更多时间线”后显示 28 条且移除继续入口；序号无重复、无倒退。
- 验证结束通过正式 BUA finish 把该任务提交为 `completed`，最终序号/事件数均为 34，外部引用 `ego:16` 标记 finished；ego-lite `listTaskSpaces()` 返回空数组。其余本轮任务均为 `cancelled`，测试 Agent 已禁用。

## 原生与结构化证据

- `apps/desktop/evidence/ds-s2-windows-20261101/native-task-space-result.json`
- `apps/desktop/evidence/ds-s2-windows-20261101/native-running.png`
- `apps/desktop/evidence/ds-s2-windows-20261101/native-all-page1.png`
- `apps/desktop/evidence/ds-s2-windows-20261101/native-all-page2.png`
- `apps/desktop/evidence/ds-s2-windows-20261101/native-cancel-retained.png`
- `apps/desktop/evidence/ds-s2-windows-20261101/native-timeline-more.png`
- 可复现脚本：`apps/desktop/evidence/ds-s2-windows-20261101/verify-native-task-space.ps1`

## 验证器失败记录与修复

失败均保留，不以最终 PASS 覆盖历史事实：

1. 首轮尝试对子进程覆盖 `APPDATA`，但 Windows Known Folder API 仍返回真实 Roaming AppData，临时 Agent 登记在错误库；Gateway 以 `-32003` 正确拒绝。解决：在 Tauri 实际系统应用目录登记专用 Agent，全部样本走正式终态并保留审计历史。
2. 首个 BUA 循环在 hand-off/take-over 路径返回存储不可用；任务按崩溃恢复保留为 `interrupted`，对应 ego 空间 8 已显式 finish。解决：分页/时间线样本改用不转移所有权的真实 `observe` 循环。
3. 早期原生 UIA 先后遇到离屏托盘坐标 `NaN`、WebView2 声明 Invoke 但未触发 DOM click、长名称定位歧义。解决：首选小龙键盘 Enter 入口；对可见 WebView 控件使用真实中心点点击；样本使用短唯一名称并记录逐页无障碍树。
4. 在异常收口加入正式 BUA finish 和 created 任务取消后，后续 UI 自动化失败均未再制造运行中或 created 残留。加入该收口前的 `task_fb207a80fbd557f9522e159c11b07a38` 与 `task_ea5dfd6e21b7953a3a60d2b27d636ea4` 仍按产品规则保留为 `interrupted`；其外部 ego 空间 8、9 已完成关闭，不修改数据库伪造终态。

## 保留门禁

- TM-S3 执行中取消/暂停/接管仍需要停止确认、工作定位、Recording 与交回 Observe 的 Windows 原生证据。
- 本结果不覆盖 CUA、跨虚拟桌面/多显示器或完整 DS-S2/TM-S3/TM-S5 Archive。
