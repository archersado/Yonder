# ST-S1 加密存储技术验证

Story: ST-S1
Epic: ST
Status: design-review
OpenSpec: e0-validate-encrypted-context-storage

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

保留现有证据，不新增密钥工作；v1 迁移须单独设计和备份。

## OpenSpec 与验证

openspec/changes/e0-validate-encrypted-context-storage/

[Change](../../../../openspec/changes/e0-validate-encrypted-context-storage/proposal.md)；独立验证在该 Change 内维护，记录存在不代表通过。

[macOS 组合验证](../../../../openspec/changes/e0-validate-encrypted-context-storage/verification-storage-macos.md)已通过 SQLCipher、FTS5、事务与 AES-GCM 附件路径；产品加密实施仍按 AD-ST-01 延期。

[原 Story 正文与历史验证](legacy-record.md)。旧编号仅作追溯，不用于新 PR。
