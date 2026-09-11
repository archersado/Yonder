# 提案：对照 CUA Driver

## 为什么

CUA Driver 决定跨平台输入、观察与故障恢复能力，必须在产品 Story 前实测定案。

## 变更

建立无 Agent 的统一黑盒 Harness，对照 Qwen CUA SDK 0.20.5 与 trycua CUA Driver 0.25.0。旧 Qwen open-computer-use 仅记录为历史基线。

## 影响

- Story：E0-S2
- 架构影响：architecture-change（候选尚未定案）
- 决策输出：AD-E0-02
- 产品代码影响：无，仅 Spike
- 协议影响：形成 Yonder CUA Driver Port 的最小能力要求
- 迁移影响：无
