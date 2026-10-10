# AG-S9 产品需求

## 问题与目标

当前圈选链路只显示“已发送给 Agent”并关闭。用户无法在 Yonder 中看到 Agent 对截图问题的回答，产品体验被拆成 Yonder 提问、Agent 客户端读答案两段。

## 需求来源与验收映射

- 原始需求：产品简报“桌宠是轻量交互入口”与 CX-S2“屏幕圈选后提问”。
- 后续用户变更：2026-10-10 用户要求圈选发送后在 Yonder 返回 Agent 消息，并同意统一使用既有 Gateway。
- 架构约束：Accepted AD-AG-11、AD-AG-10、AD-CX-02、AD-VI-02；不得新增旁路、持久化正文或读取 Agent 私有历史。
- 官方能力依据：Codex App Server v2 提供 `item/agentMessage/delta` 和 `turn/completed`；只有 completed 状态证明 turn 成功。

## 范围与非目标

1. 在 AgentSession 上协商圈选回复能力，并把回复与本次输入关联。
2. Yonder 圈选卡展示发送中、Agent 正在回复、流式正文、完成、失败和中断。
3. 回复正文严格有界、仅驻内存，关闭后清除。
4. Connector/Yonder 断连、回复超限或 App Server 非成功终态必须可诊断。

### 非目标

- 不建立通用聊天历史、消息数据库或多会话聊天 UI。
- 不把回复写入任务、事件、Outbox、索引或日志。
- 不展示模型 reasoning、工具调用原文或完整 App Server Payload。
- 不支持 Windows 或 ChatGPT Desktop。

## 验收条件

- AC1：支持回复能力的当前 Agent 接收圈选输入后，Yonder 卡片保持打开并进入“Agent 正在回复”。
- AC2：最终 Agent 消息按严格序号流式显示；图片、问题与答案始终绑定同一 `input_id + session_id`。
- AC3：只有 `turn/completed.status=completed` 显示完成；failed/interrupted/断连显示明确非成功状态。
- AC4：未知输入、跨会话、重复/跳号、终态后通知、单段或累计超限均被拒绝或安全截断，不污染其他卡片。
- AC5：输入 accepted 与回答 completed 分开表达；等待期间用户可关闭，关闭即清除本地正文且不取消 Agent turn。
- AC6：回复、问题、截图、turn/item 标识不进入 SQLite、事件、Outbox、日志或索引。
- AC7：不支持回复能力的 Agent 保持原有“已发送”行为，并明确提示回答需在 Agent 客户端查看。
