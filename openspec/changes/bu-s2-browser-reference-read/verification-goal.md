# BU-S2 Browser引用只读详情独立 Verification Goal

日期：2026-09-18  
结论：PASS（macOS读取与打开子范围）。Windows按用户决定暂缓；完整Story不Archive。

## 验收结果

- 协议：Rust唯一来源生成的JSON Schema与TypeScript包含协议1.15 `task.browser.get`和`browser-state`响应。
- 授权与存储：SQLite集成测试经Gateway 1.15读取已提交的`ego:41`引用；查询复用任务可读权限，不启动Bridge或写入状态。
- Task Space：ego-browser TaskSpace 60验证引用摘要、时间线并行展示及Browser引用局部失败；失败时任务基础详情仍保留。
- 打开交互：ego-browser TaskSpace 61验证Agent控制的活动引用显示“打开 ego-lite”，并只提交`task_id + expected_sequence`给可信桌面命令。
- 真实交接：正式Yonda应用从任务详情打开`ego:64`；ego-lite前台显示“Browser 打开验证 332000 / 你正在控制”，证明进入对应Task Space。SQLite把引用所有权更新为`agentDelegatedToUser`，任务保持`running@8`且事件为8条。
- 执行边界：用户打开复用既有Browser hand-off、资源门禁、attempt、Observe和同事务引用更新；Agent恢复继续使用既有take-over。

## 证据

- `apps/desktop/evidence/browser-reference-read-20260918/result.json`
- `apps/desktop/evidence/browser-reference-read-20260918/task-space-browser-reference.png`
- `apps/desktop/evidence/browser-open-live-20260918/result.json`
- `apps/desktop/evidence/browser-open-live-20260918/task-menu-page2.png`
- `apps/desktop/evidence/browser-open-handoff-20260918/bridge-result.json`
- `apps/desktop/check-task-space.mjs`
- `cargo test --offline --locked -p yonder-protocol -p yonder-application -p yonder-adapters -p yonder-desktop`
- `python3 scripts/check_architecture.py`

浏览器详情并发与失败使用显式UI夹具；持久化引用、Gateway协商、只读返回及本机用户hand-off由SQLite集成测试覆盖。正式应用与真实ego-lite完成一次端到端交接。Windows原生证据不在本次通过范围。
