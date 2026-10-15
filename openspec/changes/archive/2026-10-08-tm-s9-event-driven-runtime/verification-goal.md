# 独立 Verification Goal：TM-S9 事件驱动执行运行时

Story: TM-S9
Result: PASS
日期：2026-10-08；平台：macOS。Windows 按用户既定决定暂缓，本结论不外推为双平台通过。

## 验证目标

- SQLite projector 延迟或失败时，Driver/Observe、计划片段游标及下一槽位仍由内存 reducer 连续推进。
- 状态、事件、Outbox、片段游标、HandBack 与控制记录在 projector 中保持同事务、连续序号和幂等 ACK。
- 硬背压只阻止下一副作用；取消、暂停、接管与交回立即改变运行时事实。
- `task.get` 与 `task.events` 返回未 checkpoint 的活动快照和连续尾部，不回退旧 SQLite 状态。
- 崩溃恢复保持保守语义；unknown 副作用不重试；发送确认仍在 Driver 派发前消费。

## 自动化结果

- `cargo test --offline --locked -p yonder-application -p yonder-adapters -p yonder-desktop`：Application 73 项、Adapters 83 项、Desktop lib 28 项、Desktop main 11 项全部通过。
- 关键新增样本证明：SQLite 片段游标仍为 0 且 projector 未 ACK 时，两个计划槽位已经连续派发；投影后游标、任务 sequence、events 与 Outbox 追平。运行时 takeover 随后立即得到 `paused/stopped`，再异步写入 control 记录。
- `execution_runtime` 回归覆盖 projector 可控失败重试、无 ACK 连续推进、硬背压、unknown/HandBack、连续 checkpoint、目标核验绑定和计划游标。
- Adapter/Desktop 回归覆盖事务回滚、幂等投影、重启 `running → interrupted`、直接运行中取消、显式接管、发送确认一次性消费和敏感信息不进入顶部控制条。
- `node scripts/verification/desktop/check-cua-control.mjs`：规划、快脑决策、Driver 轮次、Observe 结论、显式接管、非 CUA 展示、确认按钮、无障碍文案与脱敏全部通过。
- `cargo build --offline --locked -p yonder-desktop`、`git diff --check`、`openspec validate tm-s9-event-driven-runtime --strict`：通过。

## 门禁说明

`python3 scripts/check_architecture.py` 当前被既有 `TM-S8/product-requirements.md` 缺少“问题与目标”标准章节阻断。TM-S9 自身最初同样存在旧格式，本轮已补齐四份 Story 文档字段/章节并修正 delta spec；严格 OpenSpec 校验已通过。未越界修改 TM-S8。

## 结论

TM-S9 的 macOS 产品范围满足 Accepted AD-TM-23：活动执行不再等待 SQLite，持久化作为异步恢复检查点；控制、查询、计划片段与四类执行共享同一运行时。可进入 Archive；Windows 证据继续暂缓。
