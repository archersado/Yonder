# 设计

连接握手声明`user_input`后，Gateway为该双向连接注册短生命周期`AgentInputSink`。Application接受最终输入并生成稳定`input_id`，通过模型无关的`agent.input`只向交互开始时捕获的会话投递；Agent明确accepted后完成。断连、拒绝、deadline或结果未知均失败，不自动换目标或重试。

本地样本必须证明输入进入同一Agent会话。现有MCP stdio仅提供Agent调用工具的方向，不能作为通过证据；不能原生接入双工协议的本地Agent连接器须用自己的CLI、SDK或会话API实现并验证当前会话输入Adapter，Yonder核心不得感知具体Agent。连接器必须把每条输入提交到Runtime的当前会话入口：活动turn时steer，空闲或前一turn已中断时开始新turn；Runtime内部排队成功不等于`accepted`。云端Agent Runtime不是另一套业务逻辑，也不需要额外连接器：它直接通过主动WSS双工信道承载同一个`AgentSession`、接收`agent.input`并返回accepted。

正文位于有界内存消息中；日志、任务SQLite、任务事件和Outbox不含正文。协议由Rust类型生成Schema/TypeScript。

## 协议候选

- `gateway.hello`由Agent Runtime附带稳定`session_id`和`offered_capabilities: ["user_input"]`；能力只对当前连接有效。
- Yonder向同一连接发送JSON-RPC `agent.input`，字段为`input_id/session_id/source/content/created_at/deadline`。
- `content`为去除首尾空白后的非空UTF-8文本，最多16 KiB；`source`首批为`voice`或`selection`。
- 默认确认期限10秒且不得超过60秒；超过`deadline`、断连或响应无法判定均返回`unknown`，不自动重试。
- Agent以同一请求ID返回`accepted: true|false`；连接器按`input_id`去重，同一ID重复到达必须返回原结果且不得再次写入Agent会话。
- 上述类型由Rust协议生成Schema与TypeScript。本地UDS/Named Pipe和云端WSS只负责帧传输，不重新定义字段。

Gateway用连接注册表保存`agent_id + session_id → connection sink`，会话断开即移除。任务内输入按任务归属Agent选择其当前连接；桌宠独立输入使用显式激活连接；仅一个候选时自动激活。多个候选且没有激活项返回`target_required`，不按连接时间、进程名或Agent厂商猜测。交互开始时捕获目标连接，后续只检查该连接是否仍有效。

云端产品传输不由本Change新建连接。AG-S1先负责端点、认证、TLS与重连并向统一注册表提供已认证`AgentSession`；本Change只在该会话声明`user_input`后挂载同一sink。连接断开时未确认输入归unknown，不写Outbox、不自动重试。

同一连接注册表派生布尔连接快照。Desktop Adapter在会话注册/移除后直接向桌宠发布事件；页面初始化只读取一次快照。桌宠用独立徽标显示连接态，连接变化唤醒隐藏形态，不改变任务生命周期状态，也不轮询任务数据库。
