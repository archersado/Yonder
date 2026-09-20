# TM-S3 接管定位可见性独立 Verification Goal

日期：2026-09-20（macOS）
结论：PASS（当前 Space 可见性核验增量）

## 验证目标

验证 WorkFocus 在 AX 前台与焦点成功后，还会核验原 `pid + window_id` 的 layer 0 窗口位于 WindowServer on-screen 列表；正常定位、最小化恢复、身份不唯一和窗口关闭仍遵守既有保守失败规则。

## 结果

- `cargo test -p yonder-adapters -p yonder-application`：38 项通过。
- `clang -fsyntax-only -Wall -Wextra crates/adapters/src/work_focus_macos.c -framework ApplicationServices` 通过。
- 隔离原生样本 `python3 apps/desktop/check-work-focus-macos.py apps/desktop/evidence/tm-s3-work-focus-visibility-macos-20260920` 通过：`visible_on_active_space=true`，最小化恢复、同名重叠拒绝、关闭拒绝、释放后拒绝均通过；未启动 Recording 或附加应用。
- 结构化证据：[result.json](../../../apps/desktop/evidence/tm-s3-work-focus-visibility-macos-20260920/result.json)。

## 保留边界

该 Goal 仅验证当前 Space 的可见性核验，不能代替真实不同 Space 或多显示器前置样本。Windows 按用户要求暂缓；Recording 与交回 Observe 仍分别受 RC-S1/TM-S4 门禁约束，TM-S3 继续处于 implementing。
