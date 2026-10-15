# AG-S13 架构设计

## 边界与依赖

`AgentInputHub` 继续是输入连接与回复关联的唯一运行时状态所有者，但状态从 `HashMap + active` 收敛为单一 `Option<(token, SessionSink)>`。Desktop UI 不持有或选择会话。

```text
new authenticated AgentSession
  → AgentInputHub::register
      → replace current session atomically
      → clear old reply correlation
      → close old connection
      → route future selection/voice/reply to new session only
```

## 状态与契约

### 抢占与竞态

- 每次注册生成单调 token，并用新 `(token, sink)` 替换当前槽位。
- 旧连接关闭后调用 `unregister(old_session_id, old_token)`；只有 session ID 和 token 同时匹配当前槽位时才允许清空。
- 同一 session ID 重连也生成新 token，旧连接的延迟退出不能误删新连接。
- 抢占前若 `reply.session_id` 属于旧会话，则清除短生命回复关联；后续旧通知按现有 session/input/sequence 校验拒绝。
- `gateway_connected` 继续由独立 Gateway lease 计算，输入会话抢占不关闭任务 Gateway。

## 数据与错误边界

不新增协议、Tauri 会话命令、SQLite schema、事件或 Outbox。Agent 管理只保留注册授权与语音回复设置。零活动会话继续返回稳定的输入不可用错误；不再存在“请选择接收输入的 Agent”。

## 失败与验证

Hub 单元测试覆盖首连、后连关闭旧会话、同 ID 重连、旧 unregister、当前 unregister、回复抢占和只向唯一目标投递。Desktop/UI 合约确认会话选择表面已删除；macOS 正式 GUI 使用两个真实 Codex 连接验证后连抢占，再完成真实回复朗读与停止。
