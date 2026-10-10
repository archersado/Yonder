# Design

## 决策

在既有全双工 AgentSession 增加 `user_input_reply` 能力、输入回复模式和 `agent.reply` 通知。Desktop 保持一个临时 pending reply；Connector 从 Codex App Server 正式 notification 提取最终可见消息，按严格序号回传。

## 数据流

1. Desktop 提交 `agent.input(reply_mode=stream)`。
2. Connector start/steer 成功后返回 accepted，并建立 `input_id ↔ turn_id` 临时关联。
3. Connector 发 started，随后只转发该 turn 的 final/兼容 agentMessage delta。
4. `turn/completed` 的 status 决定 completed/failed/interrupted。
5. Desktop 校验关联和序号后把事件发送给当前圈选卡；终态或关闭清理。

## 限制

单 session 单 pending reply；delta 16 KiB、累计正文 64 KiB、单次回复最长 5 分钟。超时发送中断终态并释放关联。正文、截图、问题与厂商 turn/item 数据不持久化、不记录日志。
