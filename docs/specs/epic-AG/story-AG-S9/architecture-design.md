# AG-S9 架构设计

## 边界与依赖

Rust 协议仍是 JSON Schema 与 TypeScript 的唯一来源。CLI 只依赖 protocol；Desktop 通过 application 暴露的 Agent 输入类型消费协议，不让 UI 成为连接或状态所有者。

```text
Yonder 圈选卡
  └─ AgentInputHub: input_id + session_id 临时关联
      └─ 既有 Agent Gateway / AgentSession
          └─ Codex Connector
              ├─ turn/start | turn/steer
              ├─ item/started + item/agentMessage/delta
              └─ turn/completed
          └─ agent.reply(started/delta/completed|failed|interrupted)
      └─ Tauri 只向当前 region-preview 发有界 UI 事件
```

## 状态与契约

- 协议新增 `user_input_reply` 协商能力与可选 `reply_mode=stream`。
- 新增 Agent→Yonder `agent.reply` 通知：`input_id/session_id/reply_id/sequence/phase/content?/truncated`。
- `AgentInputHub` 只允许当前唯一 session 的一个 reply 请求，保存当前输入标识、期望序号与终态；正文由 region-preview WebView 内存持有。
- Connector 的 App Server reader 把 RPC response 与 notification 分流，不能再让一个 RPC call 丢弃别的 response 或全部通知。
- Connector 在 start/steer 得到可信 turn ID 后先返回输入 accepted，再发 started；仅消费该 turn 后续最终消息。明确 commentary/partial_answer 不进入回答正文；phase 缺失按兼容消息处理。
- Connector 对正文实施 64 KiB 累计上限；每段协议上限 16 KiB。超限停止正文并以 truncated 完成，不记录被截内容。
- Connector 对单次回复设置 5 分钟运行时上限；超时发送 `interrupted` 并释放关联，不补抓历史正文。

## 失败与验证

- 回复请求已存在时拒绝第二个需要回复的输入；普通 voice/replan/任务请求不被阻塞。
- Agent reply 通知与 Yonder→Agent 输入响应在同一全双工连接复用，由单连接读循环按帧类型分派。
- 连接断开或 Hook 退出时，当前卡片进入 interrupted；不重连补抓正文、不自动重投输入。
- UI 关闭只释放 Yonder 展示订阅，不取消 Codex turn；迟到通知被输入关联门禁丢弃。

## 安全与验证

- 只转发最终用户可见 Agent 消息，不转发 reasoning、命令输出、工具参数或完整 Payload。
- 日志只允许阶段、字节数和稳定错误分类，不得写正文或标识全文。
- 测试层级：协议生成/校验、Application/Hub 单元、Gateway 双向合约、Codex App Server notification 合约、macOS 正式圈选 E2E。
