# EN-S2 发布冻结 macOS 独立 Verification Goal

日期：2026-09-23。Story：EN-S2；Change：`en-s2-release-signing`。状态：本机发布冻结子范围 PASS；完整 Story 仍为 `implementing`。

## Goal

独立验证 macOS 本机 release 构建内的版本冻结、Driver manifest、协议与 SQLite schema 一致性，以及 `.app` 临时签名与产物哈希。此 Goal 不覆盖正式签名、公证、升级/回退、安装/卸载样本或 Windows。

## 通过条件

- Workspace、desktop、CLI/MCP、协议、SQLite schema 和 Driver manifest 版本一致。
- `Yonda.app` 生成成功，`channel.json` 为 `dev`。
- `codesign --verify --deep --strict` 通过。
- 发布清单记录 desktop、CLI/MCP 与 `.app` 哈希。
- 升级前备份与迁移失败回退既有测试通过。
- OpenSpec 与架构检查通过。

## 结果

PASS。证据见 [`verification-release-freeze-macos.md`](verification-release-freeze-macos.md) 与 `target/release/yonder-release-manifest.json`。

## 边界

当前 `.app` 为临时签名，`spctl` 预期拒绝。正式 codesign 身份、`notarytool` 公证、安装/升级/回退样本、真实用户库备份和 Windows 均未完成。EN-S2 保持 `implementing`，不 Archive。
当前证据来自本工作树；正式发布仍需在提交后重新生成清单，确保提交 SHA 与产物完全对应。
