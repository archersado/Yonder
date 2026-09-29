# Tasks

- [x] 接受 AD-TM-23，完成 TM-S9 三份设计与 OpenSpec。
- [ ] 定义 Application command/event、ActiveExecution reducer 与有界队列。
- [ ] 实现 SQLite/events/Outbox 异步连续 projector 与 checkpoint ACK。
- [ ] 迁移 CUA `computer.step` 与 EX-S2 `task.plan.execute`，移除 Driver 生命周期内 SQLite/TaskHost 锁。
- [ ] 将 Driver 阶段/effect/Observe 分类直接投影到顶部控制条和 Gateway 活动查询。
- [ ] 覆盖延迟、失败、背压、取消/接管、HandBack、崩溃恢复与发送确认测试。
- [ ] 建立并运行 macOS 独立 Verification Goal；Windows 暂缓。
- [ ] 验证后迁移 BUA、Document、Command 并 Archive。
