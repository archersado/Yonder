# 设计

Application 定义完整执行身份、可信内部目标、单一文本输入动作和 `known/unknown` 结果。Adapter 校验可信组合根提供的绝对 Node/Worker/SDK 文件，按需启动一次 Worker，经继承 stdio 发送一个动作；Worker 前置 Observe 唯一编辑元素，执行 SDK 输入，再强制后置 Observe。Rust 只接受身份完全一致且后置 Observe 有效的结果。进程错误、超时、非法回包或观察失败为 unknown，动作不重试。
