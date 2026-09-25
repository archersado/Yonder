# EN-S2 发布签名与分发设计

## 边界与依赖

本 Change 只建立发布产物与验证流程，不改变 Application、Gateway、SQLite、协议或 Driver 语义。构建脚本只调用系统原生命令，不引入自研签名或更新服务。

## 产物与版本

一次发布包含 desktop、CLI、MCP、IPC 协议、迁移和 Driver manifest，版本固定为同一 `major.minor.patch`。产物内记录提交 SHA、协议版本和 SQLite schema 版本；启动时只校验一致性，不自动迁移或下载。

最小增量先生成发布冻结清单：校验 Workspace/desktop/Driver manifest 版本、协议基线、SQLite schema 和 release 产物存在性，并记录 SHA-256。签名、公证和安装样本由后续增量承接。

release 构建由 `YONDER_BUILD_COMMIT` 显式注入当前 40 位 Git SHA；desktop 与 CLI 都提供无副作用的结构化构建身份入口。macOS 打包前要求源码树无已跟踪改动，分别读取两个二进制的身份并校验版本、`release` profile、提交以及相互一致性，然后生成只含这些公开构建字段的 `build-provenance.json`。签名会改变Mach-O字节，因此不把签名前哈希写入受签名资源形成自引用；发布冻结不信任来源文件的声明，而是重新执行最终包内两个身份入口、校验包签名、计算最终二进制哈希，并把身份提交与冻结时HEAD交叉校验。开发构建可使用 `development` 占位，但不能进入release包。

桌面启动读取同一份 `release-contract.json`，核对包版本、协议基线和 SQLite schema；不一致时拒绝启动。CLI/MCP 与桌面共享 Workspace 版本，由发布冻结清单校验。

macOS 打包脚本支持 `--channel dev|stable` 和 `--notary-profile`；公证凭据只保存在系统 Keychain Profile，不进入仓库、日志或任务状态。

## 签名与公证

macOS 使用 `codesign` 与 `notarytool`；Windows 使用 code-sign。证书与密钥只存在构建环境或系统密钥库，不进入仓库、任务状态或日志。

## 升级与数据安全

升级前强制备份 SQLite 主库和必要 sidecar；备份失败拒绝升级。迁移仅向前，失败时保留备份并回退旧版本。发布验证不得覆盖真实用户库。

macOS生命周期样本不发明安装器：在隔离临时根目录复现`.app`复制与原子替换。新装与候选暂存都必须先通过签名、通道、构建提交和内容审计；升级在同一文件系统把旧包移动为回退副本后再落位候选，任何注入失败都恢复旧包。用户数据始终位于应用包外；卸载仅移动应用至隔离Trash并验证数据树哈希不变。结构化结果只记录布尔结论、文件数与哈希，不记录库或附件正文，也不操作真实`/Applications`、HOME或用户数据库。

## 发布证据

发布命令输出结构化 JSON，记录版本、提交、构建环境、产物哈希、签名/公证结果、迁移回退测试和安装/卸载样本。证据不包含正文、截图、输入或完整命令输出。

发布冻结在计算macOS包哈希前审计装配边界：只允许`Contents/MacOS`中的desktop/CLI、`Contents/Resources/cua`固定Driver运行时与依赖、四份发布元数据、`Info.plist`和系统签名目录。拒绝符号链接、数据库、日志、环境/密钥/证书容器文件；对Yonder自有JSON、plist与Worker源码检查私钥和真实令牌形态。第三方依赖源码不做关键字扫描，避免把API字段名误报为凭据；其来源仍由锁文件和允许的包目录约束。审计结果只记录文件数、通道、构建提交、最终二进制哈希与布尔结论，不回显正文或匹配值。

## 失败与验证

每个平台分别验证新装、升级、降级回退、签名失败拒绝启动、迁移失败保留备份和卸载保留用户数据。Windows 按用户决定暂缓，不据此关闭 Story。
