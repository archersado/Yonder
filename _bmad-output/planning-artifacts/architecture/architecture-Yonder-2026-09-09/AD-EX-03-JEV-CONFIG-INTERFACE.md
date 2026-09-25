# AD-EX-03 Jev 最小配置界面

状态：Accepted
日期：2026-09-22
关联：EX-S2、AD-EX-01、AD-EX-02

## 决策

Yonder 的 Application 层拥有一个最小 Jev 配置模型，只包含非敏感字段：启用状态、服务形态、端点、每片段步数/时间/token 上限和允许的执行层。桌面 UI 只负责展示与收集，不直接访问模型、Adapter、配置文件或任务状态。配置通过 Application 校验后由 SQLite Adapter 原子保存为单行 `jev_config` JSON。

2026-09-24 用户变更：设置窗口增加 API Key 密码输入框。该字段不属于 `JevConfig`，仅在用户提交时经本地 Tauri 调用传至 macOS 原生 Keychain 写入端口；读取接口只返回 `credential_configured` 布尔值。Keychain 状态查询与写入不依赖 `TaskHost` 或 SQLite 初始化。空字段不清除旧值，非空字段只可替换，密钥不得写入 SQLite、任务、事件、Outbox、日志、进程参数、环境变量或 UI 回显。

2026-09-22 用户变更：Jev 配置界面是独立的桌面设置窗口，入口放在系统托盘菜单；配置区不得嵌入 Task Space 任务面板。Task Space 只承担任务可见性与控制，配置窗口只承担 Jev 配置。

该配置界面不授权 Jev 执行，也不改变 AD-EX-02 对执行路线的双平台验证门禁。保存成功仅表示配置通过结构校验，不代表模型可用；片段启动时仍须按计划授权值与全局上限取较小值，并在派发前重新校验。

## 后果

- 配置不属于任务状态，不写事件或 Outbox。
- API Key 只保存到 macOS Keychain；设置页不能导出或展示既有值。
- 独立配置窗口只通过本机 Tauri 命令调用 Application；Task Space 不承载配置交互。
- 无效配置必须被拒绝，不能进入片段派发。
- 关闭 Jev 等同用户控制，运行中片段在下一步决策前冻结并交回慢脑。
