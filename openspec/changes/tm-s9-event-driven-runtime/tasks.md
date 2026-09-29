# Tasks

- [x] 接受 AD-TM-23，完成 TM-S9 三份设计与 OpenSpec。
- [x] 定义 Application command/event、ActiveExecution reducer 与有界队列。
- [x] 实现 SQLite schema 22 的 events/Outbox 异步连续 projector、幂等批次与 checkpoint ACK。
- [x] 迁移 CUA `computer.execute` / `computer.step`，Driver 结果先进入 Runtime，再由 projector 持久化。
- [ ] 迁移 EX-S2 `task.plan.execute` 的片段游标、HandBack 与连续槽位。
- [x] 迁移 BUA、Document/Office、File、Command 与通用步骤边界到同一 Runtime。
- [ ] 迁移取消、接管、交回控制链路，并删除正式组合根的 legacy 同步分支。
- [ ] 将 Driver 阶段/effect/Observe 分类直接投影到顶部控制条和 Gateway 活动查询。
- [ ] 覆盖延迟、失败、背压、取消/接管、HandBack、崩溃恢复与发送确认测试。
- [ ] 建立并运行 macOS 独立 Verification Goal；Windows 暂缓。
- [ ] 四类执行与控制链路全部通过后 Archive。
