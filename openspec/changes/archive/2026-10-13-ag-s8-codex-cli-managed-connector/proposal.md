# Proposal：AG-S8 macOS Codex CLI 托管连接器

关联 Story：AG-S8；关联 AD：Accepted AD-AG-10。

在 macOS 上以 Codex 公开 `SessionStart` Hook、共享 daemon、WebSocket-over-proxy 和显式 `session_id` 自动建立既有 Agent 输入/replan 通道。用户完成一次 Hook 安装与信任后，不再手填 thread ID、Socket 或单独运行桥；Yonder 重启时连接器有界重连。连接器同时补齐既有 CX-S2 同会话附件能力，让圈选截图与文字进入同一 Codex thread。

Architecture Impact：conforming。实现已接受的连接器生命周期边界与 Accepted AD-CX-02，不新增 wire 协议、持久化或状态所有者；附件只在连接器有界内存中完成校验和投递。ChatGPT Desktop 保持 Unsupported/Deferred，Windows 保持 Deferred。
