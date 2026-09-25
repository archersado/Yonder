# TM-S5 任务事件响应字节预算 Verification Goal

性质：独立验证记录；日期：2026-09-24。Story：TM-S5；Change：`tm-s5-event-response-budget`；依据：AD-TM-15、TM5-AC08。

## 验证目标

- 在协商版本投影后，以真实 JSON UTF-8 编码计量单事件 8 KiB 与完整成功响应 256 KiB。
- 到达整页预算时仅返回连续完整前缀；调用方从最后实际事件序号继续，无漏项。
- JSON 转义使单事件超限时明确失败；首项放不下时不返回误导性的空页；无后续事件则正常返回空页。
- 不改变 SQLite 事件、Outbox 或现有协议字段；Windows 原生验证按用户决定暂缓。

## 环境与结果

- macOS arm64；`cargo test --workspace`：88 项通过（边界测试最后加严后，定向 2 项重跑通过）。
- 合成事件测试覆盖转义、单项超限、256 KiB 精确临界值、首项超限、整页截断和续读；Task Space 既有分页脚本 `node apps/desktop/check-task-space.mjs` 通过。
- `openspec validate --all`：66 项通过；`python3 scripts/check_architecture.py` 与 `python3 scripts/test_check_architecture.py` 通过；`git diff --check` 通过。
- `cargo build -p yonder-desktop -p yonder-cli` 通过，macOS 预览包重建并重启。
- 本增量不改变原生 UI 或 Driver；查询字节预算为跨平台纯 Rust 路径。Windows 原生证据仍按用户指示暂缓。

## 结论与保留项

macOS 本机此增量 PASS，可进入本 Change 的 verifying 阶段。完整 TM-S5 的产物身份/不可变清单、总配额、附件清理与删除同步尚未实施，不 Archive/Done；Windows 原生验证暂缓。
