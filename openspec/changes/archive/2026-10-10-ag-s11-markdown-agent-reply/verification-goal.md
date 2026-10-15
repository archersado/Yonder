# AG-S11 macOS 独立 Verification Goal

日期：2026-10-10
平台：macOS 26.5.1 arm64
结论：PASS（Windows Deferred）

## Goal

验证 Yonder 圈选回复卡能把严格序号的累计 Agent 回复安全渲染为常用 Markdown，同时不扩大 HTML、脚本、外部资源或持久化边界。

## 自动验证

- `node scripts/verification/desktop/check-safe-markdown.mjs`：PASS。覆盖标题、列表、引用、围栏/行内代码、粗体、斜体、删除线、安全链接文本、跨 delta 补齐、原始脚本、Markdown 图片、危险 URL 与禁止 `innerHTML`。
- `node scripts/verification/desktop/check-region-submit-recovery.mjs`：PASS，失败保留与重试门禁未回归。
- `node scripts/verification/desktop/check-voice-session.mjs`：PASS，语音整段提交未回归。
- `cargo test -p yonder-desktop`：lib 33/33、main 11/11 PASS。
- `scripts/check_architecture.py`：PASS。
- `openspec validate ag-s11-markdown-agent-reply`：PASS。

## 正式 bundle 验证

- 独立 worktree 构建 `yonder-desktop` 与 `yonder-cli`，`package-macos-preview.py --allow-adhoc` 成功。
- `codesign --verify --deep --strict target/debug/Yonda.app`：PASS。
- 签名 bundle 明确包含 `Contents/Resources/ui/safe-markdown.js`。
- 启动该正式 bundle 后，Computer Use 读取到“任务通道已连接 · 输入通道已连接”；圈选回复 WebView 正常加载并可进入/退出，无脚本初始化错误。

## 安全检查

渲染器只通过 `createElement`、`createTextNode`、`textContent` 与白名单属性构建 DOM；不创建 `script`、`img`、`iframe` 或可导航 `a` 节点。图片只显示替代文本占位，链接显示安全协议目标但不在 WebView 内导航。回复累计值仍受 AG-S9 64 KiB 和当前卡片内存生命周期约束。

## 平台范围

本次只声明 macOS 圈选回复卡通过。Windows 与其他回复载体继续 Deferred，不用 macOS 证据替代。
