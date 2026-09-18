# E0 技术可行性门禁

本文件为跨模块技术门禁索引，不再作为项目 Epic。六个 Spike 已迁入各技术模块 Story，当前状态见 [项目总规划](../../docs/specs/README.md)；下方保留历史平台结论。

目标：在正式产品研发前，用可复现证据确定关键技术路线。

## Spike Stories

| Story | 验证内容 | 决策输出 | 状态 |
|---|---|---|---|
| E0-S1 | Tauri 2 常驻桌面、资源基线与跨平台 Local Socket | AD-E0-01 | 进行中 |
| E0-S2 | 当前 Qwen CUA SDK 与 trycua CUA Driver 对照 | AD-E0-02 | 已完成 |
| E0-S3 | ego-lite Task Space 集成、部署与状态映射 | AD-E0-03 | Deferred：上游暂无 Windows Runtime |
| E0-S4 | Rust 与 Node OOXML Adapter 对照 | AD-E0-04 | 已完成 |
| E0-S5 | Windows/macOS 原生上下文与 Chromium Native Messaging | AD-E0-05 | Windows 已完成；macOS 延期 |
| E0-S6 | SQLCipher、FTS5、附件加密与 Credential Store | AD-E0-06 | Windows 已完成；macOS 延期 |

门禁：当前 Windows 范围的六个 Story 均须完成 Verification Goal 并形成唯一 ADR；已明确延期项不得被描述为已验证。未通过项先修订架构，不得带入产品 Epic。
