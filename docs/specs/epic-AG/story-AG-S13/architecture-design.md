# AG-S13 架构设计

## 边界与依赖

### 状态所有权

`AgentInputHub` 继续作为在线 AgentSession、当前输入与回复关联的唯一运行时状态所有者。Desktop 新增只读会话摘要与选择用例；React/HTML 只渲染并提交不透明 `session_ref`，不保存第二份当前目标。圈选卡只负责打开 Agent 管理并消费选择完成通知，不读写选择状态。

```text
Local AgentSession register/unregister
  → AgentInputHub { sessions, active, active_explicit }
      → session_summaries() / select_session(session_ref)
          → Tauri Agent 管理窗口
      → deliver_selection / deliver_voice
```

## 状态与契约

### 选择规则

- `session_ref` 复用 Hub 为连接生成的进程内单调 token；只在当前进程生命周期有效。
- 第一个会话自动成为目标，`active_explicit=false`。
- 新的第二个会话到达时，自动目标清空；显式目标保持。
- 用户选择仍在线的引用后设置 `active_explicit=true`。
- 非目标断连不改变显式目标；目标断连后清空，再按剩余数量决定是否自动选择唯一会话。
- 重连得到新引用，不继承旧选择；替换同一 session 的连接只用于连接防重，不把 UI 引用解释成持久身份。

## 数据与错误边界

会话摘要只包含 `session_ref`、Agent ID、进程内显示序号、是否当前目标和是否支持附件/回复；不向 UI 暴露 session_id、thread ID、Socket、正文或附件。选择是本地用户对当前运行时的操作，不增加 Gateway 协议、数据库 schema 或任务事件。

选择不存在的引用返回 `agent-session-unavailable`；锁异常返回 `agent-session-state-unavailable`。圈选提交继续复用 `agent-target-required` 并保留草稿。

`agent_settings_open` 只允许圈选窗口调用；选择成功后 Desktop 向圈选窗口投递无正文、无标识的 `yonda-agent-target-changed` 通知，用于恢复可重试提示。通知不是状态源。

## 失败与验证

Hub 单元测试覆盖 0/1/2/3 会话、自动转显式、非目标断连、目标断连和陈旧引用。Desktop/UI 合约覆盖列表、当前标记、选择反馈与语音回复设置不回归；macOS 正式 GUI 使用多个真实 Codex 会话完成选择、圈选提交、回复朗读与停止。
