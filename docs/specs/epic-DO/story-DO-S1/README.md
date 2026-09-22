# DO-S1 OOXML Adapter 选型

Story: DO-S1
Epic: DO
Status: done
OpenSpec: e0-compare-ooxml-adapters

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

Accepted AD-E0-04已唯一选择Rust进程内实现并淘汰Node Worker；统一DOCX/XLSX/PPTX样本、哈希冲突、未修改part保真、ZIP重开及WPS实机打开均PASS。产品Adapter属于DO-S2，不扩张本Story范围。
2026-09-22补齐WPS实机打开证据：DOCX/XLSX/PPTX三个复杂输出在WPS Office 12.1.28496中直接打开，无修复或损坏提示。见 `openspec/changes/archive/2026-09-22-e0-compare-ooxml-adapters/evidence-wps-20260922/result.json`。

## OpenSpec 与验证

openspec/changes/archive/2026-09-22-e0-compare-ooxml-adapters/

[Change](../../../../openspec/changes/archive/2026-09-22-e0-compare-ooxml-adapters/proposal.md)；独立验证在该 Change 内维护，记录存在不代表通过。

[原 Story 正文与历史验证](legacy-record.md)。旧编号仅作追溯，不用于新 PR。
