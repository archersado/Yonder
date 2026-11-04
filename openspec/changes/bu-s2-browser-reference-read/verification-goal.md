# BU-S2 Browser引用只读详情独立 Verification Goal

日期：2026-09-18  
结论：PASS（macOS 2026-09-18；Windows补证 2026-10-25）。完整Story技术验证通过，仍需PR与Archive流程。

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

浏览器详情并发与失败使用显式UI夹具；持久化引用、Gateway协商、只读返回及本机用户hand-off由SQLite集成测试覆盖。正式应用与真实ego-lite完成一次端到端交接。

## Windows补证（2026-10-25）

- 正式Windows桌面通过当前用户Named Pipe Gateway读取已提交引用，并在真实`ego:8`上验证Agent→用户→Agent控制权路径、finish审计和桌面重启后读取。
- ego-lite TaskSpace 9验证安全摘要、“打开 ego-lite”仅提交`task_id + expected_sequence`以及引用读取失败时基础详情继续可见。
- Windows产品目标单作业测试76项通过、0失败；并行全Workspace尝试因系统分页文件不足以`LNK1102`中断，不计为产品失败，改用单作业产品目标复验通过。
- 验证脚本使用唯一测试Agent，结束后将该Agent停用；完成任务作为不可变审计保留。未执行CUA。
- 证据：`apps/desktop/evidence/bu-s2-windows-20261025/result.json`、`verify-windows.ps1`、`browser-reference-round-04.mjs`。
