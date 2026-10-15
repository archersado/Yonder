# AG-S12 macOS 独立 Verification Goal

日期：2026-10-12
平台：macOS 26.5.1 arm64
结论：PASS（Windows Deferred）

## Goal

验证圈选确认/回复卡对常用 Markdown 松散列表提供稳定语义和明确标记，并能从标题区原生拖动，同时不扩大内容执行、窗口权限、协议或持久化边界。

## 自动验证

- `node scripts/verification/desktop/check-safe-markdown.mjs`：PASS；松散列表归并为一个 `ul` 与三个 `li`，既有流式、危险内容和禁用 `innerHTML` 合约继续通过。
- `node scripts/verification/desktop/check-region-preview-drag.mjs`：PASS；标题拖动区、4 点阈值、释放 pointer capture、原生拖动命令和最小 capability 权限通过。
- `node scripts/verification/desktop/check-region-submit-recovery.mjs`、`node scripts/verification/desktop/check-voice-session.mjs`：PASS。
- `cargo test --offline --locked -p yonder-desktop`：lib 33/33、main 11/11 PASS。
- `cargo check --offline --locked -p yonder-desktop`、`node --check apps/desktop/ui/region-preview.js`：PASS。
- `openspec validate ag-s12-reply-card-interaction --strict`、`python3 scripts/check_architecture.py` 与 `git diff --check`：PASS。

## 正式 GUI 验证

- 独立 worktree 构建 `yonder-desktop` 与 `yonder-cli`，`package-macos-preview.py` 生成并临时签名正式 debug bundle，`codesign --verify --deep --strict` 通过。
- 从正式 Yonder 桌宠入口打开圈选层并进入确认卡；原生窗口从 `(185,219)` 移动到 `(301,296)`，位移后输入区与取消按钮继续可操作。
- 结构化证据：[result.json](../../../apps/desktop/evidence/ag-s12-reply-card-drag-macos-20261012/result.json)。

## 安全与边界

拖动 capability 仅增加 `core:window:allow-start-dragging`，未开放任意位置设置、缩放或其他窗口命令。Markdown 继续只创建白名单 DOM，不执行 HTML、脚本、外部图片或危险 URL。Gateway、协议、SQLite、事件、Outbox 和回复正文生命周期均未改变。

## 平台范围

本 Goal 只声明 macOS 圈选确认/回复卡通过。Windows 原生证据继续 Deferred，不使用 macOS 结果替代。
