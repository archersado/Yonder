# AG-S10 macOS 独立 Verification Goal

日期：2026-10-10  
平台：macOS 26.5.1 arm64  
结论：PASS（Windows Deferred）

## Goal

独立验证圈选输入在未被 Agent 接受时保留图片、问题与来源，错误可诊断且重试门禁正确；accepted 与明确清理路径不回归。

## 构建与门禁

- `CARGO=/Users/archersado/.cargo/bin/cargo python3 scripts/check_architecture.py`：PASS。
- `openspec validate ag-s10-circle-submit-recovery`：PASS。
- `cargo test -p yonder-application`：83/83 PASS。
- `cargo test -p yonder-desktop`：lib 33/33、main 11/11 PASS。
- `node apps/desktop/check-region-submit-recovery.mjs`：PASS，覆盖失败不清空、unknown 禁止重试和附件超限反馈。
- `swiftc -typecheck apps/desktop/check-region-preview-macos.swift`：PASS。
- `cargo build -p yonder-desktop -p yonder-cli` 与 `package-macos-preview.py --allow-adhoc`：PASS；`target/debug/Yonda.app` 的标识为 `com.yonder.desktop`，deep/strict codesign 校验通过。

## 正式 GUI 证据

使用上述正式 app bundle 与 Computer Use 对真实圈选窗口执行：打开圈选、拖动取得截图、输入“测试保留”、在没有可接收 Agent 时点击发送。

观察结果：

1. 界面显示“当前没有可接收输入的 Agent；内容已保留。”。
2. AX 树继续包含“所选屏幕区域预览”，没有删除图片节点。
3. 问题输入框仍为原测试值，没有置空，并保持可编辑。
4. “重新圈选”和“取消”仍可用；未发生持久化或自动重投。

受控 Agent `rejected` 对照实际收到 4 个正式 Gateway 帧、1 个附件分块；附件长度与 SHA-256 一致，`agent.input` 引用了同一附件且问题非空。拒绝后正式 GUI 同样保留图片、问题并显示可重试反馈。

## 补充回归说明

Workspace 全量运行时，AG-S10 所属 Application/Desktop 测试全部通过；另有两个既存非本变更失败：Command 进程 fixture 的空 PID 解析，以及 EX-S2 连续计划测试返回 `InvalidInput`。本变更未修改对应 Adapter 文件，已用所属 crate 全量测试隔离确认 AG-S10 通过，不将既存失败伪报为本 Story 通过项。

## 安全与平台范围

验证未保存截图或问题正文到仓库、SQLite、事件、Outbox 或日志。Windows 按主人既有决定继续 Deferred，本结论不外推到 Windows。
