# AG-S6 产品需求

## 问题与目标

Yonder 需要一个用户可控的本地 Agent 管理能力，用于显式登记、启用、禁用和撤权本机 Agent。该能力不改变 MVP 的 OS 用户私有端点认证边界，也不引入新的传输认证机制。

## 范围与非目标

- 登记、禁用和撤权本地 Agent。
- 撤权后阻断新建任务、任务控制和输入通道。
- 保留已有任务可见性，并允许用户继续查看和终止。

### 非目标

- 不实现云端登录、配对或令牌轮换。
- 不引入本地凭据存储、加密或第二传输协议。
- 不把本 Story 并入 AG-S2 当前实现。

## 验收条件

- REG-01：用户可在本地管理界面显式登记 Agent。
- REG-02：禁用 Agent 后，该 Agent 不能新建或控制任务。
- REG-03：撤权 Agent 后，其活动会话立即终止。
- REG-04：已有任务保持可见，可由用户查看和终止，不被自动删除。

## 需求来源与验收映射

- [AD-AG-08](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-AG-08-LOCAL-AGENT-REGISTRATION.md) → REG-01～04。
- [架构主干](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md)「Agent Gateway」与「任务、状态与恢复」→ REG-01～04。
