# Proposal：AG-S7 Codex 会话连接生命周期 Spike

关联 Story：AG-S7；关联 AD：Proposed AD-AG-10。

针对当前 Codex 接线需要用户手工启动 App Server、remote TUI、薄桥并传入 thread ID/Socket 的产品缺口，建立限时技术 Spike。对 Codex CLI 与 ChatGPT Desktop 分别验证可信当前会话绑定、连接器进程所有权、Yonder/连接器/Codex 重启恢复和连接状态反馈，输出可实施或明确不支持的路线。

Architecture Impact：architecture-change。当前 Change 只产出证据和终局 ADR，不实现守护进程、不新增 wire 协议、不修改 Codex 私有机制。
