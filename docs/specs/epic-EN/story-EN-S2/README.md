# EN-S2 发布签名与分发

Story: EN-S2
Epic: EN
Status: implementing
OpenSpec: en-s2-release-signing

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

本 Story 已完成设计与 OpenSpec 严格校验，并实施最小发布冻结：Workspace、desktop、Driver manifest、协议与 SQLite schema 版本一致性检查，release 产物哈希记录见[发布冻结 macOS 验证](../../../../openspec/changes/en-s2-release-signing/verification-release-freeze-macos.md)。该子范围已有独立 [Verification Goal](../../../../openspec/changes/en-s2-release-signing/verification-goal-release-freeze-macos.md)。当前不引入云端发布平台、自动更新服务或运行时迁移逻辑。
macOS 已能生成 `.app` 并完成临时签名校验；正式签名与公证仍在后续增量内完成。

2026-09-25 发布包内容审计增量PASS：冻结脚本现在按显式装配白名单审计macOS包，拒绝符号链接、数据库、日志、环境/私钥/证书容器文件与自有文本中的真实密钥形态，并校验`dev|stable`请求与包内通道一致。真实临时签名包191个文件通过，`stable`请求审计`dev`包被拒绝；证据不含匹配正文。既有release二进制并非当前提交重新构建，本结果不代替正式签名、公证或当前版本发布验证，见[独立Verification Goal](../../../../openspec/changes/en-s2-release-signing/verification-bundle-content-macos.md)。

同日构建来源绑定增量PASS：release desktop与CLI必须分别报告相同的版本、profile和40位构建提交，打包只接受当前干净源码树HEAD；包内`build-provenance.json`声明两份身份，冻结阶段重新执行最终包内身份入口、校验签名并记录最终二进制SHA-256，拒绝旧提交、开发profile或被改写产物。当前提交release重建、临时签名装配与冻结验证通过，见[独立Verification Goal](../../../../openspec/changes/en-s2-release-signing/verification-build-provenance-macos.md)。正式签名、公证、安装生命周期与Windows仍保留门禁。

同日macOS隔离安装生命周期增量PASS：使用哈希不同的上一版与当前提交临时签名包，在临时根目录完成新装、升级、候选落位失败回退和可恢复卸载；合成SQLite主库、sidecar与附件全程哈希不变，未访问真实`/Applications`、HOME或用户库。见[独立Verification Goal](../../../../openspec/changes/en-s2-release-signing/verification-install-lifecycle-macos.md)。正式Developer ID、公证、Gatekeeper与Windows仍保留门禁。

OpenSpec：`openspec/changes/en-s2-release-signing/`。

实施前置包括：桌面宿主、CLI/MCP、协议版本、SQLite 迁移和 Driver manifest 的版本边界明确；macOS notarize 与 Windows code-sign 证书策略确定；数据库备份和升级路径已冻结。
