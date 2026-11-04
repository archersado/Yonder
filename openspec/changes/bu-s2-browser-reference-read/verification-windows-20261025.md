# BU-S2 Windows 独立 Verification Goal

日期：2026-10-25
结论：PASS
范围：BU-S2 Browser Gateway、引用事实、控制权交接、重启恢复与Task Space引用UI；不含CUA。

## 原生端到端结果

- Windows PowerShell启动当前构建的正式`yonder-desktop.exe`，通过当前用户私有Named Pipe `com.yonder.desktop.agent`完成Gateway握手；协商协议1.17，`browser.execute`为available。
- 真实ego-lite空间`ego:8`按步骤执行create、hand-off、take-over、finish；外部引用始终不变，所有权依次为`agent → agentDelegatedToUser → agent`。
- 任务`task_82cfb598b501b53e954aaf2100a3a227`最终为`completed@18`，18条事件中4个Browser动作均为Observed attempt。
- Yonda桌面退出并使用同一当前用户数据库重启后，`task.browser.get`仍读取到相同`ego:8`和`finished=true`，且没有自动重开或接管空间。
- 验证使用唯一测试Agent并在结束后停用；已完成任务按不可变审计语义保留。脚本只停止自己启动的两个Yonda进程，并清理自己的临时WebView目录。

## UI与回归

- ego-lite TaskSpace 9以显式UI夹具验证Browser安全摘要、活动Agent引用的“打开 ego-lite”、仅提交`task_id + expected_sequence`、以及引用局部失败时任务基础详情仍保留；通过后已执行`finish({ keep: [] })`。
- Windows产品库与二进制目标以单作业执行76项测试，全部通过。第一次并行全Workspace构建因Windows分页文件不足产生`LNK1102`，缩小并发后通过，判定为环境资源限制而非产品缺陷。
- 架构、OpenSpec严格校验和diff检查均通过（见最终验证记录）。

## 证据

- `apps/desktop/evidence/bu-s2-windows-20261025/result.json`
- `apps/desktop/evidence/bu-s2-windows-20261025/verify-windows.ps1`
- `apps/desktop/evidence/bu-s2-windows-20261025/browser-reference-round-04.mjs`
- `apps/desktop/evidence/bu-s2-windows-20261025/run-browser-reference-check.ps1`

## 判定

BU-S2的Windows暂缓条件已解除，Windows与既有macOS证据共同满足Story技术验证。未执行CUA。Change尚需正常PR与Archive门禁，不能仅凭本记录标为Archived。
