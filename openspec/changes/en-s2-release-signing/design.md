# EN-S2 发布签名与分发设计

## 边界与依赖

本 Change 只建立发布产物与验证流程，不改变 Application、Gateway、SQLite、协议或 Driver 语义。构建脚本只调用系统原生命令，不引入自研签名或更新服务。

## 产物与版本

一次发布包含 desktop、CLI、MCP、IPC 协议、迁移和 Driver manifest，版本固定为同一 `major.minor.patch`。产物内记录提交 SHA、协议版本和 SQLite schema 版本；启动时只校验一致性，不自动迁移或下载。

最小增量先生成发布冻结清单：校验 Workspace/desktop/Driver manifest 版本、协议基线、SQLite schema 和 release 产物存在性，并记录 SHA-256。签名、公证和安装样本由后续增量承接。

桌面启动读取同一份 `release-contract.json`，核对包版本、协议基线和 SQLite schema；不一致时拒绝启动。CLI/MCP 与桌面共享 Workspace 版本，由发布冻结清单校验。

macOS 打包脚本支持 `--channel dev|stable` 和 `--notary-profile`；公证凭据只保存在系统 Keychain Profile，不进入仓库、日志或任务状态。

## 签名与公证

macOS 使用 `codesign` 与 `notarytool`；Windows 使用 code-sign。证书与密钥只存在构建环境或系统密钥库，不进入仓库、任务状态或日志。

## 升级与数据安全

升级前强制备份 SQLite 主库和必要 sidecar；备份失败拒绝升级。迁移仅向前，失败时保留备份并回退旧版本。发布验证不得覆盖真实用户库。

## 发布证据

发布命令输出结构化 JSON，记录版本、提交、构建环境、产物哈希、签名/公证结果、迁移回退测试和安装/卸载样本。证据不包含正文、截图、输入或完整命令输出。

## 失败与验证

每个平台分别验证新装、升级、降级回退、签名失败拒绝启动、迁移失败保留备份和卸载保留用户数据。Windows 按用户决定暂缓，不据此关闭 Story。
