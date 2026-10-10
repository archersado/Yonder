# Design

## 决策

将提交拆成“进入 submitting”和“按投递结果提交或回滚”。accepted 提交并清理；其他结果回滚到 reviewing 并保留临时数据。unknown 在 UI 层锁定再次发送，避免未知副作用自动重试。

附件字节上限直接引用 Rust 协议常量，在创建回复关联及发送附件帧之前失败。UI 按稳定错误码显示原因，不消费或记录截图正文。

## 非目标

不压缩截图、不持久化草稿、不新增重试队列、不改变 Agent 对 accepted/rejected/unknown 的定义。
