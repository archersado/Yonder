# 发布签名与分发

## ADDED Requirements

### Requirement: 固定版本发布

Yonder 发布产物 MUST 使用同一固定版本，并包含 desktop、CLI、MCP、IPC 协议、迁移和 Driver manifest。

#### Scenario: 版本一致

发布脚本 MUST 校验提交 SHA、协议版本和 SQLite schema 版本一致；不一致时拒绝发布。

#### Scenario: 构建提交可追溯

- **WHEN** macOS 打包脚本装配 release desktop 与 CLI
- **THEN** 两个二进制必须分别报告相同的固定版本、`release` profile 与 40 位构建提交
- **AND** 构建提交必须等于当前无已跟踪改动源码树的 HEAD
- **AND** 包内构建来源记录必须声明两个二进制各自的构建身份
- **AND** 发布冻结必须重新执行最终包内身份入口、校验包签名、记录最终二进制SHA-256，并拒绝旧提交、开发 profile、身份缺失或身份不一致的产物

### Requirement: 签名与公证

macOS 产物 MUST 通过 `codesign` 与 `notarytool`；Windows 产物 MUST 通过 code-sign。签名失败 MUST 阻止发布。

#### Scenario: 签名校验

发布脚本 MUST 在本地验证签名链、时间戳和公证结果，并保存结构化证据。

### Requirement: 升级前备份

升级 MUST 在执行迁移前备份 SQLite 主库和必要 sidecar；备份失败 MUST 拒绝升级。

#### Scenario: 迁移失败回退

迁移失败 MUST 保留备份并允许回退旧版本，不得丢失任务数据。

#### Scenario: macOS安装生命周期样本

- **WHEN** 对已通过发布审计的macOS候选包执行新装、升级、失败回退与卸载样本
- **THEN** 所有操作必须限制在独立临时根目录，不得读写真实应用或用户数据库
- **AND** 新装与升级候选必须先通过签名与发布身份校验，再原子落位
- **AND** 候选落位前后的注入失败必须恢复旧应用树
- **AND** 卸载必须只移动应用包，用户数据库、sidecar与附件哈希保持不变
- **AND** 结构化证据不得包含用户数据或附件正文

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
