# 任务

- [x] 将 AD-AG-09 收敛为受限 macOS CUA 基线并更新 EX-S2 设计
- [x] 从 Rust 协议生成计划片段请求、响应、Schema 与 TypeScript
- [x] 实现 Application 受限片段候选校验与 Jev 选择器
- [x] 实现 SQLite 不可变片段、CAS、事件与 Outbox 原子持久化
- [x] 实现 Gateway 提交/同步有界连续执行、Jev 候选选择与交回
- [x] 覆盖 Domain、协议、Adapter/Gateway 集成测试
- [x] 修订 Rust 计划候选协议：受限动作语义、前置/预期 Observe 与确认引用；派生 Schema/TypeScript
- [ ] 实现本地一次性 CUA 发送确认、Jev 有界 Observe 输入与 Application 前后条件复核
- [ ] 将交回原因、慢脑唤醒与展示投影改为有界内存执行日志及异步 SQLite/Outbox 投影；执行关键路径不得等待慢脑读取或投递
- [x] 收敛快慢脑职责：单一已验证慢脑步骤直接执行；仅多候选 Observe 动作空间调用 Jev 选择；单次 Gateway 调用同步连续消费剩余槽位，并覆盖连续片段与异常交回验证
- [ ] 覆盖语义候选、确认不持久化、条件偏离交回及企业微信真实 CUA 样本
- [x] 建立并运行 macOS 独立 Verification Goal
- [x] 在安装包 CLI/MCP 导出计划提交与执行，并以真实 Codex MCP 证明快慢脑产品入口
- [ ] 归档 Change；Windows 证据保留为 EX-S2 Story 的未完成项
