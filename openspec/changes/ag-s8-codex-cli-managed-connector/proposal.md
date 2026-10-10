# Proposal：AG-S8 macOS Codex CLI 托管连接器

关联 Story：AG-S8；关联 AD：Accepted AD-AG-10。

在 macOS 上以 Codex 公开 `SessionStart` Hook、共享 App Server daemon/proxy 和显式 `session_id` 自动建立既有 Agent 输入/replan 通道。用户完成一次 Hook 安装与信任后，不再手填 thread ID、Socket 或单独运行桥；Yonder 重启时连接器有界重连。

Architecture Impact：conforming。实现已接受的连接器生命周期边界，不新增 wire 协议、持久化或状态所有者。ChatGPT Desktop 保持 Unsupported/Deferred，Windows 保持 Deferred。
