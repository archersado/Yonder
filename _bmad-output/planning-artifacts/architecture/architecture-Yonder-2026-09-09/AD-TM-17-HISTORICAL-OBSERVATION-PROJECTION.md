# AD-TM-17 历史 Observe 事实投影

状态：Accepted；日期：2026-09-24。关联 TM-S5、TM5-AC01/TM5-AC03。Architecture Impact：architecture-change（只读协议 1.21 与 Task Space 展示）；不改变持久化、写入者或状态所有者。

## 来源与问题

产品简报「MVP 主干链路」第 9 步与「Task Space 与权限模型」要求保留可检查的执行历史。TM5-AC01 要求多步骤各自的实际观察不被最近一次详情覆盖；TM5-AC03 要求 unknown 与明确结论分开。Accepted AD-TM-01 AC10 已允许可信执行器把 Observe 结果、摘要与关联步骤同事务写入 `task_presentation_events`，但当前 `task.events` 只显示动作尝试结果，未展示该历史 Observe 事实。

## 决定

- Rust `TaskEvent` 在协议 1.21 增加可选 `observation`，复用现有 `TaskObservation { step_id, result, summary }` 类型。字段仅在事件同序号确有可信 `observation` 记录时出现；不能从任务最新快照反推历史。
- Adapter 只从同任务、同事件 `sequence` 的 `task_presentation_events.kind='observation'` 读取并严格解析既有 payload；畸形/不一致数据导致明确读取失败，不替换成当前观察、不跳过事件。无迁移和重写。
- Application 在鉴权、连续性检查后按协商版本投影：旧客户端 1.20 及以下不接收新字段；可信同版本桌面查询可见。仍受单事件 8 KiB 与整响应 256 KiB 预算约束。
- Task Space 将观察结果明确标为“Observe”，与 Agent 声明、动作成功及任务完成分别展示；unknown 不渲染成成功。摘要作为纯文本，不显示 Worker/宿主内部标识。
- 本决定只公开已经提交的观察事实，不新增 Observe 派发、轮次、证据附件、下一意图历史或产物版本承诺。后续若引入合法迟到 Observe，仍须单独设计写入契约。

## 验证

真实 SQLite 多步骤观察与旧版会话投影、畸形 payload 失败、权限隔离、字节预算和 UI 错误路径；macOS 原生界面证据。Windows 原生验证按用户决定暂缓，完整 TM-S5 不据此 Done。
