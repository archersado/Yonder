# 发布签名与分发

## ADDED Requirements

### Requirement: 固定版本发布

Yonder 发布产物 MUST 使用同一固定版本，并包含 desktop、CLI、MCP、IPC 协议、迁移和 Driver manifest。

#### Scenario: 版本一致

发布脚本 MUST 校验提交 SHA、协议版本和 SQLite schema 版本一致；不一致时拒绝发布。

### Requirement: 签名与公证

macOS 产物 MUST 通过 `codesign` 与 `notarytool`；Windows 产物 MUST 通过 code-sign。签名失败 MUST 阻止发布。

#### Scenario: 签名校验

发布脚本 MUST 在本地验证签名链、时间戳和公证结果，并保存结构化证据。

### Requirement: 升级前备份

升级 MUST 在执行迁移前备份 SQLite 主库和必要 sidecar；备份失败 MUST 拒绝升级。

#### Scenario: 迁移失败回退

迁移失败 MUST 保留备份并允许回退旧版本，不得丢失任务数据。

### Requirement: 发布通道与安全证据

发布产物 MUST 只声明 `dev` 或 `stable` 通道，并在冻结清单中记录可复验的产物哈希和包内容安全审计结果。

#### Scenario: 审计macOS发布包

- **WHEN** 发布脚本冻结一个macOS `.app` 产物
- **THEN** 包内文件必须来自显式允许的desktop、CLI、Driver运行时与发布元数据路径
- **AND** 必须拒绝SQLite数据库、日志、环境文件、私钥、证书容器或用户凭据文件
- **AND** 必须拒绝自有文本产物中的私钥或真实令牌形态
- **AND** `channel.json`只能包含与发布请求一致的`dev`或`stable`
- **AND** 结构化证据不得包含检测到的敏感值或完整文件正文

#### Scenario: 包内容审计失败

- **WHEN** 包内出现未允许的路径或敏感文件/内容
- **THEN** 发布冻结必须失败且不得生成可发布结论
