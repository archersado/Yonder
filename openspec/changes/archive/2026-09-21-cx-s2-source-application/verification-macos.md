# macOS Verification Goal：确认卡来源应用

日期：2026-09-21
状态：PASS（独立复核）

## 实施者证据

- 正式`com.yonder.desktop` bundle在真实非Yonda前台应用上启动圈选，确认卡显示非降级来源；重新圈选后保持同一来源，取消后窗口和PreviewSession清场。
- Application定向测试覆盖名称去空白、128字符上限、控制字符拒绝、重新圈选保留和结束清零。
- 结构化证据位于`apps/desktop/evidence/cx-s2-source-application-macos-20260921/result.json`，只含布尔值，不含真实应用名、截图或正文。
- 实现不修改`region_preview_submit`、Agent协议或持久化路径；来源仅通过安全JSON事件投影为确认卡`textContent`。

## 自动检查

- `cargo test --workspace --locked`：61项通过。
- `openspec validate cx-s2-source-application --strict`：通过。
- `python3 scripts/check_architecture.py`：通过。
- Swift、JavaScript语法检查与`git diff --check`：通过。

## 独立复核要求

非实现者须从当前提交独立重建正式bundle，使用真实非Yonda前台应用复跑首次显示、重新圈选保持和取消清场；检查降级与数据边界，不记录真实应用名。通过后才能Archive。

Windows按用户决定暂缓；多显示器与云端WSS不在本Change范围，完整CX-S2继续保持`verifying`。

## 独立复核结论

非实现者在提交`a8d52a8d2757ef5448c53cbe0ccdf9e703f4b2af`上重新构建并签名正式`com.yonder.desktop` bundle；二进制SHA-256为`eb62a520ecb6c9b2ddbcd6fa90a479410e711c780e18091fbba3943f28de038e`。

使用真实非Yonda前台应用复跑macOS原生路径：首次确认卡显示非降级来源，重新圈选后保持同一来源，取消后窗口与PreviewSession清场。验证输出仅记录`source_nonfallback`、`source_preserved_after_reselect`和`cleared`布尔值，没有记录真实应用名或截图。

Application定向测试确认名称去空白、空值降级、128字符上限、控制字符拒绝、重新圈选保留及`clear`清零。代码引用与提交差异复核确认来源字段只存在于macOS原生读取、PreviewSession和本地UI投影；`region_preview_submit`、Agent协议、Gateway、SQLite、事件与Outbox没有来源字段。使用运行时取得的来源值扫描正式应用数据目录和bundle日志未发现泄漏，提交内结构化证据也不含真实应用名、正文、截图或完整Agent Payload。

`cargo test --workspace --locked`共61项通过，严格OpenSpec、架构门禁、Swift/JavaScript语法检查和`git diff --check`通过。结论为PASS，允许Archive本Change；Windows、多显示器和云端WSS仍按既定边界保留，完整CX-S2不因此转Done。
