# BU-S1 ego-lite 集成与平台能力

Story: BU-S1
Epic: BU
Status: verifying
OpenSpec: bu-s1-ego-lite-bridge-macos

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

2026-11-01 修复 Windows 多版本安装回归：Desktop 不再直接选择安装目录中的最大版本，而是优先 canonicalize 当前用户 ego active-version 指针，并确认真实目标仍位于固定 Citro Labs 数字版本目录。当前 active `0.6.1.13` 在同时存在未激活 `0.6.2.5` 时完成真实 Desktop Task Space 生命周期；证据见 `openspec/changes/bu-s1-ego-lite-bridge-windows/verification-windows-active-runtime-20261101.md`。

macOS Bridge 已通过；Windows Bridge 于 2026-10-25 使用 ego-lite `0.6.1.11` 完成真实 Task Space 生命周期、引用持久化和独立验证。Windows 增量及证据见 `openspec/changes/bu-s1-ego-lite-bridge-windows/`；Change 仍需正常 PR/Archive 门禁后才能归档。

## OpenSpec 与验证

openspec/changes/bu-s1-ego-lite-bridge-macos/

[Change](../../../../openspec/changes/bu-s1-ego-lite-bridge-macos/proposal.md)；独立验证在该 Change 内维护，记录存在不代表通过。

[原 Story 正文与历史验证](legacy-record.md)。旧编号仅作追溯，不用于新 PR。
