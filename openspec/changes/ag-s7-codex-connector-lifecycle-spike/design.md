# 设计

## 统一样本

对每种受测 Codex 客户端使用同一任务：建立任务 Gateway 连接和明确的输入/replan 会话，触发一次无副作用交回，确认输入进入同一 thread；随后依次重启 Yonder、终止连接器、重启 Codex 和切换 thread，检查通道状态、恢复行为与旧绑定失效。

每轮记录客户端/平台、公开接口、进程所有者、endpoint 类型、会话标识来源、Gateway 状态、输入状态、重连次数、稳定错误码和是否发生误投递。不得记录输入正文、完整 endpoint、thread ID、截图内容或 Agent Payload。

## 候选实现面

- Codex CLI：共享 App Server Unix Socket、remote TUI、显式 thread ID，以及由受监管启动器或 Codex 侧连接器托管的薄桥。
- ChatGPT Desktop：只检查公开插件、SDK、生命周期 hook 或公开 App Server endpoint；私有 stdio、内部数据库和进程注入直接淘汰。
- Windows：使用同一逻辑矩阵，将本地 endpoint 替换为受支持的 Named Pipe/本机通道，不以 macOS 结论替代。

## 淘汰门槛

任一候选出现以下情况即淘汰：需要猜测 thread、启动第二 Agent、读取私有状态、无法证明 active turn steer/idle start、断线后重放未确认输入、任务/输入连接态混淆、无限重连、或正文进入持久化/日志。

## 产出

- 客户端 × 平台能力矩阵；
- 冷启动与四类重启结构化证据；
- Accepted/Rejected/Deferred 路线；
- AD-AG-10 终局结论；
- 后续产品 Story 的进程所有者、状态机、安装启动、UI 与验证验收映射。
