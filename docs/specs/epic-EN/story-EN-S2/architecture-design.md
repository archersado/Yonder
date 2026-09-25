# EN-S2 架构设计

## 边界与依赖

本 Story 只定义发布产物与验证流程，不改变运行时代码、协议语义或任务状态机。构建脚本只调用平台原生工具链，不引入自研签名或更新服务。

## 产物与版本

一次发布包含 desktop、CLI、MCP、IPC 协议、迁移和 Driver manifest，所有产物使用同一 `major.minor.patch` 版本，禁止 `latest`。发布前冻结提交 SHA、协议版本和 SQLite schema 版本；产物内记录该三元组，运行时启动时只做一致性校验，不尝试自动迁移或下载。release 构建必须显式注入 40 位提交 SHA，desktop 与 CLI 分别提供只读构建身份输出；打包脚本只接受两者版本、profile 与提交完全一致且提交等于当前干净源码树 HEAD 的产物。包内 `build-provenance.json` 固定记录两份二进制的构建身份；签名后的发布冻结重新执行两份只读身份入口、校验包签名并记录最终二进制 SHA-256，不能用当前 HEAD 为旧二进制背书，也不在签名资源内制造自引用哈希。

Driver manifest 固定为 `apps/desktop/driver-manifest.json`，只声明当前实际交付的 CUA/BUA Driver 名称、版本和支持平台；发布冻结脚本校验它与 Workspace 版本一致，并记录哈希。未交付的平台不写入 manifest。

macOS 预览/发布共用 `apps/desktop/package-macos-preview.py`：`--release` 使用 release 产物并输出 `.app`；正式发布必须提供非临时 codesign 身份，本机验证可用 `--allow-adhoc` 显式降级。

发布通道由 `--channel dev|stable` 固定，写入 `.app` 的 `channel.json`；公证通过 `--notary-profile` 读取本机 Keychain Profile，不把 Apple 凭据写入仓库或日志。

桌面启动使用 `apps/desktop/release-contract.json` 校验版本、协议基线和 SQLite schema；契约内容来自 Workspace、协议与存储的正式常量，缺失或不一致时启动失败，不做自动修复。

## 状态与契约

SQLite 仍是任务当前状态唯一事实源，发布脚本不写任务、事件或 Outbox。协议版本、迁移版本和 Driver manifest 由既有 Rust 单一来源生成，不在发布层手写第二份模型。若版本不一致，启动失败；不做自动修复或静默降级。

## 签名与公证

- macOS 使用 `codesign` 与 `notarytool`，产物必须通过本地校验和 Gatekeeper 样本。
- Windows 使用 code-sign，产物必须通过签名链和时间戳校验。
- 签名密钥只存在于构建环境或系统密钥库，不进入仓库、日志、任务事件或 SQLite。

## 升级与数据安全

升级前强制备份 SQLite 主库和必要 sidecar；备份失败时拒绝升级。迁移仅向前，失败时保留备份并回退旧版本。任何发布验证不得覆盖真实用户库，签名或迁移失败不得写任务状态。

macOS安装生命周期使用系统`.app`目录语义，不新增安装器或自更新服务。验证工具只在独立临时根目录中建立`Applications`、`Library/Application Support/com.yonder.desktop`与可恢复`Trash`：新装先完整复制至暂存路径、验证签名与发布身份后原子落位；升级先验证候选，再把旧应用原子移动为回退副本，候选落位或复核失败时恢复旧应用。卸载只移动应用包，不删除用户数据库、sidecar或附件。验证必须比较应用树哈希与用户数据树哈希，证据不得记录数据库或附件正文。

## 发布证据

发布命令输出结构化 JSON，包含版本、提交、构建环境、产物哈希、签名校验、公证结果、迁移回退测试和安装/卸载样本。证据只保存命令结果，不保存正文、截图、输入或完整输出。

## 失败与验证

每个平台分别验证新装、升级、降级回退、签名校验失败拒绝启动、迁移失败保留备份、卸载保留用户数据。Windows 按用户决定暂缓，当前不关闭 Story。
