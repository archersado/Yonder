# Design：真实慢脑统一产品验证

以 `dev` 构建 Yonder.app 与安装包内 CLI，配置 Codex MCP 指向该 CLI。每条样本由独立 Codex 慢脑创建任务和提交已验证计划片段；Yonder/Jev 快脑至少在一个真实多候选步骤中完成有界选择和连续推进，异常再交回同一 Codex 会话。所有推进只使用 MCP 暴露的产品工具。Yonder GUI 和 SQLite/事件只作为产品事实与只读证据，不向验证器提供旁路写入口。

验证顺序为 CUA→BUA→Office→Command。前一条失败不伪造通过，但不阻止验证不依赖该失败的下一条；共同 Gateway/任务状态缺陷优先修复。CUA 使用企业微信草稿并在发送前确认；BUA 使用无账号副作用的公开页面；Office 使用临时 DOCX 默认另存；Command 使用无 Shell、无网络、无删除的结构化命令。样本完成后清理临时文件和非用户数据，不删除任务审计。

按 AD-TM-21，Jev 首动作前交回也属于已开始的计划执行，持久化必须原子写为 `created→running`；`next_intent` 只描述等待慢脑，不得把任务降回未开始。正式库启动时核验审计表组，全缺才建空表，部分缺失失败关闭。

顶部执行浮窗消费 Gateway 已验证的通用 `execution_presentation_hint`，投影任务 ID、当前步骤及可用计划片段，不读取动作参数或正文。CUA 子提示只决定是否允许“接管电脑”，不再决定浮窗是否出现。UI 根据 `desktop_control` 区分 CUA 与非 CUA 标题/按钮；Rust 窗口 label 校验和 Hub 再次拒绝非 CUA 接管，避免表现层扩大权限。
