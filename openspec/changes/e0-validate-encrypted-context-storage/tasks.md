当前归属 Story：ST-S1；规划：`docs/specs/epic-ST/story-ST-S1/README.md`。旧编号保留历史追溯。

# 任务

- [x] 固化版本、统一用例和淘汰条件
- [x] 验证 SQLCipher 版本、密文文件头与错误密钥拒绝
- [x] 验证加密库内 FTS5 中英文检索
- [x] 验证状态、事件与 Outbox 同事务及回滚
- [x] 验证附件 AES-256-GCM 往返与篡改拒绝
- [x] 验证 Windows Credential Manager 写入、读取与删除
- [x] 记录构建、性能、体积和许可证证据
- [x] 产出 AD-E0-06
- [x] 完成 Verification Goal

## macOS 补充批次（原 Windows 结论保持）

- [x] 原生 Keychain 临时密钥往返、重复写入拒绝与删除后不可读
- [x] 独立 Verification Goal 与结构化日志（verification-keychain-macos.md）
- [ ] macOS 完整 SQLCipher/FTS5/附件加密组合验证与 ADR 范围评审
- [ ] 产品凭据 Adapter、内存清零及宿主身份认证接线（单独产品变更）
