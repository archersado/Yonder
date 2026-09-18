# 任务

- [x] 完成AG-S5三份设计与来源映射
- [x] 建立Proposed AD-VI-02
- [x] 验证通用双向连接及macOS UDS、Windows Named Pipe、云端WSS协议样本
- [x] 撤回Codex当前会话候选通过结论并记录活动turn失败证据
- [x] 定稿Rust协议、能力协商、长度和deadline
- [x] 实现Application `AgentInputSink`与有界内存交付
- [ ] 将Codex薄桥接从内部队列改为当前会话start/steer输入，并验证中断后下一条输入可继续
  - [x] 记录Codex 0.154活动turn投递失败证据；`codex queue`成功不等于steer或accepted
  - [x] 移除`codex queue`误报桥接；不支持时失败关闭且不注册`user_input`
- [ ] 等待AG-S1交付端点、认证、TLS与重连完整的产品WSS `AgentSession`，再复用其接入`agent.input`
- [x] 接入VI-S1最终转写自动投递
- [x] 设计桌宠Agent连接态的事件、视觉与可访问反馈
- [x] 实现连接注册事件、启动快照与桌宠连接徽标
- [ ] 完成Windows/macOS独立Verification Goal
- [ ] Archive
