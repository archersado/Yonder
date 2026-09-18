# AD-TM-11 Agent 失败终态提交

状态：Accepted（2026-09-18，TM-S2 已观察失败子范围）。

## 问题

任务状态与桌宠均支持 `failed`，但正式 Gateway 只有 `task.complete`。Agent 无法把动作已确定失败且已 Observe 的任务安全终结；直接把 unknown 当失败会掩盖副作用不确定性，自由文本原因又会新增未定稿的审计正文。

## 决定

协议 1.18 增加 `task.fail` 与同名 capability，参数为 `agent_id/capability/deadline/task_id/expected_sequence`。仅归属 Agent、当前 running 任务、Desktop 资源仍被该任务持有、无 pending control、最新 attempt 已 stopped，且该 attempt 持久化的 Observe 结论明确为 `action_succeeded=false` 时接受。

失败依据只引用既有 attempt 结果和步骤，不接受自由文本、截图或 Payload。Application 以 `Action::Fail` 通过现有 SQLite CAS 事务原子提交任务状态、事件和 Outbox，成功后释放资源；冲突、unknown、动作成功、未停止或释放失败均不得报告 failed。重复请求按终态保护拒绝，不自动重试。

## 影响

Rust 协议仍是 JSON Schema 与 TypeScript 唯一来源；SQLite schema 不变。共享终结门禁对称校验结果：`task.complete` 只接受 `action_succeeded=true`，`task.fail` 只接受 `false`。桌宠只消费成功写响应产生的一次性终态展示，读取历史不重播。Windows 原生接入按用户要求暂缓；协议/Application/SQLite 合约先独立验证。
