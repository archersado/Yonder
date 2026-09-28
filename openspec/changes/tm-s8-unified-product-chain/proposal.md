# Proposal：TM-S8 统一产品执行闭环

## Why

现有证据按能力与子范围分散，测试探针可以证明 Driver 可用，却不能证明用户拿到的是一套能由真实慢脑运行 CUA、BUA、Office 与 Command 的完整产品。

## What Changes

- 建立四条基于 Codex MCP 慢脑、Yonder/Jev 快脑、正式 Yonder GUI 和统一 Gateway 的产品验收任务。
- 统一验证创建可见性、慢脑计划、快脑有界选择/连续执行、交回、确认、running、Observe、事件/Outbox和终态。
- 修复真实链路暴露的状态分裂与读侧阻断：首动作前 Jev 交回仍原子进入 running，标称 v20 且审计表组全缺时安全建空表。
- 将顶部规划/执行步骤浮窗从 CUA 扩展到 BUA、Office/Document、Command 及统一 Gateway 的所有执行请求；非 CUA 不开放桌面接管。
- 验证发现的缺口返回所属 Story 实施；本 Change 不建立第二编排器或测试专用产品路径。

## Impact

主要影响验证、MCP 产品接入与跨模块组合根；若验证暴露协议、状态所有权或持久化缺口，先更新对应 Accepted ADR 和能力 Story，再实施。
