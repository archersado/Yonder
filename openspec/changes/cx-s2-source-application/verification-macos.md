# macOS Verification Goal：确认卡来源应用

日期：2026-09-21  
状态：待独立复核

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
