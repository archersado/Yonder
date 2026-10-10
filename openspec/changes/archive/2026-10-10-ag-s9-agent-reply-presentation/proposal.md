# Proposal：AG-S9 Agent 回复回传与圈选会话卡

关联 Story：AG-S9；关联 AD：Accepted AD-AG-11。

## Why

圈选输入已能进入当前 Codex thread，但 Yonder 在 accepted 后关闭卡片，用户必须切换客户端读取答案。当前 Connector 还会丢弃 App Server 正式回复通知。

## What Changes

- 扩展 AgentSession 协议，协商并回传与输入关联的有界 Agent 回复流。
- 分流 Codex App Server RPC response 与 notification，映射最终消息和 turn 终态。
- 圈选卡在 accepted 后继续展示等待、流式正文与终态。
- 回复只驻内存，不进入任务持久化、日志或索引。

## Impact

Architecture Impact：architecture-change，由 Accepted AD-AG-11 授权。影响 `crates/protocol`、`crates/application`、`apps/yonder-cli`、`apps/desktop` 及生成协议；不新增数据库迁移或第二传输。
