# AD-BU-03 Browser引用只读展示

状态：Accepted（2026-09-18用户修订）；日期：2026-09-18。关联BU-S2、DS-S2、AD-BU-01、AD-DS-01。

## 决定

协议1.15新增`task.browser.get`只读查询，复用`task.read`授权和AD-BU-01已持久化的Browser引用。Application先校验任务可读，再返回现有`BrowserReference`；引用缺失返回`reference=null`，不创建、接管、交接或恢复ego-lite空间。

Task Space详情并行读取步骤、时间线和Browser引用，只展示`ego:<正整数>`、所有权、托管页数、完成状态与更新时间序号。读取失败不影响任务基础详情。UI不得提交或改写引用，不显示URL、标题、正文、截图或浏览历史。

用户修订确认进入对应任务使用ego-lite控制权动作：用户从Yonder打开Agent控制的活动空间时调用`handOff()`，Agent恢复执行时调用`takeOverTaskSpace()`。详情显示“打开 ego-lite”；可信桌面命令只接收`task_id`，重新读取任务、当前引用和序号，通过既有受监督Browser用例执行hand-off、Observe并提交事件与引用。已结束、缺失、身份不符、并发冲突及依赖不可用必须明确失败，不创建替代空间。

## 影响

Rust协议类型仍为JSON Schema和TypeScript唯一来源；不改SQLite结构。macOS复用现有ego-lite Adapter、Browser资源门禁与attempt/Observe事务；Windows无Runtime时仍可读取已经存在的引用事实，但打开入口明确不可用。
