# 任务

- [x] 建立 AD-CM-02 并同步 CM-S1 三份设计与来源映射
- [x] 建立本 Change 的 Proposal 与设计
- [x] 从 Rust 协议唯一来源派生命令提议/执行类型与生成物
- [x] 从 Rust 协议唯一来源派生无正文命令批准响应摘要与生成物
- [x] 接入 CLI/MCP 的命令提议/执行映射（Gateway 未协商时保持失败关闭）
- [x] 接入 Gateway 1.30 能力协商与无副作用命令提议登记
- [ ] 接入已批准命令的一次性受控执行
- [x] 实现有界内存批准 Registry：LocalUser-only 预览/批准/拒绝、摘要绑定、一次消费与过期
- [x] 接入 TaskHost 的本机预览、批准与拒绝入口
- [x] 接入 Task Space 本机批准卡
- [ ] 实现复用 TM-S7 启动事务的受控 Command 执行用例
- [x] 接入 macOS TaskHost、Gateway 与 Task Space 批准卡
- [ ] 覆盖替换、重放、撤权、超时、unknown 不重试及无正文边界
- [ ] 创建独立 Verification Goal 与 macOS 原生证据
- [ ] Windows 按用户决定延期；完整 Story 暂不 Archive
