# AG-S8 macOS Codex CLI 托管连接器

Story: AG-S8  
Epic: AG  
Status: verifying
OpenSpec: ag-s8-codex-cli-managed-connector

## 目标

让 macOS 用户完成一次 Hook 安装与信任后，只需正常启动 Yonder 和 Codex CLI。当前 Codex 会话自动向 Yonder 注册为慢脑输入通道；Yonder 重启时连接器恢复同一显式会话，不再要求用户复制 thread ID、维护 Socket 或单独运行 `agent-bridge`。

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)
- [Accepted AD-AG-10](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-AG-10-CODEX-CONNECTOR-LIFECYCLE.md)

## 范围

- 本 Story 只交付 macOS Codex CLI 0.162.1+。
- ChatGPT Desktop 显示为不支持，不使用私有 stdio 或进程注入。
- Windows 按主人决定继续挂起，不宣称跨平台 Done。

关联变更：[ag-s8-codex-cli-managed-connector](../../../../openspec/changes/ag-s8-codex-cli-managed-connector/proposal.md)。

2026-10-10 实现与自动化合约已通过并合并 `dev`：正式插件已被本机 Codex CLI 安装、启用并解析。真实探针发现 `app-server proxy` 不提供 JSONL RPC，且独立 stdio App Server 会与当前 active writer 冲突；实现改为在 proxy 原始字节流上完成 WebSocket 协议。独立 Verification Goal 继续验证真实生命周期，不绕过 Hook 信任。
