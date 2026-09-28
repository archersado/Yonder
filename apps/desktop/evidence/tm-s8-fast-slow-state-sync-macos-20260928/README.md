# TM-S8 快慢脑状态同步 macOS 产品证据

日期：2026-09-28。构建：`dev` 正式打包 `Yonda.app`，Codex MCP 指向安装包内 `yonder mcp`。本样本不使用测试宿主、私有业务脚本或数据库写入，不发送消息。

外部 Codex 慢脑经 MCP 创建任务并提交一个含两个候选的 CUA 计划槽位；Yonder 内置 Jev 快脑实际参与选择并在首个 Driver 动作前交回。修复后的产品结果：计划提交 `accepted`，计划执行 `handback`，任务快照为 `running`，下一意图为“需要慢脑重新 Observe 或规划”，事件序列为 `created→created`、`created→created`、`created→running`。

同一正式库的 `task.get` 与 `task.events` 均成功；启动时只为原本完整缺失的 schema 18 审计表组创建空表，保留既有任务、事件、Outbox 与 `user_version=20`。结构化结果见 `result.json`，不含任务 ID、凭据、正文、截图、Driver 参数或完整 Agent Payload。
