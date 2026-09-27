# AD-AG-08 本地 Agent 注册与撤权

状态：Accepted
日期：2026-09-23
关联：AG-S2、AD-AG-01、AD-AG-05

## 待决问题

AD-AG-05 已把当前 OS 用户私有 UDS 定为 MVP 本机认证边界；`agent_id` 只做任务归属隔离，不是细粒度身份认证。还缺一个后续设计：是否以及如何提供本地 Agent 注册、撤权和会话终止。

## 候选决策

本决策只定义后置产品能力，不改变 MVP：

- 本地传输继续沿用“当前 OS 用户 + 私有 UDS/Named Pipe”作为认证边界。
- 细粒度 Agent 注册、撤权和会话终止是 MVP 之后的产品能力，另建 Story/OpenSpec 实施。
- 已完成当前 OS 用户私有 UDS/Named Pipe 或受控本机 stdio 身份绑定的 Agent，可由 Desktop 组合根在首次成功握手前自动登记为 `enabled`；该自动登记只信传输绑定身份，不信请求 JSON 自报的 `agent_id`。
- 云端 WSS、未知传输或无法证明本机身份的连接绝不自动登记，仍须经用户面板显式授权。
- 注册信息持久化在 SQLite，用户仍可显式启用/禁用 Agent；撤权立即断开该 Agent 的活动会话。
- 撤权后该 Agent 不能新建或控制任务，已有任务保持可见、仍可由用户查看和终止。
- 不引入本地凭据存储、不新增传输认证机制、不改变现有 Gateway 协议。

## 架构影响

Architecture Impact：architecture-change。本决策不改变 Gateway wire 协议、传输认证边界或依赖方向；实施新增独立 `AgentRegistry` Port 与 SQLite `agent_registry` 表，撤权由同一桌面宿主锁串行化并立即断开活动输入会话。实施必须通过独立 Story、OpenSpec 和验证 Goal，不能并入 AG-S2 当前范围。
